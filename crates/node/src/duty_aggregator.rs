//! Aggregator duty evaluation against the in-memory aggregate pool.

use crate::chain_owner::ChainOwner;
use crate::events::ChainEvent;
use ethean_network::SMOKE_ATTESTATION_SUBNETS;
use ethean_primitives::Hash32;
use ethean_validator::{run_aggregator, AggregatorOutcome, AggregatorPlan, DutyTick};

/// Minimum pool coverage before emitting [`ChainEvent::AggregatorReady`].
const MIN_AGGREGATOR_COVERAGE: u32 = 1;

/// Provisional subnet from attestation-data root until committee mapping lands.
fn provisional_subnet(message_root: Hash32, subnet_count: u16) -> u16 {
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&message_root[0..8]);
    (u64::from_be_bytes(buf) % u64::from(subnet_count.max(1))) as u16
}

/// Evaluate aggregator readiness for each retained pool key on this tick.
pub fn evaluate_aggregator_duties(
    owner: &ChainOwner,
    tick: DutyTick,
    head_lag_slots: u64,
) -> Vec<ChainEvent> {
    if !owner.is_aggregator {
        return Vec::new();
    }
    let view = owner.snapshot(tick.slot, head_lag_slots).duty_view;
    let mut out = Vec::new();
    for (key, entry) in owner.aggregates.best_entries() {
        let plan = AggregatorPlan {
            tick,
            data_root: key.message_root,
            subnet: provisional_subnet(key.message_root, SMOKE_ATTESTATION_SUBNETS),
            coverage: entry.coverage.max(1),
            min_coverage: MIN_AGGREGATOR_COVERAGE,
        };
        match run_aggregator(&view, &plan) {
            AggregatorOutcome::Ready {
                data_root,
                subnet,
                coverage,
            } => out.push(ChainEvent::AggregatorReady {
                data_root,
                subnet,
                coverage,
            }),
            AggregatorOutcome::InsufficientCoverage { .. }
            | AggregatorOutcome::Suppressed(_) => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregation::{PoolEntry, PoolKey};
    use crate::chain_owner::ChainOwner;
    use ethean_primitives::Slot;
    use ethean_validator::DutyTick;

    #[test]
    fn emits_ready_when_aggregator_and_pool_nonempty() {
        let mut owner = ChainOwner::default();
        owner.is_aggregator = true;
        let key = PoolKey {
            profile_digest: [9u8; 32],
            message_root: [1u8; 32],
        };
        owner.aggregates.insert_verified(
            key,
            PoolEntry {
                proof: vec![1, 2, 3],
                coverage: 2,
                inserted_slot: 0,
                attestation_ssz: Vec::new(),
            },
        );
        let tick = DutyTick {
            slot: Slot::new(0),
            interval: 1,
            generation: 1,
        };
        let events = evaluate_aggregator_duties(&owner, tick, 0);
        assert!(matches!(
            events.as_slice(),
            [ChainEvent::AggregatorReady { coverage: 2, .. }]
        ));
        owner.is_aggregator = false;
        assert!(evaluate_aggregator_duties(&owner, tick, 0).is_empty());
    }
}
