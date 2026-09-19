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
    if remote.head_root == local_head || remote.head_root == Hash32::default() {
        return Ok(None);
    }
    BlocksByRootRequest::new(vec![remote.head_root])
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

/// Encode a blocks-by-root request as length-prefixed roots (scaffold, not leanSpec SSZ yet).
pub fn encode_blocks_by_root(req: &BlocksByRootRequest) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + req.roots.len() * 32);
    let n = req.roots.len() as u32;
    out.extend_from_slice(&n.to_le_bytes());
    for root in &req.roots {
        out.extend_from_slice(root);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gap_requests_remote_head() {
        let remote = Status {
            genesis_root: [1u8; 32],
            fork_segment: "aabbccdd".into(),
            head_slot: 9,
            head_root: [5u8; 32],
            finalized_slot: 0,
            finalized_root: [0u8; 32],
        };
        let req = blocks_by_root_for_status_gap([2u8; 32], &remote)
            .unwrap()
            .expect("gap");
        assert_eq!(req.roots, vec![[5u8; 32]]);
        let enc = encode_blocks_by_root(&req);
        assert_eq!(&enc[..4], &1u32.to_le_bytes());
        assert_eq!(&enc[4..], &[5u8; 32]);
    }

    #[test]
    fn no_gap_when_heads_match() {
        let root = [3u8; 32];
        let remote = Status {
            genesis_root: [1u8; 32],
            fork_segment: "aabbccdd".into(),
            head_slot: 1,
            head_root: root,
            finalized_slot: 0,
            finalized_root: [0u8; 32],
        };
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
