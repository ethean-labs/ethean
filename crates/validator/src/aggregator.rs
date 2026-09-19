//! Aggregator duty: collect assigned subnet coverage and prepare Type-1 work.

use crate::duty_gate::{evaluate_gate, DutyView, SuppressReason};
use crate::scheduler::DutyTick;
use ethean_primitives::Hash32;

/// One attestation-data aggregation assignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregatorPlan {
    /// Tick when aggregation should publish (profile interval).
    pub tick: DutyTick,
    /// Attestation-data root being aggregated.
    pub data_root: Hash32,
    /// Assigned subnet.
    pub subnet: u16,
    /// Observed unique participant coverage count (from pool).
    pub coverage: u32,
    /// Minimum coverage required before dispatching prove work.
    pub min_coverage: u32,
}

/// Aggregator attempt result (prove/publish happen via node aggregation worker).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AggregatorOutcome {
    /// Enough coverage; caller should dispatch Type-1 prove for `data_root`.
    Ready {
        data_root: Hash32,
        subnet: u16,
        coverage: u32,
    },
    /// Not enough disjoint coverage yet.
    InsufficientCoverage { have: u32, need: u32 },
    /// Gate suppressed the duty.
    Suppressed(SuppressReason),
}

/// Evaluate aggregator readiness (no signing here — Type-1 uses the proof worker).
pub fn run_aggregator(view: &DutyView, plan: &AggregatorPlan) -> AggregatorOutcome {
    if let Err(reason) = evaluate_gate(view) {
        return AggregatorOutcome::Suppressed(reason);
    }
    if plan.tick.slot.get() != view.wall_slot.get() {
        return AggregatorOutcome::Suppressed(SuppressReason::HeadLagExceeded);
    }
    if plan.coverage < plan.min_coverage {
        return AggregatorOutcome::InsufficientCoverage {
            have: plan.coverage,
            need: plan.min_coverage,
        };
    }
    AggregatorOutcome::Ready {
        data_root: plan.data_root,
        subnet: plan.subnet,
        coverage: plan.coverage,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::Slot;

    fn view() -> DutyView {
        DutyView {
            wall_slot: Slot::new(2),
            genesis_slot: Slot::new(0),
            syncing: false,
            parent_state_available: true,
            profile_matches: true,
            signer_safe: true,
            head_lag_slots: 0,
            max_head_lag_slots: 2,
        }
    }

    #[test]
    fn ready_when_coverage_met() {
        let plan = AggregatorPlan {
            tick: DutyTick {
                slot: Slot::new(2),
                interval: 3,
                generation: 1,
            },
            data_root: [3u8; 32],
            subnet: 1,
            coverage: 4,
            min_coverage: 2,
        };
        match run_aggregator(&view(), &plan) {
            AggregatorOutcome::Ready { coverage, .. } => assert_eq!(coverage, 4),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn insufficient_coverage() {
        let plan = AggregatorPlan {
            tick: DutyTick {
                slot: Slot::new(2),
                interval: 3,
                generation: 1,
            },
            data_root: [3u8; 32],
            subnet: 1,
            coverage: 1,
            min_coverage: 3,
        };
        assert!(matches!(
            run_aggregator(&view(), &plan),
            AggregatorOutcome::InsufficientCoverage { have: 1, need: 3 }
        ));
    }
}
