//! Parent selection for block building from a chain snapshot.

use crate::chain_owner::ChainSnapshot;
use ethean_primitives::Hash32;

/// Select the parent root for a proposal from the owner snapshot.
pub fn select_parent(snapshot: &ChainSnapshot) -> Hash32 {
    snapshot.parent_root
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::Slot;
    use ethean_validator::DutyView;

    #[test]
    fn uses_snapshot_parent() {
        let snap = ChainSnapshot {
            generation: 1,
            wall_slot: Slot::new(4),
            head_root: [1u8; 32],
            parent_root: [2u8; 32],
            safe_target: [1u8; 32],
            justified_root: [1u8; 32],
            finalized_root: [1u8; 32],
            duty_view: DutyView {
                wall_slot: Slot::new(4),
                genesis_slot: Slot::new(0),
                syncing: false,
                parent_state_available: true,
                profile_matches: true,
                signer_safe: true,
                head_lag_slots: 0,
                max_head_lag_slots: 2,
            },
        };
        assert_eq!(select_parent(&snap), [2u8; 32]);
    }
}
