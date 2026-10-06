//! Sign a local attestation for an owned validator, publish it as a
//! `SignedAttestation` on its subnet, and keep it for aggregation.

use crate::aggregation_gossip::AggregationGossip;
use crate::chain_owner::ChainOwner;
use crate::events::ChainEvent;
use ethean_crypto::Signature;
use ethean_metrics::lean::{inc, observe_since};
use ethean_network_wire::{fork_segment_from_name, topic_attestation};
use ethean_primitives::{ValidatorIndex, HASH32_ZERO};
use ethean_types::{AttestationData, Checkpoint, SignedAttestation};
use ethean_validator::DutyTick;
use std::time::Instant;

/// Interval used for attestation duties (proposal uses 0).
pub const ATTESTATION_INTERVAL: u8 = 1;

/// Sign one attestation per owned validator with a local attester (head = FC
/// tip; target = safe target when live), queue each for subnet gossip, and
/// pool the signatures when this node aggregates.
pub fn try_local_attest(owner: &mut ChainOwner, tick: DutyTick) -> Vec<ChainEvent> {
    let mut out = Vec::new();
    if tick.interval != ATTESTATION_INTERVAL {
        return out;
    }
    let production_started = Instant::now();
    let indices = owner.attesting_indices();
    if indices.is_empty() {
        return out;
    }
    let Some(state) = owner.head_state.as_ref() else {
        return out;
    };
    let n = state.validators.len() as u64;
    let head = Checkpoint {
        root: owner.head_root,
        slot: state.slot,
    };
    let Some((source, target)) = vote_checkpoints(owner, head) else {
        return out;
    };
    let data = AttestationData {
        slot: tick.slot,
        head,
        target,
        source,
    };
    for index in indices.into_iter().filter(|i| *i < n) {
        if let Some(event) = attest_as(owner, tick, index, data) {
            out.push(event);
        }
    }
    if !out.is_empty() {
        observe_since(
            "lean_attestations_production_time_seconds",
            &[],
            production_started,
        );
    }
    out
}

fn attest_as(
    owner: &mut ChainOwner,
    tick: DutyTick,
    index: u64,
    data: AttestationData,
) -> Option<ChainEvent> {
    let data_root = data.hash_tree_root();
    let subnet = (index % owner.attestation_committees()) as u16;
    let duty_view = owner.snapshot(tick.slot, 0).duty_view;
    let attester = owner.attester_for(index)?;
    let signing_started = Instant::now();
    let sig = match attester.sign_attestation(tick, &duty_view, data_root, subnet) {
        Ok(s) => {
            observe_since(
                "lean_pq_sig_attestation_signing_time_seconds",
                &[],
                signing_started,
            );
            inc("lean_pq_sig_attestation_signatures_total", &[], 1.0);
            s
        }
        Err(e) => {
            tracing::debug!(error = %e, index, "local attestation sign skipped");
            return None;
        }
    };
    if let Err(e) = attester.verify_attestation(tick, data_root, &sig) {
        tracing::warn!(error = %e, index, "local attestation binding verify failed");
        return None;
    }

    let signature = Signature::try_from_slice(&sig).ok()?;
    let vote = SignedAttestation::new(ValidatorIndex::new(index), data, sig.clone()).ok()?;
    if owner.is_aggregator && owner.aggregates_vote_of(index) {
        owner.signatures.insert(data_root, &data, index, signature);
    }
    owner.fc_on_attestation(ValidatorIndex::new(index), data);
    if let Some(fork) = owner.profile.as_ref().map(|p| p.fork_name) {
        let topic = fork_segment_from_name(fork)
            .ok()
            .and_then(|segment| topic_attestation(&segment, subnet).ok());
        if let Some(topic) = topic {
            owner.pending_aggregation_gossip.push(AggregationGossip {
                topic,
                payload: vote.ssz_encode(),
                data_root,
                proof_len: 0,
            });
        }
    }
    Some(ChainEvent::AttestationSigned {
        data_root,
        validator_index: ValidatorIndex::new(index),
        signature_len: sig.len(),
        subnet,
    })
}

