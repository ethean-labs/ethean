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
        .unwrap_or(&[])
        .iter()
        .filter(|e| !e.proof.is_empty())
        .filter_map(|e| {
            let att = AggregatedAttestation::ssz_decode(&e.attestation_ssz).ok()?;
            Some((att.aggregation_bits.bits, e.proof.clone()))
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

/// True when this node may queue aggregation work (no head-state copy).
fn aggregator_ready(owner: &ChainOwner) -> bool {
    let skipped = if !owner.is_aggregator {
        "not_aggregator"
    } else if owner.syncing {
        "not_synced"
    } else if owner.prover.is_none() {
        "other"
    } else if owner.head_state.is_none() {
        "missing_state"
    } else {
        return true;
    };
    crate::lean_metrics::aggregation_skipped(skipped);
    false
}

/// Merge partial aggregates of other subnets as soon as they arrive; run on
/// the intervals where [`schedule_aggregations`] does not.
pub fn schedule_union_merges_only(
    owner: &mut ChainOwner,
    tick: ethean_validator::DutyTick,
) -> Vec<ChainEvent> {
    let mut events = Vec::new();
    if !owner.is_aggregator || owner.syncing || owner.prover.is_none() {
        return events;
    }
    if crate::duty_propose::prover_reserved_for_block(owner, tick) {
        tracing::debug!("union merge deferred: prover reserved for the next block");
        return events;
    }
    if owner.head_state.is_some() {
        schedule_union_merges(owner, &mut events);
    }
    events
}

/// Queue Type-1 jobs for every attestation data with fresh signatures.
pub fn schedule_aggregations(owner: &mut ChainOwner) -> Vec<ChainEvent> {
    let mut events = Vec::new();
    if !aggregator_ready(owner) {
        return events;
    }
    if owner.prover.as_ref().is_some_and(|p| p.block_in_flight()) {
        tracing::debug!("type-1 deferred: block proof in flight");
        return events;
    }
    let next_is_ours = owner
        .last_tick
        .is_some_and(|t| crate::duty_propose::proposes_next_slot(owner, t));
    if defer_type1_for_early_block(owner) {
        tracing::debug!("type-1 deferred: next slot is ours and a body proof is pooled");
    } else {
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
            let mut participants = covered;
            let mut raw = Vec::with_capacity(fresh.len());
            for (index, signature) in fresh {
                let key = match owner.head_state.as_ref() {
                    Some(state) => match attestation_key(state, index) {
                        Ok(key) => key,
                        Err(e) => {
                            tracing::debug!(index, error = %e, "signature dropped");
                            continue;
                        }
                    },
                    None => continue,
                };
                set(&mut participants, index as usize);
                raw.push((key, signature));
            }
            if trivial_type1(raw.len(), children.len()) {
                continue;
            }
            submit_type1(owner, data, participants, raw, children, &mut events);
        }
    }
    // A union merge started here would still be running when the early block
    // proof is queued two intervals later.
    if !next_is_ours {
        schedule_union_merges(owner, &mut events);
    }
    events
}

/// Type-1 jobs that add no consensus value (leanSpec PR #748 / zeam #907).
///
/// A single raw gossip signature is already on the attestation subnet; a lone
/// child proof is already a valid Type-1. Proving either costs a full STARK.
fn trivial_type1(raw: usize, children: usize) -> bool {
    (children == 0 && raw <= 1) || (raw == 0 && children < 2)
}

/// Interval-2 Type-1 (~1.1 s) would still occupy the prover when the early
/// Type-2 starts. Skip it when this node proposes N+1 and a proved body is
/// already pooled. An empty pool still proves so a lone aggregator can fill
/// the next block.
fn defer_type1_for_early_block(owner: &ChainOwner) -> bool {
    owner
        .last_tick
        .is_some_and(|t| crate::duty_propose::proposes_next_slot(owner, t))
        && owner.aggregates.has_proved_entry()
}

/// Slots back from the current one whose partial aggregates are still merged.
const UNION_WINDOW_SLOTS: u64 = 2;

/// A union merge costs about as much as a Type-2. A second in-flight
/// attestation on the same process (Type-1 or another union) delayed the
/// block proof past the 4 s slot on the 2-aggregator host.
const MAX_IN_FLIGHT_UNIONS: usize = 1;

fn remaining_union_slots(in_flight: usize) -> usize {
    MAX_IN_FLIGHT_UNIONS.saturating_sub(in_flight)
}

/// Aggregators on different subnets each prove only their own votes, so no
/// single pooled proof reaches a supermajority. leanSpec folds those proofs
/// into a child merge while building the block; Ethean keeps that off the
/// proposer's path and merges here, one round after the partial aggregates
/// were gossiped. Only the aggregator owning subnet `slot % committees` runs
/// the merge, so the others do not prove the same union again.
fn schedule_union_merges(owner: &mut ChainOwner, events: &mut Vec<ChainEvent>) {
    let in_flight = owner
        .prover
        .as_ref()
        .map(|p| p.attestations_in_flight_count())
        .unwrap_or(0);
    if remaining_union_slots(in_flight) == 0 {
        tracing::debug!("union merge deferred: an attestation proof is already in flight");
        return;
    }
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
            .unwrap_or(&[])
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
        submit_type1(owner, data, covered, Vec::new(), children, events);
        break;
    }
}

