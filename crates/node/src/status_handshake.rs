//! Drive StatusSessionBook from QuicSwarm pump events.

use crate::chain_owner::ChainOwner;
use crate::local_status::{local_status, observe_remote_status};
use ethean_network::{
    blocks_by_root_for_status_gap, encode_blocks_by_root, PumpEvent, StatusSessionBook,
};
use ethean_network_wire::Status;
use ethean_primitives::{Hash32, Slot};
use ethean_sync::SyncStatus;
use tracing::info;

/// Queue Status handshakes for connected peer fingerprints.
pub fn queue_peers(
    book: &mut StatusSessionBook,
    local: &Status,
    peers: &[Hash32],
) -> usize {
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
pub fn on_pump_event(
    book: &mut StatusSessionBook,
    event: &PumpEvent,
    local: &Status,
) -> usize {
    match event {
        PumpEvent::ConnectionEstablished { peer: Some(peer) } => {
            book.on_peer_connected(*peer, local.clone());
            1
        }
        PumpEvent::ConnectionClosed { peer: Some(peer) } => {
            book.on_peer_disconnected(peer);
            0
        }
        _ => 0,
    }
}

/// Complete a handshake with remote Status bytes and update sync + optional blocks-by-root plan.
pub fn complete_status_handshake(
    book: &mut StatusSessionBook,
    sync: &mut SyncStatus,
    owner: &ChainOwner,
    peer: Hash32,
    remote_bytes: &[u8],
) -> Result<Option<Vec<u8>>, String> {
    let exchange = book
        .ingest_remote(peer, remote_bytes)
        .map_err(|e| e.to_string())?;
    let local_head = owner
        .head_state
        .as_ref()
        .map(|s| s.slot)
        .unwrap_or(Slot::new(exchange.local.head_slot));
    observe_remote_status(sync, local_head, &exchange.remote);
    info!(
        peer_head = exchange.remote.head_slot,
        lag = sync.lag(),
        "Status handshake completed"
    );
    let req = blocks_by_root_for_status_gap(owner.head_root, &exchange.remote)
        .map_err(|e| e.to_string())?;
    Ok(req.map(|r| encode_blocks_by_root(&r)))
}

/// Build local Status from owner + genesis root + fork segment.
pub fn build_local(owner: &ChainOwner, genesis_root: Hash32, fork_segment: &str) -> Status {
    local_status(owner, genesis_root, fork_segment)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_network_wire::Status as WireStatus;

    #[test]
    fn connection_queues_pending() {
        let mut book = StatusSessionBook::default();
        let local = WireStatus {
            genesis_root: [1u8; 32],
            fork_segment: "aabbccdd".into(),
            head_slot: 0,
            head_root: [0u8; 32],
            finalized_slot: 0,
            finalized_root: [0u8; 32],
        };
        let peer = [9u8; 32];
        let n = on_pump_event(
            &mut book,
            &PumpEvent::ConnectionEstablished { peer: Some(peer) },
            &local,
        );
        assert_eq!(n, 1);
        assert_eq!(book.pending_len(), 1);
    }

    #[test]
    fn queue_peers_counts() {
        let mut book = StatusSessionBook::default();
        let local = WireStatus {
            genesis_root: [1u8; 32],
            fork_segment: "aabbccdd".into(),
            head_slot: 0,
            head_root: [0u8; 32],
            finalized_slot: 0,
            finalized_root: [0u8; 32],
        };
        assert_eq!(queue_peers(&mut book, &local, &[[8u8; 32]]), 1);
    }
}