/// Source and target for a local vote on `head`.
///
/// With a live store this is leanSpec `produce_attestation_data`: the head
/// state's justified checkpoint and `get_attestation_target` (lookback toward
/// the safe target, then back to a slot justifiable after finalization).
fn vote_checkpoints(owner: &ChainOwner, head: Checkpoint) -> Option<(Checkpoint, Checkpoint)> {
    let state = owner.head_state.as_ref()?;
    if let (Some(fc), Some(profile)) = (owner.fc.as_ref(), owner.profile.as_ref()) {
        let source = fc.attestation_source()?;
        let target = fc.attestation_target(profile.justification_lookback_slots);
        if target.slot < source.slot {
            tracing::debug!(
                source = source.slot.get(),
                target = target.slot.get(),
                "attestation skipped: target behind the head's justified source"
            );
            return None;
        }
        return Some((source, target));
    }
    let source = state.latest_justified;
    let target = if owner.safe_target != HASH32_ZERO {
        Checkpoint {
            root: owner.safe_target,
            slot: ethean_primitives::Slot::new(owner.safe_target_slot()),
        }
    } else if head.slot > source.slot {
        head
    } else {
        source
    };
    Some((source, target))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_attester::LocalAttester;
    use ethean_primitives::{Bytes52, Slot, ValidatorIndex};
    use ethean_types::{BlockHeader, GenesisConfig, State, Validator};

    fn state_n(n: usize) -> State {
        let mut vals = Vec::new();
        for i in 0..n {
            vals.push(
                Validator::new(Bytes52::ZERO, Bytes52::ZERO, ValidatorIndex::new(i as u64))
                    .unwrap(),
            );
        }
        State {
            config: GenesisConfig::new(1_700_000_000),
            slot: Slot::new(3),
            latest_block_header: BlockHeader {
                slot: Slot::new(3),
                ..Default::default()
            },
            latest_justified: Checkpoint::genesis(),
            latest_finalized: Checkpoint::genesis(),
            historical_block_hashes: Vec::new(),
            justified_slots: Vec::new(),
            validators: vals,
            justifications_roots: Vec::new(),
            justifications_validators: Vec::new(),
        }
    }

    #[test]
    fn signs_publishes_and_pools_own_vote() {
        let mut owner = ChainOwner::new(4);
        owner.head_state = Some(state_n(4));
        owner.head_root = [7u8; 32];
        owner.profile = Some(ethean_profile::lstar_devnet().unwrap());
        owner.attester = Some(LocalAttester::smoke().unwrap());
        owner.owned_validator_indices = vec![2];
        owner.is_aggregator = true;
        let tick = DutyTick {
            slot: Slot::new(3),
            interval: 1,
            generation: 1,
        };
        let ev = try_local_attest(&mut owner, tick);
        assert_eq!(ev.len(), 1);
        assert!(
            owner.aggregates.is_empty(),
            "a vote is not an aggregate proof"
        );
        assert_eq!(owner.signatures.len(), 1, "aggregator keeps its own vote");
        let gossip = &owner.pending_aggregation_gossip[0];
        assert!(gossip.topic.contains("/attestation_"), "{}", gossip.topic);
        let vote = SignedAttestation::ssz_decode(&gossip.payload).unwrap();
        assert_eq!(vote.validator_index.get(), 2);
        assert_eq!(vote.data.hash_tree_root(), gossip.data_root);
    }

    #[test]
    fn signs_once_per_validator_with_its_own_key() {
        use crate::chain_owner::ValidatorSigners;
        let mut owner = ChainOwner::new(4);
        owner.head_state = Some(state_n(4));
        owner.head_root = [7u8; 32];
        owner.profile = Some(ethean_profile::lstar_devnet().unwrap());
        owner.owned_validator_indices = vec![0, 2, 3];
        for index in [0, 3] {
            owner.signers.insert(
                index,
                ValidatorSigners {
                    attester: Some(LocalAttester::smoke().unwrap()),
                    proposer: None,
                },
            );
        }
        owner.signers.insert(2, ValidatorSigners::default());
        let tick = DutyTick {
            slot: Slot::new(3),
            interval: 1,
            generation: 1,
        };
        assert_eq!(try_local_attest(&mut owner, tick).len(), 2);
        let voters: Vec<u64> = owner
            .pending_aggregation_gossip
            .iter()
            .map(|g| {
                SignedAttestation::ssz_decode(&g.payload)
                    .unwrap()
                    .validator_index
                    .get()
            })
            .collect();
        assert_eq!(voters, vec![0, 3]);
    }

    #[test]
    fn live_store_votes_carry_the_real_genesis_root_as_source() {
        let mut genesis = state_n(4);
        genesis.slot = Slot::ZERO;
        genesis.latest_block_header = BlockHeader {
            body_root: ethean_types::BlockBody::default().hash_tree_root().unwrap(),
            ..Default::default()
        };
        let mut owner = ChainOwner::new(4);
        owner.head_state = Some(genesis);
        owner.profile = Some(ethean_profile::lstar_devnet().unwrap());
        crate::local_finality::seal_genesis_head(&mut owner);
        owner.try_init_fork_choice();
        assert!(owner.fc.is_some());
        owner.attester = Some(LocalAttester::smoke().unwrap());
        owner.owned_validator_indices = vec![1];
        let tick = DutyTick {
            slot: Slot::new(1),
            interval: 1,
            generation: 1,
        };
        assert_eq!(try_local_attest(&mut owner, tick).len(), 1);
        let vote =
            SignedAttestation::ssz_decode(&owner.pending_aggregation_gossip[0].payload).unwrap();
        assert_eq!(vote.data.source.root, owner.head_root);
        assert_eq!(vote.data.target.root, owner.head_root);
        assert_eq!(vote.data.head.root, owner.head_root);
    }

    #[test]
    fn attests_to_safe_target_when_set() {
        let mut owner = ChainOwner::new(4);
        owner.head_state = Some(state_n(4));
        owner.head_root = [7u8; 32];
        owner.safe_target = [9u8; 32];
        owner.profile = Some(ethean_profile::lstar_devnet().unwrap());
        owner.attester = Some(LocalAttester::smoke().unwrap());
        owner.owned_validator_indices = vec![1];
        let tick = DutyTick {
            slot: Slot::new(3),
            interval: 1,
            generation: 1,
        };
        let ev = try_local_attest(&mut owner, tick);
        assert_eq!(ev.len(), 1);
        let vote =
            SignedAttestation::ssz_decode(&owner.pending_aggregation_gossip[0].payload).unwrap();
        assert_eq!(vote.data.head.root, [7u8; 32]);
        assert_eq!(vote.data.target.root, [9u8; 32]);
    }
}
