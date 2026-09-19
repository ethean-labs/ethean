//! Prepare outbound blocks-by-root requests after a Status head gap.

use crate::error::Result;
use crate::reqresp::blocks_by_root::{
    blocks_by_root_for_status_gap, blocks_by_root_protocol_id, encode_blocks_by_root,
};
use crate::reqresp::tracker::{RequestId, RequestTracker};
use ethean_network_wire::Status;
use ethean_primitives::Hash32;

/// One blocks-by-root request ready for a Lean req/resp stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundBlocksByRootRequest {
    /// Peer fingerprint that advertised the remote head.
    pub peer: Hash32,
    /// Tracker id until response or disconnect.
    pub request_id: RequestId,
    /// Lean blocks-by-root protocol id.
    pub protocol_id: &'static str,
    /// Encoded request body (length-prefixed roots scaffold).
    pub payload: Vec<u8>,
}

/// Build an outbound blocks-by-root request when remote head differs from local.
pub fn prepare_blocks_by_root_outbound(
    peer: Hash32,
    local_head: Hash32,
    remote: &Status,
    tracker: &mut RequestTracker,
) -> Result<Option<OutboundBlocksByRootRequest>> {
    let Some(req) = blocks_by_root_for_status_gap(local_head, remote)? else {
        return Ok(None);
    };
    let request_id = tracker.insert(peer);
    Ok(Some(OutboundBlocksByRootRequest {
        peer,
        request_id,
        protocol_id: blocks_by_root_protocol_id(),
        payload: encode_blocks_by_root(&req),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stages_when_heads_differ() {
        let remote = Status {
            genesis_root: [1u8; 32],
            fork_segment: "aabbccdd".into(),
            head_slot: 9,
            head_root: [5u8; 32],
            finalized_slot: 0,
            finalized_root: [0u8; 32],
        };
        let mut tracker = RequestTracker::default();
        let peer = [7u8; 32];
        let out = prepare_blocks_by_root_outbound(peer, [2u8; 32], &remote, &mut tracker)
            .unwrap()
            .expect("gap");
        assert_eq!(out.peer, peer);
        assert!(!out.payload.is_empty());
        assert!(out.protocol_id.contains("blocks_by_root"));
        assert_eq!(tracker.len(), 1);
    }

    #[test]
    fn skips_when_heads_match() {
        let root = [3u8; 32];
        let remote = Status {
            genesis_root: [1u8; 32],
            fork_segment: "aabbccdd".into(),
            head_slot: 1,
            head_root: root,
            finalized_slot: 0,
            finalized_root: [0u8; 32],
        };
        let mut tracker = RequestTracker::default();
        assert!(prepare_blocks_by_root_outbound([1u8; 32], root, &remote, &mut tracker)
            .unwrap()
            .is_none());
        assert!(tracker.is_empty());
    }
}
