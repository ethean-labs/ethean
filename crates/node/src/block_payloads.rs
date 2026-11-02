//! Post-block payload bookkeeping: seed the known pool with the votes a block
//! carried and, for aggregators, recover each vote's Type-1 proof from the
//! block proof so the next aggregation round can merge it with local
//! partials (leanSpec reaggregation vector `test_post_block_reaggregation`).

use ethean_transition::block_proof_components;
use ethean_types::{Block, Validator};

use crate::chain_owner::ChainOwner;
use crate::events::ChainEvent;
use crate::gossip_attestation::pool_key;
use crate::proof_service::ProofJob;

/// Record the block's attestation data as known payloads and queue proof
/// recovery for the votes this aggregator does not already cover.
pub fn seed_known_payloads(
    owner: &mut ChainOwner,
    block: &Block,
    block_proof: &[u8],
    validators: &[Validator],
) -> Vec<ChainEvent> {
    let mut events = Vec::new();
    for attestation in &block.body.attestations {
        owner.remember_known_payload(
            attestation.data.hash_tree_root(),
            attestation.data.slot.get(),
        );
    }
    if !owner.is_aggregator || owner.prover.is_none() || block_proof.is_empty() {
        return events;
    }
    let Ok(components) = block_proof_components(block, validators) else {
        return events;
    };
    let mut keys: Vec<_> = components.into_iter().map(|c| c.public_keys).collect();
    let mut pending = Vec::new();
    for attestation in &block.body.attestations {
        let data = attestation.data;
        let data_root = data.hash_tree_root();
        let want = attestation
            .aggregation_bits
            .bits
            .iter()
            .filter(|b| **b)
            .count() as u32;
        let covered = owner
            .aggregates
            .best(&pool_key(data_root))
            .is_some_and(|e| e.coverage >= want);
        let in_flight = owner
            .prover
            .as_ref()
            .is_some_and(|p| p.attestation_in_flight(&data_root));
        // Defer Split while a Type-2 block proof is in flight so the critical
        // path is not queued behind reaggregation on a busy host.
        let block_busy = owner.prover.as_ref().is_some_and(|p| p.block_in_flight())
            || owner
                .last_tick
                .is_some_and(|t| crate::duty_propose::prover_reserved_for_block(owner, t));
        if covered || in_flight || block_busy {
            continue;
        }
        pending.push((data, attestation.aggregation_bits.bits.clone(), data_root));
    }
    let last = pending.len().saturating_sub(1);
    for (i, (data, participants, data_root)) in pending.into_iter().enumerate() {
        let public_keys_per_component = if i == last {
            std::mem::take(&mut keys)
        } else {
            keys.clone()
        };
        let job = ProofJob::Split {
            data,
            participants,
            block_proof: block_proof.to_vec(),
            public_keys_per_component,
        };
        if owner.prover.as_mut().is_some_and(|p| p.submit(job)) {
            events.push(ChainEvent::ProofScheduled {
                kind: "split",
                root: data_root,
            });
        }
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Slot, ValidatorIndex, HASH32_ZERO};
    use ethean_types::{
        AggregatedAttestation, AggregationBits, AttestationData, BlockBody, Checkpoint,
    };

    #[test]
    fn records_block_votes_as_known_payloads_without_a_prover() {
        let mut owner = ChainOwner::new(2);
        let data = AttestationData {
            slot: Slot::new(3),
            head: Checkpoint::new([1u8; 32], Slot::new(2)),
            target: Checkpoint::new([1u8; 32], Slot::new(2)),
            source: Checkpoint::new([2u8; 32], Slot::ZERO),
        };
        let att = AggregatedAttestation {
            aggregation_bits: AggregationBits::new(vec![true, false]).unwrap(),
            data,
        };
        let block = Block {
            slot: Slot::new(3),
            proposer_index: ValidatorIndex::new(0),
            parent_root: [1u8; 32],
            state_root: HASH32_ZERO,
            body: BlockBody::new(vec![att]).unwrap(),
        };
        let events = seed_known_payloads(&mut owner, &block, &[1, 2, 3], &[]);
        assert!(events.is_empty());
        assert_eq!(owner.known_payloads.get(&data.hash_tree_root()), Some(&3));
    }
}
