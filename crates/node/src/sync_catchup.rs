//! Remember peer Status tips and stage follow-up block sync while catching up.

use ethean_network::{
    prepare_blocks_by_range_outbound, prepare_blocks_by_root_outbound, OutboundBlocksByRangeRequest,
    OutboundBlocksByRootRequest, RequestTracker,
};
use ethean_network_wire::Status;
use ethean_primitives::Hash32;

use crate::status_handshake::RANGE_PREFER_LAG_SLOTS;

/// Last remote Status tip we are trying to catch up to for one peer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerSyncTarget {
    pub peer: Hash32,
    pub remote: Status,
}

/// Upsert a peer tip; keep the higher head slot when both exist.
pub fn remember_target(targets: &mut Vec<PeerSyncTarget>, peer: Hash32, remote: Status) {
    if let Some(existing) = targets.iter_mut().find(|t| t.peer == peer) {
        if remote.head_slot() >= existing.remote.head_slot() {
            existing.remote = remote;
        }
        return;
    }
    targets.push(PeerSyncTarget { peer, remote });
}

/// Drop targets whose head is no longer ahead of the local chain.
pub fn prune_caught_up(targets: &mut Vec<PeerSyncTarget>, local_head_slot: u64) {
    targets.retain(|t| t.remote.head_slot() > local_head_slot);
}

/// Follow-up block sync requests after a Status batch (or empty response).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CatchupOutbounds {
    pub blocks_by_root: Option<OutboundBlocksByRootRequest>,
    pub blocks_by_range: Option<OutboundBlocksByRangeRequest>,
}

/// Stage the next range/root fetch for the first peer still ahead of local head.
pub fn prepare_follow_up(
    targets: &[PeerSyncTarget],
    local_head_root: Hash32,
    local_head_slot: u64,
    tracker: &mut RequestTracker,
) -> Result<CatchupOutbounds, String> {
    let Some(target) = targets
        .iter()
        .find(|t| t.remote.head_slot() > local_head_slot)
    else {
        return Ok(CatchupOutbounds::default());
    };
    let lag = target.remote.head_slot().saturating_sub(local_head_slot);
    if lag >= RANGE_PREFER_LAG_SLOTS {
        Ok(CatchupOutbounds {
            blocks_by_root: None,
            blocks_by_range: prepare_blocks_by_range_outbound(
                target.peer,
                local_head_slot,
                &target.remote,
                tracker,
            )
            .map_err(|e| e.to_string())?,
        })
    } else {
        Ok(CatchupOutbounds {
            blocks_by_root: prepare_blocks_by_root_outbound(
                target.peer,
                local_head_root,
                &target.remote,
                tracker,
            )
            .map_err(|e| e.to_string())?,
            blocks_by_range: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_network_wire::Checkpoint;

    fn tip(slot: u64) -> Status {
        Status {
            finalized: Checkpoint {
                root: [0u8; 32],
                slot: 0,
            },
            head: Checkpoint {
                root: [9u8; 32],
                slot,
            },
        }
    }

    #[test]
    fn remember_keeps_higher_head() {
        let mut targets = Vec::new();
        let peer = [1u8; 32];
        remember_target(&mut targets, peer, tip(10));
        remember_target(&mut targets, peer, tip(8));
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].remote.head_slot(), 10);
        remember_target(&mut targets, peer, tip(12));
        assert_eq!(targets[0].remote.head_slot(), 12);
    }

    #[test]
    fn prune_removes_caught_up() {
        let mut targets = vec![PeerSyncTarget {
            peer: [1u8; 32],
            remote: tip(5),
        }];
        prune_caught_up(&mut targets, 5);
        assert!(targets.is_empty());
    }

    #[test]
    fn follow_up_stages_range_when_deep() {
        let targets = vec![PeerSyncTarget {
            peer: [2u8; 32],
            remote: tip(20),
        }];
        let mut tracker = RequestTracker::default();
        let out = prepare_follow_up(&targets, [0u8; 32], 3, &mut tracker)
            .unwrap();
        assert!(out.blocks_by_range.is_some());
        assert!(out.blocks_by_root.is_none());
    }
}