fn submit_type1(
    owner: &mut ChainOwner,
    data: AttestationData,
    participants: Vec<bool>,
    raw: Vec<(ethean_crypto::PublicKey, ethean_crypto::Signature)>,
    children: Vec<BitsAndProof>,
    events: &mut Vec<ChainEvent>,
) {
    let data_root = data.hash_tree_root();
    let children: Vec<KeyedProof> = {
        let Some(state) = owner.head_state.as_ref() else {
            crate::lean_metrics::aggregation_skipped("missing_state");
            return;
        };
        children
            .into_iter()
            .filter_map(|(bits, proof)| {
                Some(KeyedProof {
                    public_keys: attestation_keys_for_bits(state, &bits).ok()?,
                    proof,
                })
            })
            .collect()
    };
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
    insert_proved(owner, &data, &participants, &proof)?;
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
        owner.queue_aggregation_gossip(AggregationGossip {
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
    insert_proved(owner, &data, &participants, &proof)?;
    owner.remember_known_payload(data_root, data.slot.get());
    Ok(ChainEvent::AggregateProved {
        data_root,
        coverage,
        proof_len,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        defer_type1_for_early_block, remaining_union_slots, trivial_type1, MAX_IN_FLIGHT_UNIONS,
    };
    use crate::aggregation::{PoolEntry, PoolKey};
    use crate::chain_owner::ChainOwner;
    use crate::duty_propose::proposes_next_slot;
    use ethean_primitives::{Bytes52, Slot};
    use ethean_profile::lstar_devnet;
    use ethean_types::{BlockHeader, Checkpoint, GenesisConfig, State, Validator};
    use ethean_validator::DutyTick;

    #[test]
    fn skips_jobs_that_cannot_move_quorum() {
        assert!(trivial_type1(0, 0));
        assert!(trivial_type1(1, 0));
        assert!(trivial_type1(0, 1));
        assert!(!trivial_type1(2, 0));
        assert!(!trivial_type1(1, 1));
        assert!(!trivial_type1(0, 2));
    }

    fn owner_three_validators() -> ChainOwner {
        let vals = (0..3)
            .map(|i| {
                Validator::new(
                    Bytes52::ZERO,
                    Bytes52::ZERO,
                    ethean_primitives::ValidatorIndex::new(i),
                )
                .unwrap()
            })
            .collect();
        let mut owner = ChainOwner::new(2);
        owner.head_state = Some(State {
            config: GenesisConfig::new(1_700_000_000),
            slot: Slot::ZERO,
            latest_block_header: BlockHeader::default(),
            latest_justified: Checkpoint::genesis(),
            latest_finalized: Checkpoint::genesis(),
            historical_block_hashes: Vec::new(),
            justified_slots: Vec::new(),
            validators: vals,
            justifications_roots: Vec::new(),
            justifications_validators: Vec::new(),
        });
        owner.profile = Some(lstar_devnet().unwrap());
        owner.last_tick = Some(DutyTick {
            slot: Slot::ZERO,
            interval: 2,
            generation: 1,
        });
        owner
    }

    #[test]
    fn defers_type1_only_when_next_slot_is_ours_and_a_body_is_pooled() {
        let mut owner = owner_three_validators();
        owner.owned_validator_indices = vec![1];
        assert!(proposes_next_slot(&owner, owner.last_tick.unwrap()));
        assert!(!defer_type1_for_early_block(&owner));
        owner.aggregates.insert_verified(
            PoolKey {
                profile_digest: [1u8; 32],
                message_root: [2u8; 32],
            },
            PoolEntry {
                proof: vec![9],
                coverage: 1,
                inserted_slot: 0,
                attestation_ssz: Vec::new(),
            },
        );
        assert!(defer_type1_for_early_block(&owner));
        owner.owned_validator_indices = vec![0];
        assert!(!defer_type1_for_early_block(&owner));
    }

    #[test]
    fn one_in_flight_attestation_uses_the_union_budget() {
        assert_eq!(MAX_IN_FLIGHT_UNIONS, 1);
        assert_eq!(remaining_union_slots(0), 1);
        assert_eq!(remaining_union_slots(1), 0);
        assert_eq!(remaining_union_slots(4), 0);
    }
}
