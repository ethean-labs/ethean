//! Aggregator duty (leanSpec `lstar/aggregation.py::aggregate`).
//!
//! For each attestation data with fresh signatures: reuse pooled proofs as
//! children (greedy new coverage), add the uncovered raw signatures, and ask
//! the prover for one Type-1 over the union. Finished proofs are re-verified,
//! pooled, and published on the aggregation topic.

use ethean_crypto::AggregateVerifier;
use ethean_multisig::{limits::MAX_CHILDREN, KeyedProof, LeanMultisigVerifier};
use ethean_network_wire::{fork_segment_from_name, topic_aggregation};
use ethean_primitives::Hash32;
use ethean_types::{
    AggregatedAttestation, AggregationBits, AttestationData, SignedAggregatedAttestation,
    SingleMessageAggregate,
};

use crate::aggregation_gossip::AggregationGossip;
use crate::chain_owner::ChainOwner;
use crate::events::ChainEvent;
use crate::gossip_attestation::{insert_proved, pool_key};
use crate::proof_service::ProofJob;
use crate::registry_keys_view::{attestation_key, attestation_keys_for_bits};

fn set(bits: &mut Vec<bool>, index: usize) {
    if bits.len() <= index {
        bits.resize(index + 1, false);
    }
    bits[index] = true;
}

/// A pooled proof and the participant bits it covers.
type BitsAndProof = (Vec<bool>, Vec<u8>);

/// Pooled proofs for `data_root` chosen greedily by new coverage, with the
/// union of validators they cover.
fn select_children(owner: &ChainOwner, data_root: Hash32) -> (Vec<BitsAndProof>, Vec<bool>) {
    let mut variants: Vec<BitsAndProof> = owner
        .aggregates
        .variants(&pool_key(data_root))
        .unwrap_or_default()
        .into_iter()
        .filter(|e| !e.proof.is_empty())
        .filter_map(|e| {
            let att = AggregatedAttestation::ssz_decode(&e.attestation_ssz).ok()?;
            Some((att.aggregation_bits.bits, e.proof))
        })
        .collect();
    variants.sort_by_key(|(bits, _)| std::cmp::Reverse(bits.iter().filter(|b| **b).count()));
    let mut covered = Vec::new();
    let mut chosen = Vec::new();
    for (bits, proof) in variants {
        if chosen.len() == MAX_CHILDREN {
            break;
        }
        let adds = bits
            .iter()
            .enumerate()
            .any(|(i, b)| *b && !covered.get(i).copied().unwrap_or(false));
        if adds {
            for (i, b) in bits.iter().enumerate() {
                if *b {
                    set(&mut covered, i);
                }
            }
            chosen.push((bits, proof));
        }
    }
    (chosen, covered)
}

/// Interval in which aggregators prove the pooled votes (leanSpec lstar
/// `tick_interval`): once per slot, after the interval-1 votes have spread.
pub const AGGREGATION_INTERVAL: u8 = 2;

/// Head state when this node may queue aggregation work.
fn aggregation_state(owner: &ChainOwner) -> Option<ethean_types::State> {
    let skipped = if !owner.is_aggregator {
        "not_aggregator"
    } else if owner.syncing {
        "not_synced"
    } else if owner.prover.is_none() {
        "other"
    } else if let Some(state) = owner.head_state.as_ref() {
        return Some(state.clone());
    } else {
        "missing_state"
    };
    crate::lean_metrics::aggregation_skipped(skipped);
    None
}

/// Merge partial aggregates of other subnets as soon as they arrive; run on
/// the intervals where [`schedule_aggregations`] does not.
pub fn schedule_union_merges_only(owner: &mut ChainOwner) -> Vec<ChainEvent> {
    let mut events = Vec::new();
    if !owner.is_aggregator || owner.syncing || owner.prover.is_none() {
        return events;
    }
    if let Some(state) = owner.head_state.clone() {
        schedule_union_merges(owner, &state, &mut events);
    }
    events
}

/// Queue Type-1 jobs for every attestation data with fresh signatures.
pub fn schedule_aggregations(owner: &mut ChainOwner) -> Vec<ChainEvent> {
    let mut events = Vec::new();
    let Some(state) = aggregation_state(owner) else {
        return events;
    };
    for data_root in owner.signatures.roots() {
        if owner
            .prover
            .as_ref()
            .is_some_and(|p| p.attestation_in_flight(&data_root))
        {
            continue;
        }
        let Some(data) = owner.signatures.data(&data_root).cloned() else {
            continue;
        };
        let (children, covered) = select_children(owner, data_root);
        let fresh = owner.signatures.uncovered(&data_root, &covered);
        if fresh.is_empty() && children.len() < 2 {
            continue;
        }
        let mut participants = covered;
        let mut raw = Vec::with_capacity(fresh.len());
        for (index, signature) in fresh {
            match attestation_key(&state, index) {
                Ok(key) => {
                    set(&mut participants, index as usize);
                    raw.push((key, signature));
                }
                Err(e) => tracing::debug!(index, error = %e, "signature dropped"),
            }
        }
        submit_type1(
            owner,
            &state,
            data,
            participants,
            raw,
            children,
            &mut events,
        );
    }
    schedule_union_merges(owner, &state, &mut events);
    events
}

/// Slots back from the current one whose partial aggregates are still merged.
const UNION_WINDOW_SLOTS: u64 = 2;

