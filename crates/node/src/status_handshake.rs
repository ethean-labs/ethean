//! Drive StatusSessionBook from QuicSwarm pump events.

use crate::chain_owner::ChainOwner;
use crate::local_status::local_status;
use ethean_network::{
    OutboundBlocksByRangeRequest, OutboundBlocksByRootRequest, PumpEvent, RequestTracker,
    StatusSessionBook,
};
use ethean_network_wire::Status;
use ethean_primitives::Hash32;
use ethean_sync::SyncStatus;
use tracing::info;

/// Prefer blocks-by-range when remote head is at least this many slots ahead.
///
/// Smaller gaps use blocks-by-root (head / parent walk). Matches the duty sync-lag
/// threshold so deep catch-up does not double-fetch root + range.
pub const RANGE_PREFER_LAG_SLOTS: u64 = 4;

/// Result of a Status handshake. Block catch-up is staged by `sync_catchup`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StatusSyncOutbounds {
    /// Always `None` — majority catch-up stages via `sync_catchup`.
    pub blocks_by_root: Option<OutboundBlocksByRootRequest>,
    /// Always `None` — majority catch-up stages via `sync_catchup`.
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

/// Complete a handshake with remote Status bytes.
///
/// Block catch-up is **not** staged here: the caller remembers the tip and runs
/// majority-aware [`crate::sync_catchup::prepare_follow_up`] so a lone ahead
/// adversarial peer cannot hijack sync when honest helpers agree.
pub fn complete_status_handshake(
    book: &mut StatusSessionBook,
    sync: &mut SyncStatus,
    _owner: &ChainOwner,
    peer: Hash32,
    remote_bytes: &[u8],
    _tracker: &mut RequestTracker,
) -> Result<StatusSyncOutbounds, String> {
    let exchange = book
        .ingest_remote(peer, remote_bytes)
        .map_err(|e| e.to_string())?;
    let _ = sync;
    info!(
        peer_head = exchange.remote.head_slot(),
        peer_finalized = exchange.remote.finalized_slot(),
        "Status handshake completed"
    );
    Ok(StatusSyncOutbounds {
        blocks_by_root: None,
        blocks_by_range: None,
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
