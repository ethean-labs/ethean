//! Sign a local attestation for an owned validator, publish it as a
//! `SignedAttestation` on its subnet, and keep it for aggregation.

use crate::aggregation_gossip::AggregationGossip;
use crate::chain_owner::ChainOwner;
use crate::events::ChainEvent;
use ethean_crypto::Signature;
use ethean_network_wire::{fork_segment_from_name, topic_attestation};
use ethean_primitives::ValidatorIndex;
use ethean_types::{AttestationData, Checkpoint, SignedAttestation};
use ethean_validator::DutyTick;

/// Interval used for attestation duties (proposal uses 0).
pub const ATTESTATION_INTERVAL: u8 = 1;

/// When a local attester is installed, sign head attestation data for the first
/// owned validator index, queue it for subnet gossip, and pool the signature
/// when this node aggregates.
pub fn try_local_attest(owner: &mut ChainOwner, tick: DutyTick) -> Vec<ChainEvent> {
    let mut out = Vec::new();
    if tick.interval != ATTESTATION_INTERVAL {
        return out;
    }
    if owner.attester.is_none() {
        return out;
    }
    let Some(index) = owner.owned_validator_indices.first().copied() else {
        return out;
    };
    let Some(state) = owner.head_state.as_ref() else {
        return out;
    };
    let n = state.validators.len();
    if n == 0 || (index as usize) >= n {
        return out;
    }

    let head = Checkpoint {
        root: owner.head_root,
        slot: state.slot,
    };
    let source = state.latest_justified;
    let target = if head.slot > source.slot {
        head
    } else {
        source
    };
    let data = AttestationData {
        slot: tick.slot,
        head,
        target,
        source,
    };
    let data_root = data.hash_tree_root();
    let committees = owner
        .profile
        .as_ref()
        .map(|p| p.attestation_committee_count.max(1))
        .unwrap_or(1);
    let subnet = (index % committees) as u16;
    let lag = 0u64;
    let duty_view = owner.snapshot(tick.slot, lag).duty_view;

    let sig = {
        let Some(attester) = owner.attester.as_mut() else {
            return out;
        };
        match attester.sign_attestation(tick, &duty_view, data_root, subnet) {
            Ok(s) => s,
            Err(e) => {
                tracing::debug!(error = %e, index, "local attestation sign skipped");
                return out;
            }
        }
    };

    if let Some(attester) = owner.attester.as_ref() {
        if let Err(e) = attester.verify_attestation(tick, data_root, &sig) {
            tracing::warn!(error = %e, "local attestation binding verify failed");
            return out;
        }
    }

    let Ok(signature) = Signature::try_from_slice(&sig) else {
        return out;
    };
    let Ok(vote) = SignedAttestation::new(ValidatorIndex::new(index), data, sig.clone()) else {
        return out;
    };
    if owner.is_aggregator {
        owner.signatures.insert(data_root, &data, index, signature);
    }
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
    out.push(ChainEvent::AttestationSigned {
        data_root,
        validator_index: ValidatorIndex::new(index),
        signature_len: sig.len(),
        subnet,
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_attester::LocalAttester;
    use ethean_primitives::{Bytes52, Slot, HASH32_ZERO};
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
        let _ = HASH32_ZERO;
    }
}
