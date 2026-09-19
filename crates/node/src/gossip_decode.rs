//! Decode Lean gossip SSZ payloads into importable roots.

use ethean_primitives::Hash32;
use ethean_types::{
    AggregatedAttestation, Block, MultiMessageAggregate, SignedAggregatedAttestation, SignedBlock,
};
use sha2::{Digest, Sha256};

/// Block gossip that can drive import / state transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedBlockGossip {
    /// `hash_tree_root` of the block (not the signed envelope).
    pub root: Hash32,
    /// Parent root from the block header fields.
    pub parent: Hash32,
    /// Decoded block body used for structural transition.
    pub block: Block,
    /// Present when the payload was a `SignedBlock` envelope.
    pub signed: Option<SignedBlock>,
}

/// Attestation-subnet gossip retained for the aggregate pool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedAttestationGossip {
    /// Content id for ingest events (aggregate HTR or data root).
    pub content_root: Hash32,
    /// Attestation-data tree root (pool message key).
    pub message_root: Hash32,
    /// Attestation data slot.
    pub slot: u64,
    /// Participation / coverage count.
    pub coverage: u32,
    /// Proof bytes when a signed aggregate is present.
    pub proof: Vec<u8>,
}

/// Try to decode a `/block/` topic payload as `SignedBlock` or bare `Block`.
pub fn try_decode_block(topic: &str, payload: &[u8]) -> Option<DecodedBlockGossip> {
    if !topic.contains("/block/") {
        return None;
    }
    if let Ok(signed) = SignedBlock::ssz_decode(payload) {
        let root = signed.block.hash_tree_root().ok()?;
        return Some(DecodedBlockGossip {
            root,
            parent: signed.block.parent_root,
            block: signed.block.clone(),
            signed: Some(signed),
        });
    }
    if let Ok(block) = Block::ssz_decode(payload) {
        let root = block.hash_tree_root().ok()?;
        return Some(DecodedBlockGossip {
            root,
            parent: block.parent_root,
            block,
            signed: None,
        });
    }
    None
}

/// Decode `/attestation_*/` gossip into pool-ready fields.
pub fn try_decode_attestation(topic: &str, payload: &[u8]) -> Option<DecodedAttestationGossip> {
    if !topic.contains("/attestation_") {
        return None;
    }
    // Prefer unsigned aggregated shape first; signed envelopes can false-positive decode.
    if let Ok(agg) = AggregatedAttestation::ssz_decode(payload) {
        let content_root = agg.hash_tree_root().ok()?;
        let coverage = agg.aggregation_bits.bits.iter().filter(|b| **b).count() as u32;
        return Some(DecodedAttestationGossip {
            content_root,
            message_root: agg.data.hash_tree_root(),
            slot: agg.data.slot.get(),
            coverage,
            proof: Vec::new(),
        });
    }
    if let Ok(signed) = SignedAggregatedAttestation::ssz_decode(payload) {
        let message_root = signed.data.hash_tree_root();
        let coverage = signed
            .proof
            .participants
            .bits
            .iter()
            .filter(|b| **b)
            .count() as u32;
        return Some(DecodedAttestationGossip {
            content_root: message_root,
            message_root,
            slot: signed.data.slot.get(),
            coverage,
            proof: signed.proof.proof.clone(),
        });
    }
    None
}

/// Tree root for `/attestation_*/` gossip (aggregated or signed).
pub fn try_decode_attestation_root(topic: &str, payload: &[u8]) -> Option<Hash32> {
    try_decode_attestation(topic, payload).map(|d| d.content_root)
}

/// Prefer SSZ tree roots; otherwise domain-separated SHA-256 of bytes.
pub fn content_root_for(topic: &str, payload: &[u8]) -> Hash32 {
    if let Some(decoded) = try_decode_block(topic, payload) {
        return decoded.root;
    }
    if let Some(root) = try_decode_attestation_root(topic, payload) {
        return root;
    }
    if topic.contains("/aggregation/") {
        if let Ok(agg) = MultiMessageAggregate::new(payload.to_vec()) {
            if let Ok(root) = agg.hash_tree_root() {
                return root;
            }
        }
    }
    provisional_content_root(payload)
}

/// Fallback content id when SSZ decode is unavailable.
pub fn provisional_content_root(payload: &[u8]) -> Hash32 {
    let mut hasher = Sha256::new();
    hasher.update(b"ethean-gossip-ingest-v1");
    hasher.update(payload);
    let dig = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&dig);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Slot, ValidatorIndex};
    use ethean_types::{AggregationBits, AttestationData, BlockBody, Checkpoint};

    #[test]
    fn decodes_bare_block_topic() {
        let block = Block {
            slot: Slot::new(3),
            proposer_index: ValidatorIndex::new(0),
            parent_root: [1u8; 32],
            state_root: [2u8; 32],
            body: BlockBody::default(),
        };
        let enc = block.ssz_encode().unwrap();
        let topic = "/leanconsensus/abcd/block/ssz_snappy";
        let d = try_decode_block(topic, &enc).expect("decode");
        assert_eq!(d.parent, [1u8; 32]);
        assert_eq!(d.root, block.hash_tree_root().unwrap());
        assert!(d.signed.is_none());
        assert_eq!(content_root_for(topic, &enc), d.root);
    }

    #[test]
    fn attestation_topic_uses_tree_root() {
        let agg = AggregatedAttestation {
            aggregation_bits: AggregationBits {
                bits: vec![true, true],
            },
            data: AttestationData {
                slot: Slot::new(2),
                head: Checkpoint::genesis(),
                target: Checkpoint::genesis(),
                source: Checkpoint::genesis(),
            },
        };
        let enc = agg.ssz_encode();
        let topic = "/leanconsensus/abcd/attestation_0/ssz_snappy";
        let decoded = try_decode_attestation(topic, &enc).expect("att");
        assert_eq!(decoded.coverage, 2);
        assert_eq!(decoded.content_root, agg.hash_tree_root().unwrap());
        assert_eq!(decoded.message_root, agg.data.hash_tree_root());
        assert_eq!(content_root_for(topic, &enc), decoded.content_root);
    }

    #[test]
    fn aggregation_topic_uses_ssz_tree_root() {
        let topic = "/leanconsensus/abcd/aggregation/ssz_snappy";
        let payload = b"agg-proof-bytes";
        let expected = MultiMessageAggregate::new(payload.to_vec())
            .unwrap()
            .hash_tree_root()
            .unwrap();
        assert_eq!(content_root_for(topic, payload), expected);
        assert_ne!(
            content_root_for(topic, payload),
            provisional_content_root(payload)
        );
    }

    #[test]
    fn unknown_topic_uses_provisional_hash() {
        let topic = "/leanconsensus/abcd/unknown/ssz_snappy";
        let payload = b"not-a-block";
        assert!(try_decode_block(topic, payload).is_none());
        assert_eq!(
            content_root_for(topic, payload),
            provisional_content_root(payload)
        );
    }
}
