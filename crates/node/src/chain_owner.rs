//! Chain owner: sole writer of transition, fork choice, and import status.

use crate::aggregation::AggregatePool;
use crate::block_builder::PlanTransition;
use ethean_primitives::{Hash32, Slot};
use ethean_profile::ChainProfile;
use ethean_types::State;
use ethean_validator::{DutyTick, DutyView};

/// Immutable worker snapshot published by the chain owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainSnapshot {
    /// Scheduler generation at snapshot time.
    pub generation: u64,
    /// Wall slot.
    pub wall_slot: Slot,
    /// Canonical head root.
    pub head_root: Hash32,
    /// Parent root for the next proposal (usually head).
    pub parent_root: Hash32,
    /// Safe target root from fork choice.
    pub safe_target: Hash32,
    /// Justified checkpoint root.
    pub justified_root: Hash32,
    /// Finalized checkpoint root.
    pub finalized_root: Hash32,
    /// Duty view for the gate.
    pub duty_view: DutyView,
}

/// Owns chain mutation; workers only read snapshots / send commands.
#[derive(Debug, Default)]
pub struct ChainOwner {
    /// Current generation for stale rejection.
    pub generation: u64,
    /// Last applied tick.
    pub last_tick: Option<DutyTick>,
    /// Latest head root.
    pub head_root: Hash32,
    /// Post-state of head (optional until storage wiring).
    pub head_state: Option<State>,
    /// Syncing flag from sync subsystem.
    pub syncing: bool,
    /// Last ingested gossip content root (provisional until SSZ import).
    pub last_gossip_root: Option<Hash32>,
    /// Chain profile for structural gossip state transitions.
    pub profile: Option<ChainProfile>,
    /// In-memory aggregate proofs from attestation / aggregation gossip.
    pub aggregates: AggregatePool,
    /// Last locally planned proposal (cleared when superseded).
    pub planned_proposal: Option<PlanTransition>,
    /// Tick that produced `planned_proposal`, if any.
    pub planned_tick: Option<DutyTick>,
    /// Configured max head lag.
    pub max_head_lag_slots: u64,
}

impl ChainOwner {
    /// Construct with lag budget.
    pub fn new(max_head_lag_slots: u64) -> Self {
        Self {
            max_head_lag_slots,
            ..Self::default()
        }
    }

    /// Apply a clock tick; returns false when duplicate.
    pub fn on_tick(&mut self, tick: DutyTick) -> bool {
        if !ethean_validator::should_process(self.last_tick, tick) {
            return false;
        }
        self.last_tick = Some(tick);
        true
    }

    /// Bump generation after restart / catch-up.
    pub fn bump_generation(&mut self) {
        self.generation = self.generation.saturating_add(1);
    }

    /// Build a snapshot for workers (stale if generation mismatches later).
    pub fn snapshot(&self, wall_slot: Slot, head_lag_slots: u64) -> ChainSnapshot {
        let duty_view = DutyView {
            wall_slot,
            genesis_slot: Slot::new(0),
            syncing: self.syncing,
            parent_state_available: self.head_state.is_some() || wall_slot.get() == 0,
            profile_matches: true,
            signer_safe: true,
            head_lag_slots,
            max_head_lag_slots: self.max_head_lag_slots,
        };
        ChainSnapshot {
            generation: self.generation,
            wall_slot,
            head_root: self.head_root,
            parent_root: self.head_root,
            safe_target: self.head_root,
            justified_root: self.head_root,
            finalized_root: self.head_root,
            duty_view,
        }
    }

    /// Reject a worker result when generation or parent drifted.
    pub fn accept_result(&self, generation: u64, parent_root: Hash32) -> bool {
        generation == self.generation && parent_root == self.head_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedupes_ticks() {
        let mut owner = ChainOwner::new(2);
        let tick = DutyTick {
            slot: Slot::new(1),
            interval: 0,
            generation: 1,
        };
        assert!(owner.on_tick(tick));
        assert!(!owner.on_tick(tick));
    }

    #[test]
    fn rejects_stale_generation() {
        let mut owner = ChainOwner::new(2);
        owner.head_root = [9u8; 32];
        owner.bump_generation();
        assert!(!owner.accept_result(0, [9u8; 32]));
        assert!(owner.accept_result(1, [9u8; 32]));
    }
}
