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

/// Queue Type-1 jobs for every attestation data with fresh signatures.
pub fn schedule_aggregations(owner: &mut ChainOwner) -> Vec<ChainEvent> {
    let mut events = Vec::new();
    if !owner.is_aggregator {
        crate::lean_metrics::aggregation_skipped("not_aggregator");
        return events;
    }
    if owner.syncing {
        crate::lean_metrics::aggregation_skipped("not_synced");
        return events;
    }
    if owner.prover.is_none() {
        crate::lean_metrics::aggregation_skipped("other");
        return events;
    }
    let Some(state) = owner.head_state.clone() else {
        crate::lean_metrics::aggregation_skipped("missing_state");
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
        let children: Vec<KeyedProof> = children
            .into_iter()
            .filter_map(|(bits, proof)| {
                Some(KeyedProof {
                    public_keys: attestation_keys_for_bits(&state, &bits).ok()?,
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
    events
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
