//! Consensus-derived Type-2 aggregate statement for a block body.

use ethean_crypto::{
    domain_digest, AggregateStatement, ParticipantSet, ProofKind, Type2ComponentRef,
    PROD_AGGREGATION_FINGERPRINT,
};
use ethean_types::Block;

use crate::error::TransitionError;

/// Build the Type-2 statement that [`crate::apply_block`] verifies against.
pub fn type2_statement_for_block(block: &Block) -> Result<AggregateStatement, TransitionError> {
    let body_root = block
        .body
        .hash_tree_root()
        .map_err(|e| TransitionError::Types(e.to_string()))?;
    let profile = domain_digest(
        b"ethean-transition/v1/agg-profile",
        PROD_AGGREGATION_FINGERPRINT.as_bytes(),
    );
    let mut components = Vec::with_capacity(1 + block.body.attestations.len());
    components.push(Type2ComponentRef {
        message_root: body_root,
        slot: block.slot.get(),
    });
    for att in &block.body.attestations {
        components.push(Type2ComponentRef {
            message_root: att.data.hash_tree_root(),
            slot: block.slot.get(),
        });
    }
    Ok(AggregateStatement {
        kind: ProofKind::Type2,
        profile_digest: profile,
        message_root: body_root,
        slot: block.slot.get(),
        participants: ParticipantSet::empty(),
        components,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Slot, ValidatorIndex};
    use ethean_types::{BlockBody};

    #[test]
    fn empty_body_has_one_component() {
        let block = Block {
            slot: Slot::new(3),
            proposer_index: ValidatorIndex::new(0),
            parent_root: [1u8; 32],
            state_root: [2u8; 32],
            body: BlockBody::default(),
        };
        let s = type2_statement_for_block(&block).unwrap();
        assert_eq!(s.kind, ProofKind::Type2);
        assert_eq!(s.components.len(), 1);
        assert_eq!(s.slot, 3);
    }
}
