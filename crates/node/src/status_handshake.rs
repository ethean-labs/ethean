//! Drive StatusSessionBook from QuicSwarm pump events.

use crate::chain_owner::ChainOwner;
use crate::local_status::{local_status, observe_remote_status};
use ethean_network::{
    prepare_blocks_by_range_outbound, prepare_blocks_by_root_outbound,
    OutboundBlocksByRangeRequest, OutboundBlocksByRootRequest, PumpEvent, RequestTracker,
    StatusSessionBook,
};
use ethean_network_wire::Status;
use ethean_primitives::{Hash32, Slot};
use ethean_sync::SyncStatus;
use tracing::info;

/// Prefer blocks-by-range when remote head is at least this many slots ahead.
///
/// Smaller gaps use blocks-by-root (head / parent walk). Matches the duty sync-lag
/// threshold so deep catch-up does not double-fetch root + range.
pub const RANGE_PREFER_LAG_SLOTS: u64 = 4;

/// Blocks fetch requests staged after a successful Status handshake.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StatusSyncOutbounds {
    /// Immediate head catch-up via blocks-by-root (when heads differ and lag is small).
    pub blocks_by_root: Option<OutboundBlocksByRootRequest>,
    /// Deep slot catch-up via blocks-by-range (when remote head slot lag is large).
    pub blocks_by_range: Option<OutboundBlocksByRangeRequest>,
    /// Remote Status tip (for follow-up catch-up while still behind).
    pub remote: Option<Status>,
}

/// Queue Status handshakes for connected peer fingerprints.
pub fn queue_peers(book: &mut StatusSessionBook, local: &Status, peers: &[Hash32]) -> usize {
    let mut n = 0;
    for peer in peers {
        book.on_peer_connected(*peer, local.clone());
        info!(
            peer0 = peer[0],
            peer1 = peer[1],
            protocol = StatusSessionBook::protocol_id(),
            "queued Status handshake for peer"
        );
        n += 1;
    }
    n
}

/// Apply one pump event to the Status session book.
pub fn on_pump_event(book: &mut StatusSessionBook, event: &PumpEvent, local: &Status) -> usize {
    match event {
        PumpEvent::ConnectionEstablished {
            peer: Some(peer), ..
        } => {
            book.on_peer_connected(*peer, local.clone());
            1
        }
        PumpEvent::ConnectionClosed {
            peer: Some(peer), ..
        } => {
            book.on_peer_disconnected(peer);
            0
        }
        _ => 0,
    }
}

/// Complete a handshake with remote Status bytes and stage block sync requests.
pub fn complete_status_handshake(
    book: &mut StatusSessionBook,
    sync: &mut SyncStatus,
    owner: &ChainOwner,
    peer: Hash32,
    remote_bytes: &[u8],
    tracker: &mut RequestTracker,
) -> Result<StatusSyncOutbounds, String> {
    let exchange = book
        .ingest_remote(peer, remote_bytes)
        .map_err(|e| e.to_string())?;
    let local_head = owner
        .head_state
        .as_ref()
        .map(|s| s.slot)
        .unwrap_or(Slot::new(exchange.local.head_slot()));
    observe_remote_status(sync, local_head, &exchange.remote);
    info!(
        peer_head = exchange.remote.head_slot(),
        lag = sync.lag(),
        "Status handshake completed"
    );
    let lag = exchange.remote.head_slot().saturating_sub(local_head.get());
    let (blocks_by_root, blocks_by_range) = if lag >= RANGE_PREFER_LAG_SLOTS {
        (
            None,
            prepare_blocks_by_range_outbound(peer, local_head.get(), &exchange.remote, tracker)
                .map_err(|e| e.to_string())?,
        )
    } else {
        (
            prepare_blocks_by_root_outbound(peer, owner.head_root, &exchange.remote, tracker)
                .map_err(|e| e.to_string())?,
            None,
        )
    };
    Ok(StatusSyncOutbounds {
        blocks_by_root,
        blocks_by_range,
        remote: Some(exchange.remote),
    })
}

/// Build local Status from the chain owner (head / finalized checkpoints).
pub fn build_local(owner: &ChainOwner) -> Status {
    local_status(owner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_network_wire::Status as WireStatus;

    #[test]
    fn connection_queues_pending() {
        let mut book = StatusSessionBook::default();
        let local = WireStatus::default();
        let peer = [9u8; 32];
        let n = on_pump_event(
            &mut book,
            &PumpEvent::ConnectionEstablished {
                peer: Some(peer),
                outbound: true,
            },
            &local,
        );
        assert_eq!(n, 1);
        assert_eq!(book.pending_len(), 1);
    }

    #[test]
    fn queue_peers_counts() {
        let mut book = StatusSessionBook::default();
        let local = WireStatus::default();
        assert_eq!(queue_peers(&mut book, &local, &[[8u8; 32]]), 1);
    }

    #[test]
    fn range_prefer_threshold_matches_sync_lag_default() {
        assert_eq!(RANGE_PREFER_LAG_SLOTS, 4);
    }
}
