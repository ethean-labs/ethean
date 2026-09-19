//! Prepare outbound Lean Status request payloads for connected peers.

use crate::error::Result;
use crate::reqresp::status_session::StatusSessionBook;
use crate::reqresp::tracker::{RequestId, RequestTracker};
use ethean_primitives::Hash32;

/// One Status request ready to send on a Lean req/resp stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundStatusRequest {
    /// Peer fingerprint (SHA-256 of PeerId bytes).
    pub peer: Hash32,
    /// Tracker id until a response arrives or the peer disconnects.
    pub request_id: RequestId,
    /// Lean Status protocol id.
    pub protocol_id: &'static str,
    /// SSZ-encoded local Status body.
    pub payload: Vec<u8>,
}

/// Encode pending Status payloads and register them on the request tracker.
pub fn prepare_status_outbounds(
    book: &StatusSessionBook,
    tracker: &mut RequestTracker,
) -> Result<Vec<OutboundStatusRequest>> {
    let mut out = Vec::new();
    for peer in book.pending_peers() {
        let payload = book.encode_local_for(&peer)?;
        let request_id = tracker.insert(peer);
        out.push(OutboundStatusRequest {
            peer,
            request_id,
            protocol_id: StatusSessionBook::protocol_id(),
            payload,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_network_wire::Status;

    fn sample() -> Status {
        Status {
            genesis_root: [1u8; 32],
            fork_segment: "aabbccdd".into(),
            head_slot: 0,
            head_root: [0u8; 32],
            finalized_slot: 0,
            finalized_root: [0u8; 32],
        }
    }

    #[test]
    fn prepares_one_outbound_per_pending_peer() {
        let mut book = StatusSessionBook::default();
        let peer = [4u8; 32];
        book.on_peer_connected(peer, sample());
        let mut tracker = RequestTracker::default();
        let reqs = prepare_status_outbounds(&book, &mut tracker).expect("prepare");
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].peer, peer);
        assert!(!reqs[0].payload.is_empty());
        assert!(reqs[0].protocol_id.starts_with("/leanconsensus/req/status/"));
        assert_eq!(tracker.len(), 1);
    }
}
