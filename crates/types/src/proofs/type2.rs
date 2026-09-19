//! Type-2 (block envelope) statement helpers.

use ethean_primitives::Hash32;

use crate::aggregate::MultiMessageAggregate;
use crate::block::Block;
use crate::error::TypesError;
use crate::limits::{BYTE_LIST_512_KIB, MAX_ATTESTATIONS_DATA};

/// Component message roots for Type-2: proposer first, then attestation data roots in body order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type2ComponentRoots {
    /// Proposer signing root (first component).
    pub proposer_message: Hash32,
    /// Attestation-data roots in block body order (≤ MAX_ATTESTATIONS_DATA).
    pub attestation_messages: Vec<Hash32>,
}

impl Type2ComponentRoots {
    /// Total component count including proposer.
    pub fn len(&self) -> usize {
        1 + self.attestation_messages.len()
    }

    /// True when only proposer is present.
    pub fn is_empty_attestations(&self) -> bool {
        self.attestation_messages.is_empty()
    }
}

/// Validate a Type-2 envelope against block-derived component list.
///
/// Does not verify SNARK bytes — only size and component cardinality bounds.
pub fn type2_statement_for_block(
    block: &Block,
    envelope: &MultiMessageAggregate,
    proposer_message: Hash32,
    attestation_messages: &[Hash32],
) -> Result<Type2ComponentRoots, TypesError> {
    if envelope.proof.is_empty() {
        return Err(TypesError::InvalidContainer(
            "Type-2 proof bytes are empty".into(),
        ));
    }
    if envelope.proof.len() > BYTE_LIST_512_KIB {
        return Err(TypesError::BytesTooLong {
            got: envelope.proof.len(),
            max: BYTE_LIST_512_KIB,
        });
    }
    if attestation_messages.len() > MAX_ATTESTATIONS_DATA {
        return Err(TypesError::ListTooLong {
            got: attestation_messages.len(),
            limit: MAX_ATTESTATIONS_DATA,
        });
    }
    if attestation_messages.len() != block.body.attestations.len() {
        return Err(TypesError::InvalidContainer(
            "Type-2 attestation component count must match block body".into(),
        ));
    }
    let _ = block.slot;
    Ok(Type2ComponentRoots {
        proposer_message,
        attestation_messages: attestation_messages.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::MultiMessageAggregate;
    use crate::block::{Block, BlockBody, BlockHeader};
    use ethean_primitives::{Hash32, HASH32_ZERO};

    #[test]
    fn rejects_empty_proof() {
        let body = BlockBody::new(vec![]).unwrap();
        let header = BlockHeader {
            slot: 1,
            proposer_index: 0,
            parent_root: HASH32_ZERO,
            state_root: HASH32_ZERO,
            body_root: body.hash_tree_root().unwrap(),
        };
        let block = Block { slot: 1, header, body };
        let env = MultiMessageAggregate::default();
        assert!(type2_statement_for_block(&block, &env, HASH32_ZERO, &[]).is_err());
    }
}
