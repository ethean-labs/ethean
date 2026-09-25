//! Remember peer Status tips and stage follow-up block sync while catching up.
//!
//! When several peers advertise tips, catch-up follows the **majority** finalized
//! checkpoint (Hive “rejects ahead bad checkpoint”: two honest helpers beat one
//! artificially ahead adversarial peer). A singleton tip is only chased when no
//! finalized group has two or more votes.

use ethean_network::{
    prepare_blocks_by_range_outbound, prepare_blocks_by_root_for_roots,
    prepare_blocks_by_root_outbound, OutboundBlocksByRangeRequest, OutboundBlocksByRootRequest,
    RequestTracker,
};
use ethean_network_wire::Status;
use ethean_primitives::{Hash32, Slot, HASH32_ZERO};
use ethean_sync::SyncStatus;
use std::collections::HashMap;

use crate::status_handshake::RANGE_PREFER_LAG_SLOTS;

/// Last remote Status tip we are trying to catch up to for one peer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerSyncTarget {
    pub peer: Hash32,
    pub remote: Status,
}

/// Upsert a peer tip; keep the higher head / finalized checkpoints.
pub fn remember_target(targets: &mut Vec<PeerSyncTarget>, peer: Hash32, remote: Status) {
    if let Some(existing) = targets.iter_mut().find(|t| t.peer == peer) {
        let head = if remote.head_slot() >= existing.remote.head_slot() {
            remote.head
        } else {
            existing.remote.head
        };
        let finalized = if remote.finalized_slot() >= existing.remote.finalized_slot() {
            remote.finalized
        } else {
            existing.remote.finalized
        };
        existing.remote = Status { finalized, head };
        return;
    }
    targets.push(PeerSyncTarget { peer, remote });
}

/// Drop targets that are no longer ahead on head **or** finalized.
pub fn prune_caught_up(targets: &mut Vec<PeerSyncTarget>, local_head_slot: u64) {
    targets.retain(|t| {
        t.remote.head_slot() > local_head_slot || t.remote.finalized_slot() > local_head_slot
    });
}

/// Follow-up block sync requests after a Status batch (or empty response).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CatchupOutbounds {
    pub blocks_by_root: Option<OutboundBlocksByRootRequest>,
    pub blocks_by_range: Option<OutboundBlocksByRangeRequest>,
}

fn behind_finalized(remote: &Status, local_head_slot: u64) -> bool {
    remote.finalized_slot() > local_head_slot
}

/// Roots that unblock head-behind-finalized recovery (finalized tip + head).
fn recovery_roots(remote: &Status, local_head_root: Hash32) -> Vec<Hash32> {
    let mut roots = Vec::new();
    let fz = remote.finalized_root();
    if fz != HASH32_ZERO {
        roots.push(fz);
    }
    let head = remote.head_root();
    if head != HASH32_ZERO && head != local_head_root && head != fz {
        roots.push(head);
    }
    roots
}

fn still_ahead(remote: &Status, local_head_slot: u64) -> bool {
    remote.head_slot() > local_head_slot || remote.finalized_slot() > local_head_slot
}

/// Finalized tip with the strongest peer agreement.
///
/// Prefers any `(root, slot)` with **≥2** votes (highest slot, then vote count).
/// With a single remembered peer, that peer's finalized tip wins. With a 1–1
/// disagreement and no multi-vote group, returns `None` so we do not chase a
/// lone adversarial tip.
pub fn preferred_finalized(targets: &[PeerSyncTarget]) -> Option<(Hash32, u64)> {
    if targets.is_empty() {
        return None;
    }
    if targets.len() == 1 {
        let t = &targets[0];
        return Some((t.remote.finalized_root(), t.remote.finalized_slot()));
    }
    let mut counts: HashMap<(Hash32, u64), usize> = HashMap::new();
    for t in targets {
        *counts
            .entry((t.remote.finalized_root(), t.remote.finalized_slot()))
            .or_default() += 1;
    }
    counts
        .into_iter()
        .filter(|(_, n)| *n >= 2)
        .max_by_key(|((_, slot), n)| (*n, *slot))
        .map(|((root, slot), _)| (root, slot))
}

