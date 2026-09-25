//! Build local Lean Status from chain owner state.

use crate::chain_owner::ChainOwner;
use ethean_network_wire::{Checkpoint, Status};
use ethean_primitives::Slot;

/// Construct the Status we advertise / compare during handshakes.
pub fn local_status(owner: &ChainOwner) -> Status {
    let head_slot = owner
        .head_state
        .as_ref()
        .map(|s| s.slot.get())
        .or_else(|| owner.last_tick.map(|t| t.slot.get()))
        .unwrap_or(0);
    let (finalized_slot, finalized_root) = owner
        .head_state
        .as_ref()
        .map(|s| (s.latest_finalized.slot.get(), s.latest_finalized.root))
        .unwrap_or((0, Default::default()));
    Status {
        finalized: Checkpoint {
            root: finalized_root,
            slot: finalized_slot,
        },
        head: Checkpoint {
            root: owner.head_root,
            slot: head_slot,
        },
    }
}

/// Apply a single remote Status into the sync lag gate (tests / sole peer).
///
/// Live multi-peer catch-up uses
/// [`crate::sync_catchup::apply_preferred_horizon`] so a lone ahead tip cannot
/// pin the horizon above the agreeing majority.
pub fn observe_remote_status(
    sync: &mut ethean_sync::SyncStatus,
    local_head: Slot,
    remote: &Status,
) {
    let horizon = remote.head_slot().max(remote.finalized_slot());
    sync.observe(local_head, Slot::new(horizon));
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_sync::SyncStatus;

    #[test]
    fn builds_zero_head_status() {
        let owner = ChainOwner::new(2);
        let s = local_status(&owner);
        assert_eq!(s.head_slot(), 0);
        assert_eq!(s.finalized_slot(), 0);
    }

    #[test]
    fn observe_moves_peer_horizon() {
        let mut sync = SyncStatus::new(Slot::new(0), Slot::new(0));
        let remote = Status {
            finalized: Checkpoint {
                root: [0u8; 32],
                slot: 0,
            },
            head: Checkpoint {
                root: [1u8; 32],
                slot: 12,
            },
        };
        observe_remote_status(&mut sync, Slot::new(1), &remote);
        assert_eq!(sync.peer_horizon.get(), 12);
        assert_eq!(sync.lag(), 11);
    }

    #[test]
    fn observe_uses_finalized_when_ahead_of_head() {
        let mut sync = SyncStatus::new(Slot::new(0), Slot::new(0));
        let remote = Status {
            finalized: Checkpoint {
                root: [2u8; 32],
                slot: 40,
            },
            head: Checkpoint {
                root: [1u8; 32],
                slot: 10,
            },
        };
        observe_remote_status(&mut sync, Slot::new(0), &remote);
        assert_eq!(sync.peer_horizon.get(), 40);
        assert_eq!(sync.lag(), 40);
    }
}
