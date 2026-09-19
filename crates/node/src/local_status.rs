//! Build local Lean Status from chain owner state.

use crate::chain_owner::ChainOwner;
use ethean_network_wire::Status;
use ethean_primitives::{Hash32, Slot};

/// Construct the Status we advertise / compare during handshakes.
pub fn local_status(
    owner: &ChainOwner,
    genesis_root: Hash32,
    fork_segment: &str,
) -> Status {
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
        .unwrap_or((0, Hash32::default()));
    Status {
        genesis_root,
        fork_segment: fork_segment.to_string(),
        head_slot,
        head_root: owner.head_root,
        finalized_slot,
        finalized_root,
    }
}

/// Apply a compatible remote Status into the sync lag gate.
pub fn observe_remote_status(
    sync: &mut ethean_sync::SyncStatus,
    local_head: Slot,
    remote: &Status,
) {
    sync.observe(local_head, Slot::new(remote.head_slot));
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_sync::SyncStatus;

    #[test]
    fn builds_zero_head_status() {
        let owner = ChainOwner::new(2);
        let s = local_status(&owner, [9u8; 32], "aabbccdd");
        assert_eq!(s.genesis_root, [9u8; 32]);
        assert_eq!(s.fork_segment, "aabbccdd");
        assert_eq!(s.head_slot, 0);
    }

    #[test]
    fn observe_moves_peer_horizon() {
        let mut sync = SyncStatus::new(Slot::new(0), Slot::new(0));
        let remote = Status {
            genesis_root: [0u8; 32],
            fork_segment: "aabbccdd".into(),
            head_slot: 12,
            head_root: [1u8; 32],
            finalized_slot: 0,
            finalized_root: [0u8; 32],
        };
        observe_remote_status(&mut sync, Slot::new(1), &remote);
        assert_eq!(sync.peer_horizon.get(), 12);
        assert_eq!(sync.lag(), 11);
    }
}
