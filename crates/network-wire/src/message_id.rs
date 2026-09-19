//! Gossip message-ID derivation (preimage order still open in Phase 00).

use ethean_primitives::Hash32;
use sha2::{Digest, Sha256};

/// Interim message-ID: SHA-256(topic_len_le || topic || data).
///
/// Phase 00 left `gossip_message_id_preimage_order` open; this binding is
/// deterministic for Ethean until the ledger pins peer-compatible order.
pub fn message_id(topic: &str, data: &[u8]) -> Hash32 {
    let mut hasher = Sha256::new();
    hasher.update((topic.len() as u32).to_le_bytes());
    hasher.update(topic.as_bytes());
    hasher.update((data.len() as u32).to_le_bytes());
    hasher.update(data);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_and_topic_sensitive() {
        let a = message_id("/leanconsensus/abcd/block/ssz_snappy", b"payload");
        let b = message_id("/leanconsensus/abcd/block/ssz_snappy", b"payload");
        let c = message_id("/leanconsensus/abcd/aggregation/ssz_snappy", b"payload");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
