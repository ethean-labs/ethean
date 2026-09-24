//! Build blocks-by-range requests from Status slot gaps.

use crate::error::{NetworkError, Result};
use ethean_network_wire::limits::MAX_BLOCKS_PER_REQUEST;
use ethean_network_wire::{rpc_blocks_by_range, BlocksByRangeRequest, Status};

/// Lean blocks-by-range protocol id.
pub fn blocks_by_range_protocol_id() -> &'static str {
    rpc_blocks_by_range()
}

/// Request a contiguous slot range when the remote head slot is ahead of local.
pub fn blocks_by_range_for_status_gap(
    local_head_slot: u64,
    remote: &Status,
) -> Result<Option<BlocksByRangeRequest>> {
    if remote.head_slot() <= local_head_slot {
        return Ok(None);
    }
    let lag = remote.head_slot().saturating_sub(local_head_slot);
    let count = lag.min(MAX_BLOCKS_PER_REQUEST).max(1);
    let start = local_head_slot.saturating_add(1);
    BlocksByRangeRequest::new(start, count)
        .map(Some)
        .map_err(|e| NetworkError::Handshake(e.to_string()))
}

/// Encode a blocks-by-range request as two little-endian u64 fields (no step).
pub fn encode_blocks_by_range(req: &BlocksByRangeRequest) -> Vec<u8> {
    req.encode()
}

/// Decode a SSZ blocks-by-range request body.
pub fn decode_blocks_by_range(input: &[u8]) -> Result<BlocksByRangeRequest> {
    BlocksByRangeRequest::decode(input).map_err(|e| NetworkError::Handshake(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_network_wire::Checkpoint;

    fn remote_at(slot: u64) -> Status {
        Status {
            finalized: Checkpoint {
                root: [0u8; 32],
                slot: 0,
            },
            head: Checkpoint {
                root: [5u8; 32],
                slot,
            },
        }
    }

    #[test]
    fn gap_builds_range_from_local_plus_one() {
        let remote = remote_at(10);
        let req = blocks_by_range_for_status_gap(3, &remote)
            .unwrap()
            .expect("gap");
        assert_eq!(req.start_slot, 4);
        assert_eq!(req.count, 7);
        let enc = encode_blocks_by_range(&req);
        assert_eq!(enc.len(), 16);
        let dec = decode_blocks_by_range(&enc).unwrap();
        assert_eq!(dec, req);
    }

    #[test]
    fn no_gap_when_caught_up() {
        let remote = remote_at(3);
        assert!(blocks_by_range_for_status_gap(3, &remote)
            .unwrap()
            .is_none());
    }
}
