//! Profile-derived Lean gossip and req/resp topic strings.

use ethean_primitives::Hash32;
use sha2::{Digest, Sha256};

use crate::error::{Result, WireError};

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
    if fork.is_empty() || fork.contains('/') {
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
    use crate::fork_id::fork_segment_from_name;
    use crate::fork_id::LSTAR_GOSSIP_DIGEST;

    #[test]
    fn lstar_block_topic_uses_gossip_digest() {
        let fork = fork_segment_from_name("lstar").unwrap();
        assert_eq!(fork, LSTAR_GOSSIP_DIGEST);
        let t = topic_block(&fork).unwrap();
        assert_eq!(t, "/leanconsensus/12345678/block/ssz_snappy");
    }

    #[test]
    fn topic_vectors_from_leanspec() {
        assert_eq!(
            topic_block("12345678").unwrap(),
            "/leanconsensus/12345678/block/ssz_snappy"
        );
        assert_eq!(
            topic_block("aabbccdd").unwrap(),
            "/leanconsensus/aabbccdd/block/ssz_snappy"
        );
        assert_eq!(
            topic_aggregation("12345678").unwrap(),
            "/leanconsensus/12345678/aggregation/ssz_snappy"
        );
        assert_eq!(
            topic_attestation("12345678", 0).unwrap(),
            "/leanconsensus/12345678/attestation_0/ssz_snappy"
        );
        assert_eq!(
            topic_attestation("12345678", 7).unwrap(),
            "/leanconsensus/12345678/attestation_7/ssz_snappy"
        );
        assert_eq!(
            topic_attestation("12345678", 63).unwrap(),
            "/leanconsensus/12345678/attestation_63/ssz_snappy"
        );
        assert_eq!(
            topic_block("00000000").unwrap(),
            "/leanconsensus/00000000/block/ssz_snappy"
        );
        assert_eq!(
            topic_block("ffffffff").unwrap(),
            "/leanconsensus/ffffffff/block/ssz_snappy"
        );
    }

    #[test]
    fn rejects_empty_or_slash() {
        assert!(topic_block("").is_err());
        assert!(topic_block("12/34").is_err());
    }
}
