//! Build blocks-by-root requests from Status / parent gaps.

use crate::error::{NetworkError, Result};
use ethean_network_wire::{rpc_blocks_by_root, BlocksByRootRequest, Status};
use ethean_primitives::Hash32;

/// Lean blocks-by-root protocol id.
pub fn blocks_by_root_protocol_id() -> &'static str {
    rpc_blocks_by_root()
}

/// Request the peer head root when it differs from our local head.
pub fn blocks_by_root_for_status_gap(
    local_head: Hash32,
    remote: &Status,
) -> Result<Option<BlocksByRootRequest>> {
    if remote.head_root() == local_head || remote.head_root() == Hash32::default() {
        return Ok(None);
    }
    BlocksByRootRequest::new(vec![remote.head_root()])
        .map(Some)
        .map_err(|e| NetworkError::Handshake(e.to_string()))
}

/// Build a blocks-by-root request for an explicit root list (parent catch-up).
pub fn blocks_by_root_for_roots(roots: Vec<Hash32>) -> Result<Option<BlocksByRootRequest>> {
    let mut uniq = Vec::new();
    for r in roots {
        if r == Hash32::default() {
            continue;
        }
        if !uniq.contains(&r) {
            uniq.push(r);
        }
    }
    if uniq.is_empty() {
        return Ok(None);
    }
    BlocksByRootRequest::new(uniq)
        .map(Some)
        .map_err(|e| NetworkError::Handshake(e.to_string()))
}

/// Encode a blocks-by-root request as SSZ `List[Bytes32, 1024]`.
pub fn encode_blocks_by_root(req: &BlocksByRootRequest) -> Vec<u8> {
    req.encode()
}

/// Decode a SSZ blocks-by-root request.
pub fn decode_blocks_by_root(input: &[u8]) -> Result<BlocksByRootRequest> {
    BlocksByRootRequest::decode(input).map_err(|e| NetworkError::Handshake(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_network_wire::Checkpoint;

    fn remote_with_head(root: Hash32, slot: u64) -> Status {
        Status {
            finalized: Checkpoint {
                root: [0u8; 32],
                slot: 0,
            },
            head: Checkpoint { root, slot },
        }
    }

    #[test]
    fn gap_requests_remote_head() {
        let remote = remote_with_head([5u8; 32], 9);
        let req = blocks_by_root_for_status_gap([2u8; 32], &remote)
            .unwrap()
            .expect("gap");
        assert_eq!(req.roots, vec![[5u8; 32]]);
        let enc = encode_blocks_by_root(&req);
        assert_eq!(&enc[..4], &4u32.to_le_bytes());
        assert_eq!(&enc[4..], &[5u8; 32]);
    }

    #[test]
    fn no_gap_when_heads_match() {
        let root = [3u8; 32];
        let remote = remote_with_head(root, 1);
        assert!(blocks_by_root_for_status_gap(root, &remote)
            .unwrap()
            .is_none());
    }

    #[test]
    fn roots_helper_dedupes_and_skips_zero() {
        let req = blocks_by_root_for_roots(vec![[0u8; 32], [5u8; 32], [5u8; 32]])
            .unwrap()
            .expect("roots");
        assert_eq!(req.roots, vec![[5u8; 32]]);
    }
}
