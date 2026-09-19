//! Profile-derived Lean gossip and req/resp topic strings.

use ethean_primitives::Hash32;
use sha2::{Digest, Sha256};

use crate::error::{Result, WireError};
use crate::fork_id::fork_segment_hex;

/// Forbidden placeholder fork id from the migration plan (never emit).
pub const FORBIDDEN_DUMMY_FORK: &str = "12345678";

/// Derive an 8-byte-hex fork segment from `fork_name` until Phase 00 closes digest bytes.
///
/// This is an interim Ethean mapping (SHA-256 prefix), not a leanSpec wire pin.
pub fn fork_segment_from_name(fork_name: &str) -> Result<String> {
    fork_segment_hex(fork_name)
}

/// Gossip block topic: `/leanconsensus/{fork}/block/ssz_snappy`.
pub fn topic_block(fork: &str) -> Result<String> {
    validate_fork(fork)?;
    Ok(format!("/leanconsensus/{fork}/block/ssz_snappy"))
}

/// Gossip attestation subnet topic.
pub fn topic_attestation(fork: &str, subnet: u16) -> Result<String> {
    validate_fork(fork)?;
    Ok(format!(
        "/leanconsensus/{fork}/attestation_{subnet}/ssz_snappy"
    ))
}

/// Gossip aggregation topic.
pub fn topic_aggregation(fork: &str) -> Result<String> {
    validate_fork(fork)?;
    Ok(format!("/leanconsensus/{fork}/aggregation/ssz_snappy"))
}

/// Req/resp Status protocol id.
pub fn rpc_status() -> &'static str {
    "/leanconsensus/req/status/1/ssz_snappy"
}

/// Blocks-by-root protocol id.
pub fn rpc_blocks_by_root() -> &'static str {
    "/leanconsensus/req/blocks_by_root/1/ssz_snappy"
}

/// Blocks-by-range protocol id.
pub fn rpc_blocks_by_range() -> &'static str {
    "/leanconsensus/req/blocks_by_range/1/ssz_snappy"
}

fn validate_fork(fork: &str) -> Result<()> {
    if fork.is_empty() || fork == FORBIDDEN_DUMMY_FORK || fork.contains('/') {
        return Err(WireError::InvalidTopic(format!("invalid fork segment {fork}")));
    }
    Ok(())
}

/// Hash32 helper used by message-id tests.
pub fn hash32_of(bytes: &[u8]) -> Hash32 {
    Sha256::digest(bytes).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_topic_shape() {
        let fork = fork_segment_from_name("lstar").unwrap();
        let t = topic_block(&fork).unwrap();
        assert!(t.starts_with("/leanconsensus/"));
        assert!(t.ends_with("/block/ssz_snappy"));
        assert!(!t.contains(FORBIDDEN_DUMMY_FORK));
    }

    #[test]
    fn rejects_dummy_fork() {
        assert!(topic_block(FORBIDDEN_DUMMY_FORK).is_err());
    }
}
