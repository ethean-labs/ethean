//! Build blocks-by-range requests from Status slot gaps.

use crate::error::{NetworkError, Result};
use ethean_network_wire::{rpc_blocks_by_range, BlocksByRangeRequest, Status};
use ethean_network_wire::limits::MAX_BLOCKS_PER_REQUEST;

/// Lean blocks-by-range protocol id.
pub fn blocks_by_range_protocol_id() -> &'static str {
    rpc_blocks_by_range()
}

/// Request a contiguous slot range when the remote head slot is ahead of local.
///
/// Used for deep catch-up once a slot→block serve cache exists. Step is always 1.
pub fn blocks_by_range_for_status_gap(
    local_head_slot: u64,
    remote: &Status,
) -> Result<Option<BlocksByRangeRequest>> {
    if remote.head_slot <= local_head_slot {
        return Ok(None);
    }
    let lag = remote.head_slot.saturating_sub(local_head_slot);
    let count = lag.min(MAX_BLOCKS_PER_REQUEST).max(1);
    let start = local_head_slot.saturating_add(1);
    BlocksByRangeRequest::new(start, count, 1)
        .map(Some)
        .map_err(|e| NetworkError::Handshake(e.to_string()))
}

/// Encode a blocks-by-range request as three little-endian u64 fields.
pub fn encode_blocks_by_range(req: &BlocksByRangeRequest) -> Vec<u8> {
    let mut out = Vec::with_capacity(24);
    out.extend_from_slice(&req.start_slot.to_le_bytes());
    out.extend_from_slice(&req.count.to_le_bytes());
    out.extend_from_slice(&req.step.to_le_bytes());
    out
}

/// Decode a scaffold blocks-by-range request body.
pub fn decode_blocks_by_range(input: &[u8]) -> Result<BlocksByRangeRequest> {
    if input.len() < 24 {
        return Err(NetworkError::Handshake(
            "blocks-by-range request too short".into(),
        ));
    }
    let start = u64::from_le_bytes(input[0..8].try_into().unwrap_or([0; 8]));
    let count = u64::from_le_bytes(input[8..16].try_into().unwrap_or([0; 8]));
    let step = u64::from_le_bytes(input[16..24].try_into().unwrap_or([0; 8]));
    BlocksByRangeRequest::new(start, count, step).map_err(|e| NetworkError::Handshake(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gap_builds_range_from_local_plus_one() {
        let remote = Status {
            genesis_root: [1u8; 32],
            fork_segment: "aabbccdd".into(),
            head_slot: 10,
            head_root: [5u8; 32],
            finalized_slot: 0,
            finalized_root: [0u8; 32],
        };
        let req = blocks_by_range_for_status_gap(3, &remote)
            .unwrap()
            .expect("gap");
        assert_eq!(req.start_slot, 4);
        assert_eq!(req.count, 7);
        assert_eq!(req.step, 1);
        let enc = encode_blocks_by_range(&req);
        let dec = decode_blocks_by_range(&enc).unwrap();
        assert_eq!(dec, req);
    }

    #[test]
    fn no_gap_when_caught_up() {
        let remote = Status {
            genesis_root: [1u8; 32],
            fork_segment: "aabbccdd".into(),
            head_slot: 3,
            head_root: [5u8; 32],
            finalized_slot: 0,
            finalized_root: [0u8; 32],
        };
        assert!(blocks_by_range_for_status_gap(3, &remote)
            .unwrap()
            .is_none());
    }
}
