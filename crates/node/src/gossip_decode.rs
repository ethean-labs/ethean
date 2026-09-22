//! Decode Lean gossip payloads (Snappy already removed).

use ethean_primitives::Hash32;
use ethean_types::{Block, SignedBlock};
use sha2::{Digest, Sha256};

/// A `SignedBlock` from the block topic (or a sync response).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedBlockGossip {
    /// `hash_tree_root` of the inner block.
    pub root: Hash32,
    /// Parent root the block extends.
    pub parent: Hash32,
    /// Inner block (convenience copy of `signed.block`).
    pub block: Block,
    /// Block with its merged proof.
    pub signed: SignedBlock,
}

/// Decode a block-topic payload. Only `SignedBlock` is valid on the wire.
pub fn try_decode_block(topic: &str, payload: &[u8]) -> Option<DecodedBlockGossip> {
    if !topic.contains("/block/") {
        return None;
    }
    let signed = SignedBlock::ssz_decode(payload).ok()?;
    let root = signed.block.hash_tree_root().ok()?;
    Some(DecodedBlockGossip {
        root,
        parent: signed.block.parent_root,
        block: signed.block.clone(),
        signed,
    })
}

/// Stable content id for a gossip payload: the block root for blocks, a
/// domain-separated SHA-256 of the payload otherwise.
pub fn content_root_for(topic: &str, payload: &[u8]) -> Hash32 {
    if let Some(decoded) = try_decode_block(topic, payload) {
        return decoded.root;
    }
    provisional_content_root(payload)
}

/// SHA-256 content id for payloads without a canonical root.
pub fn provisional_content_root(payload: &[u8]) -> Hash32 {
    let mut hasher = Sha256::new();
    hasher.update(b"ethean-gossip-ingest-v1");
    hasher.update(payload);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Slot, ValidatorIndex, HASH32_ZERO};
    use ethean_types::{BlockBody, MultiMessageAggregate};

    #[test]
    fn decodes_signed_blocks_only_on_block_topics() {
        let block = Block {
            slot: Slot::new(2),
            proposer_index: ValidatorIndex::new(1),
            parent_root: [3u8; 32],
            state_root: HASH32_ZERO,
            body: BlockBody::default(),
        };
        let signed = SignedBlock::new(block.clone(), MultiMessageAggregate::new(vec![9]).unwrap());
        let payload = signed.ssz_encode().unwrap();
        let decoded = try_decode_block("/leanconsensus/x/block/ssz_snappy", &payload).unwrap();
        assert_eq!(decoded.root, block.hash_tree_root().unwrap());
        assert_eq!(decoded.parent, [3u8; 32]);
        assert!(try_decode_block("/leanconsensus/x/aggregation/ssz_snappy", &payload).is_none());
        assert!(try_decode_block(
            "/leanconsensus/x/block/ssz_snappy",
            &block.ssz_encode().unwrap()
        )
        .is_none());
        assert_eq!(content_root_for("/x/block/y", &payload), decoded.root);
    }
}
