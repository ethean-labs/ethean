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

/// Apply a compatible remote Status into the sync lag gate.
pub fn observe_remote_status(
    sync: &mut ethean_sync::SyncStatus,
    local_head: Slot,
    remote: &Status,
) {
    sync.observe(local_head, Slot::new(remote.head_slot()));
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
}
