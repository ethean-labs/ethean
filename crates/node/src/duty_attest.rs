//! Sign a local attestation for owned registry indices and seed the aggregate pool.

use crate::aggregation::{PoolEntry, PoolKey};
use crate::chain_owner::ChainOwner;
use crate::events::ChainEvent;
use crate::gossip_pool::pool_profile_digest;
use ethean_primitives::ValidatorIndex;
use ethean_types::{AggregatedAttestation, AggregationBits, AttestationData, Checkpoint};
use ethean_validator::DutyTick;

/// Interval used for attestation duties (proposal uses 0).
pub const ATTESTATION_INTERVAL: u8 = 1;

/// When a local attester is installed, sign head attestation data for the first
/// owned validator index and insert a single-bit aggregate into the pool.
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
    let target = if head.slot > source.slot { head } else { source };
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

    let mut bits = vec![false; n];
    bits[index as usize] = true;
    let Ok(agg_bits) = AggregationBits::new(bits) else {
        return out;
    };
    let agg = AggregatedAttestation {
        aggregation_bits: agg_bits,
        data,
    };
    let attestation_ssz = agg.ssz_encode();
    let key = PoolKey {
        profile_digest: pool_profile_digest(),
        message_root: data_root,
    };
    owner.aggregates.insert_verified(
        key,
        PoolEntry {
            // Empty until leanVM / test-aggregate Type-1 prove fills a real proof.
            // Never stuff individual XMSS signatures into the aggregate proof field.
            proof: Vec::new(),
            coverage: 1,
            inserted_slot: tick.slot.get(),
            attestation_ssz,
        },
    );
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
    fn signs_and_seeds_pool_on_interval_1() {
        let mut owner = ChainOwner::new(4);
        owner.head_state = Some(state_n(4));
        owner.head_root = [7u8; 32];
        owner.attester = Some(LocalAttester::smoke().unwrap());
        owner.owned_validator_indices = vec![2];
        let tick = DutyTick {
            slot: Slot::new(3),
            interval: 1,
            generation: 1,
        };
        let ev = try_local_attest(&mut owner, tick);
        assert_eq!(ev.len(), 1);
        assert!(!owner.aggregates.is_empty());
        let key = owner.aggregates.best_entries()[0].0;
        assert!(
            owner.aggregates.best(&key).unwrap().proof.is_empty(),
            "XMSS must not stand in as Type-1 proof"
        );
        let _ = HASH32_ZERO;
    }
}