/// Aggregators on different subnets each prove only their own votes, so no
/// single pooled proof reaches a supermajority. leanSpec folds those proofs
/// into a child merge while building the block; Ethean keeps that off the
/// proposer's path and merges here, one round after the partial aggregates
/// were gossiped. Only the aggregator owning subnet `slot % committees` runs
/// the merge, so the others do not prove the same union again.
fn schedule_union_merges(
    owner: &mut ChainOwner,
    state: &ethean_types::State,
    events: &mut Vec<ChainEvent>,
) {
    let current = owner.last_tick.map(|t| t.slot.get()).unwrap_or(0);
    let min_slot = current.saturating_sub(UNION_WINDOW_SLOTS);
    for key in owner.aggregates.mergeable_keys_since(min_slot) {
        let data_root = key.message_root;
        if owner
            .prover
            .as_ref()
            .is_some_and(|p| p.attestation_in_flight(&data_root))
        {
            continue;
        }
        let Some(data) = owner
            .aggregates
            .variants(&key)
            .unwrap_or_default()
            .iter()
            .find_map(|e| AggregatedAttestation::ssz_decode(&e.attestation_ssz).ok())
            .map(|a| a.data)
        else {
            continue;
        };
        if data.slot.get() < min_slot
            || !owner.aggregates_subnet(data.slot.get() % owner.attestation_committees())
        {
            continue;
        }
        let (children, covered) = select_children(owner, data_root);
        if children.len() < 2 {
            continue;
        }
        tracing::debug!(
            slot = data.slot.get(),
            children = children.len(),
            coverage = covered.iter().filter(|b| **b).count(),
            "union merge queued"
        );
        submit_type1(owner, state, data, covered, Vec::new(), children, events);
    }
}

fn submit_type1(
    owner: &mut ChainOwner,
    state: &ethean_types::State,
    data: AttestationData,
    participants: Vec<bool>,
    raw: Vec<(ethean_crypto::PublicKey, ethean_crypto::Signature)>,
    children: Vec<BitsAndProof>,
    events: &mut Vec<ChainEvent>,
) {
    let data_root = data.hash_tree_root();
    let children: Vec<KeyedProof> = children
        .into_iter()
        .filter_map(|(bits, proof)| {
            Some(KeyedProof {
                public_keys: attestation_keys_for_bits(state, &bits).ok()?,
                proof,
            })
        })
        .collect();
    let job = ProofJob::Attestation {
        data,
        participants,
        raw,
        children,
    };
    if owner.prover.as_mut().is_some_and(|p| p.submit(job)) {
        events.push(ChainEvent::ProofScheduled {
            kind: "attestation",
            root: data_root,
        });
    } else {
        crate::lean_metrics::aggregation_skipped("spawn_failed");
    }
}

/// Re-verify, pool and publish a finished Type-1 proof.
pub fn accept_attestation_proof(
    owner: &mut ChainOwner,
    data: AttestationData,
    participants: Vec<bool>,
    proof: Vec<u8>,
    building: std::time::Duration,
) -> Result<ChainEvent, String> {
    let state = owner.head_state.as_ref().ok_or("no head state")?;
    let keys = attestation_keys_for_bits(state, &participants)?;
    let data_root = data.hash_tree_root();
    LeanMultisigVerifier
        .verify_single(&proof, &keys, &data_root, data.slot.get())
        .map_err(|e| format!("prover returned an invalid proof: {e}"))?;
    let coverage = keys.len() as u32;
    let proof_len = proof.len();
    crate::lean_metrics::aggregate_built(coverage, building);
    tracing::debug!(
        slot = data.slot.get(),
        coverage,
        elapsed_ms = building.as_millis() as u64,
        "aggregate proved"
    );
    insert_proved(owner, &data, participants.clone(), proof.clone())?;
    owner.signatures.remove_covered(&data_root, &participants);
    if let Some(fork) = owner.profile.as_ref().map(|p| p.fork_name) {
        let signed = SignedAggregatedAttestation {
            data,
            proof: SingleMessageAggregate::new(
                AggregationBits::new(participants).map_err(|e| e.to_string())?,
                proof,
            )
            .map_err(|e| e.to_string())?,
        };
        let segment = fork_segment_from_name(fork).map_err(|e| e.to_string())?;
        owner.pending_aggregation_gossip.push(AggregationGossip {
            topic: topic_aggregation(&segment).map_err(|e| e.to_string())?,
            payload: signed.ssz_encode().map_err(|e| e.to_string())?,
            data_root,
            proof_len,
        });
    }
    Ok(ChainEvent::AggregateProved {
        data_root,
        coverage,
        proof_len,
    })
}

/// Re-verify and pool a Type-1 recovered from a block proof. It is a known
/// payload: it joins the next merge as a child but is not gossiped again.
pub fn accept_recovered_proof(
    owner: &mut ChainOwner,
    data: AttestationData,
    participants: Vec<bool>,
    proof: Vec<u8>,
    building: std::time::Duration,
) -> Result<ChainEvent, String> {
    let state = owner.head_state.as_ref().ok_or("no head state")?;
    let keys = attestation_keys_for_bits(state, &participants)?;
    let data_root = data.hash_tree_root();
    LeanMultisigVerifier
        .verify_single(&proof, &keys, &data_root, data.slot.get())
        .map_err(|e| format!("recovered proof does not verify: {e}"))?;
    let coverage = keys.len() as u32;
    let proof_len = proof.len();
    tracing::debug!(
        coverage,
        proof_len,
        elapsed_ms = building.as_millis() as u64,
        "block vote proof recovered"
    );
    insert_proved(owner, &data, participants, proof)?;
    owner.known_payloads.insert(data_root, data.slot.get());
    Ok(ChainEvent::AggregateProved {
        data_root,
        coverage,
        proof_len,
    })
}
