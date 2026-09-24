//! Prepare outbound blocks-by-range requests after a Status slot gap.

use crate::error::Result;
use crate::reqresp::blocks_by_range::{
    blocks_by_range_for_status_gap, blocks_by_range_protocol_id, encode_blocks_by_range,
};
use crate::reqresp::tracker::{RequestId, RequestTracker};
use ethean_network_wire::Status;
use ethean_primitives::Hash32;

/// One blocks-by-range request ready for a Lean req/resp stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundBlocksByRangeRequest {
    /// Peer fingerprint that advertised the remote head.
    pub peer: Hash32,
    /// Tracker id until response or disconnect.
    pub request_id: RequestId,
    /// Lean blocks-by-range protocol id.
    pub protocol_id: &'static str,
    /// Encoded request body (start_slot + count, 16 bytes).
    pub payload: Vec<u8>,
}

/// Build an outbound blocks-by-range request when the remote head slot is ahead.
pub fn prepare_blocks_by_range_outbound(
    peer: Hash32,
    local_head_slot: u64,
    remote: &Status,
    tracker: &mut RequestTracker,
) -> Result<Option<OutboundBlocksByRangeRequest>> {
    let Some(req) = blocks_by_range_for_status_gap(local_head_slot, remote)? else {
        return Ok(None);
    };
    let request_id = tracker.insert(peer);
    Ok(Some(OutboundBlocksByRangeRequest {
        peer,
        request_id,
        protocol_id: blocks_by_range_protocol_id(),
        payload: encode_blocks_by_range(&req),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stages_when_slots_lag() {
        let remote = Status {
            finalized: ethean_network_wire::Checkpoint {
                root: [0u8; 32],
                slot: 0,
            },
            head: ethean_network_wire::Checkpoint {
                root: [5u8; 32],
                slot: 10,
            },
        };
        let mut tracker = RequestTracker::default();
        let peer = [7u8; 32];
        let out = prepare_blocks_by_range_outbound(peer, 3, &remote, &mut tracker)
            .unwrap()
            .expect("gap");
        assert_eq!(out.peer, peer);
        assert_eq!(out.payload.len(), 16);
        assert!(out.protocol_id.contains("blocks_by_range"));
        assert_eq!(tracker.len(), 1);
    }

    #[test]
    fn skips_when_caught_up() {
        let remote = Status {
            finalized: ethean_network_wire::Checkpoint {
                root: [0u8; 32],
                slot: 0,
            },
            head: ethean_network_wire::Checkpoint {
                root: [5u8; 32],
                slot: 3,
            },
        };
        let mut tracker = RequestTracker::default();
        assert!(prepare_blocks_by_range_outbound([1u8; 32], 3, &remote, &mut tracker)
            .unwrap()
            .is_none());
        assert!(tracker.is_empty());
    }
}