/// Peer to fetch from: majority finalized tip that is still ahead of local head.
pub fn select_catchup_target(
    targets: &[PeerSyncTarget],
    local_head_slot: u64,
) -> Option<&PeerSyncTarget> {
    let (root, slot) = preferred_finalized(targets)?;
    targets.iter().find(|t| {
        still_ahead(&t.remote, local_head_slot)
            && t.remote.finalized_root() == root
            && t.remote.finalized_slot() == slot
    })
}

/// Recompute sync lag from the majority tip (may lower horizon after a bad tip drops).
pub fn apply_preferred_horizon(
    targets: &[PeerSyncTarget],
    sync: &mut SyncStatus,
    local_head: Slot,
) {
    let horizon = match preferred_finalized(targets) {
        Some((root, slot)) => {
            let head = targets
                .iter()
                .filter(|t| t.remote.finalized_root() == root && t.remote.finalized_slot() == slot)
                .map(|t| t.remote.head_slot())
                .max()
                .unwrap_or(slot);
            head.max(slot)
        }
        None => local_head.get(),
    };
    sync.replace_peer_horizon(local_head, Slot::new(horizon));
}

/// Stage the next range/root fetch for the preferred peer still ahead of local head.
///
/// Deep lag prefers blocks-by-range. When the peer's **finalized** tip is also
/// ahead (head-behind-finalized), also stage a blocks-by-root for that
/// finalized root so recovery is not stuck on an empty range stream.
pub fn prepare_follow_up(
    targets: &[PeerSyncTarget],
    local_head_root: Hash32,
    local_head_slot: u64,
    tracker: &mut RequestTracker,
) -> Result<CatchupOutbounds, String> {
    let Some(target) = select_catchup_target(targets, local_head_slot) else {
        return Ok(CatchupOutbounds::default());
    };
    let lag = target
        .remote
        .head_slot()
        .max(target.remote.finalized_slot())
        .saturating_sub(local_head_slot);
    let need_finalized_pin = behind_finalized(&target.remote, local_head_slot);

    if lag >= RANGE_PREFER_LAG_SLOTS {
        let blocks_by_range = prepare_blocks_by_range_outbound(
            target.peer,
            local_head_slot,
            &target.remote,
            tracker,
        )
        .map_err(|e| e.to_string())?;
        let blocks_by_root = if need_finalized_pin {
            let roots = recovery_roots(&target.remote, local_head_root);
            prepare_blocks_by_root_for_roots(target.peer, roots, tracker)
                .map_err(|e| e.to_string())?
        } else {
            None
        };
        Ok(CatchupOutbounds {
            blocks_by_root,
            blocks_by_range,
        })
    } else if need_finalized_pin {
        let roots = recovery_roots(&target.remote, local_head_root);
        Ok(CatchupOutbounds {
            blocks_by_root: prepare_blocks_by_root_for_roots(target.peer, roots, tracker)
                .map_err(|e| e.to_string())?,
            blocks_by_range: None,
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

    fn tip_finalized(head: u64, finalized: u64, fz_root: Hash32) -> Status {
        Status {
            finalized: Checkpoint {
                root: fz_root,
                slot: finalized,
            },
            head: Checkpoint {
                root: [9u8; 32],
                slot: head,
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
    fn remember_raises_finalized_when_head_unchanged() {
        let mut targets = Vec::new();
        let peer = [1u8; 32];
        remember_target(&mut targets, peer, tip_finalized(10, 2, [2u8; 32]));
        remember_target(&mut targets, peer, tip_finalized(10, 8, [8u8; 32]));
        assert_eq!(targets[0].remote.head_slot(), 10);
        assert_eq!(targets[0].remote.finalized_slot(), 8);
        assert_eq!(targets[0].remote.finalized_root(), [8u8; 32]);
    }

    #[test]
    fn prune_keeps_behind_finalized_even_if_head_matched() {
        let mut targets = vec![PeerSyncTarget {
            peer: [1u8; 32],
            remote: tip_finalized(5, 8, [3u8; 32]),
        }];
        prune_caught_up(&mut targets, 5);
        assert_eq!(targets.len(), 1);
        prune_caught_up(&mut targets, 8);
        assert!(targets.is_empty());
    }

    #[test]
    fn follow_up_stages_range_when_deep() {
        let targets = vec![PeerSyncTarget {
            peer: [2u8; 32],
            remote: tip(20),
        }];
        let mut tracker = RequestTracker::default();
        let out = prepare_follow_up(&targets, [0u8; 32], 3, &mut tracker).unwrap();
        assert!(out.blocks_by_range.is_some());
        assert!(out.blocks_by_root.is_none());
    }

    #[test]
    fn follow_up_pins_finalized_root_when_behind_finalized() {
        let targets = vec![PeerSyncTarget {
            peer: [2u8; 32],
            remote: tip_finalized(20, 15, [7u8; 32]),
        }];
        let mut tracker = RequestTracker::default();
        let out = prepare_follow_up(&targets, [0u8; 32], 3, &mut tracker).unwrap();
        assert!(out.blocks_by_range.is_some());
        assert!(
            out.blocks_by_root.is_some(),
            "head-behind-finalized must also stage finalized root fetch"
        );
    }

    #[test]
    fn majority_finalized_beats_lone_ahead_adversary() {
        let honest = [0xau8; 32];
        let bad = [0xbbu8; 32];
        let targets = vec![
            PeerSyncTarget {
                peer: [1u8; 32],
                remote: tip_finalized(12, 10, honest),
            },
            PeerSyncTarget {
                peer: [2u8; 32],
                remote: tip_finalized(12, 10, honest),
            },
            PeerSyncTarget {
                peer: [3u8; 32],
                remote: tip_finalized(40, 30, bad),
            },
        ];
        assert_eq!(preferred_finalized(&targets), Some((honest, 10)));
        let chosen = select_catchup_target(&targets, 5).unwrap();
        assert_eq!(chosen.remote.finalized_root(), honest);
        assert_ne!(chosen.peer, [3u8; 32]);
        let mut tracker = RequestTracker::default();
        let out = prepare_follow_up(&targets, [0u8; 32], 5, &mut tracker).unwrap();
        assert!(out.blocks_by_range.is_some() || out.blocks_by_root.is_some());
    }

    #[test]
    fn no_majority_skips_disputed_ahead_tips() {
        let targets = vec![
            PeerSyncTarget {
                peer: [1u8; 32],
                remote: tip_finalized(12, 10, [1u8; 32]),
            },
            PeerSyncTarget {
                peer: [2u8; 32],
                remote: tip_finalized(40, 30, [2u8; 32]),
            },
        ];
        assert_eq!(preferred_finalized(&targets), None);
        assert!(select_catchup_target(&targets, 5).is_none());
    }

    #[test]
    fn preferred_horizon_ignores_adversary_slot() {
        let honest = [0xau8; 32];
        let bad = [0xbbu8; 32];
        let targets = vec![
            PeerSyncTarget {
                peer: [1u8; 32],
                remote: tip_finalized(12, 10, honest),
            },
            PeerSyncTarget {
                peer: [2u8; 32],
                remote: tip_finalized(12, 10, honest),
            },
            PeerSyncTarget {
                peer: [3u8; 32],
                remote: tip_finalized(40, 30, bad),
            },
        ];
        let mut sync = SyncStatus::new(Slot::new(0), Slot::new(99));
        apply_preferred_horizon(&targets, &mut sync, Slot::new(5));
        assert_eq!(sync.peer_horizon.get(), 12);
        assert_eq!(sync.lag(), 7);
    }
}
