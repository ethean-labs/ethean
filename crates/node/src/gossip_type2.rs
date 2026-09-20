//! D5 block gossip: require a single non-empty Type-2 proof envelope.

use crate::gossip_decode::DecodedBlockGossip;
use ethean_types::type2_statement_for_block;

/// Why a decoded block fails the D5 Type-2 envelope gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type2GossipReject {
    /// Payload was a bare `Block` without `SignedBlock.proof`.
    MissingSignedEnvelope,
    /// `SignedBlock.proof` bytes are empty.
    EmptyProof,
    /// Envelope shape failed `type2_statement_for_block` bounds.
    InvalidShape(String),
}

/// Validate that gossip carries exactly one Type-2 proof field on `SignedBlock`.
pub fn require_type2_envelope(
    decoded: &DecodedBlockGossip,
) -> Result<(), Type2GossipReject> {
    let Some(signed) = decoded.signed.as_ref() else {
        return Err(Type2GossipReject::MissingSignedEnvelope);
    };
    if signed.proof.proof.is_empty() {
        return Err(Type2GossipReject::EmptyProof);
    }
    let proposer_message = signed
        .block
        .hash_tree_root()
        .map_err(|e| Type2GossipReject::InvalidShape(e.to_string()))?;
    let attestation_messages: Vec<_> = signed
        .block
        .body
        .attestations
        .iter()
        .map(|a| a.data.hash_tree_root())
        .collect();
    type2_statement_for_block(
        &signed.block,
        &signed.proof,
        proposer_message,
        &attestation_messages,
    )
    .map_err(|e| Type2GossipReject::InvalidShape(e.to_string()))?;
    Ok(())
}

/// True when the decoded gossip is eligible for verified Type-2 import.
pub fn has_type2_envelope(decoded: &DecodedBlockGossip) -> bool {
    require_type2_envelope(decoded).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Slot, ValidatorIndex};
    use ethean_types::{Block, BlockBody, MultiMessageAggregate, SignedBlock};

    fn sample_block() -> Block {
        Block {
            slot: Slot::new(4),
            proposer_index: ValidatorIndex::new(0),
            parent_root: [1u8; 32],
            state_root: [2u8; 32],
            body: BlockBody::default(),
        }
    }

    #[test]
    fn rejects_bare_block() {
        let block = sample_block();
        let decoded = DecodedBlockGossip {
            root: block.hash_tree_root().unwrap(),
            parent: block.parent_root,
            block,
            signed: None,
            proposer_signature: None,
        };
        assert_eq!(
            require_type2_envelope(&decoded),
            Err(Type2GossipReject::MissingSignedEnvelope)
        );
    }

    #[test]
    fn rejects_empty_proof() {
        let block = sample_block();
        let signed = SignedBlock::new(block.clone(), MultiMessageAggregate::default());
        let decoded = DecodedBlockGossip {
            root: block.hash_tree_root().unwrap(),
            parent: block.parent_root,
            block,
            signed: Some(signed),
            proposer_signature: None,
        };
        assert_eq!(
            require_type2_envelope(&decoded),
            Err(Type2GossipReject::EmptyProof)
        );
    }

    #[test]
    fn accepts_non_empty_type2_proof() {
        let block = sample_block();
        let proof = MultiMessageAggregate::new(vec![9, 9, 9]).unwrap();
        let signed = SignedBlock::new(block.clone(), proof);
        let decoded = DecodedBlockGossip {
            root: block.hash_tree_root().unwrap(),
            parent: block.parent_root,
            block,
            signed: Some(signed),
            proposer_signature: None,
        };
        assert!(require_type2_envelope(&decoded).is_ok());
        assert!(has_type2_envelope(&decoded));
    }
}
