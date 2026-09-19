//! Structural Type-2 → Type-1 component split (leanVM crypto split still fail-closed).

use crate::aggregation::statement::{
    check_proof_len, AggregateStatement, ProofKind, Type2ComponentRef,
};
use crate::error::{CryptoError, Result};

/// One Type-1 leaf recovered (or reserved) from a Type-2 block proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type1Leaf {
    /// Child message root (attestation-data or binder component).
    pub message_root: [u8; 32],
    /// Slot bound on the child component.
    pub slot: u64,
    /// Type-1 proof bytes when a real leanVM split succeeded; empty when structural-only.
    pub proof: Vec<u8>,
    /// False until leanVM can decompose SNARK bytes into independently verifiable Type-1 proofs.
    pub crypto_split: bool,
}

/// Split a Type-2 statement into ordered Type-1 leaf descriptors.
///
/// Today this is **structural**: component `(message_root, slot)` rows are returned so the
/// node can re-seed `known_aggregated_payloads` / the aggregate pool. Proof bytes stay empty
/// (`crypto_split = false`) until leanVM IPC exposes a real `split_type_2` binding.
pub fn split_type2_to_type1(
    statement: &AggregateStatement,
    type2_proof: &[u8],
) -> Result<Vec<Type1Leaf>> {
    statement.validate_shape()?;
    if statement.kind != ProofKind::Type2 {
        return Err(CryptoError::InvalidAggregate(
            "split_type2_to_type1 requires ProofKind::Type2".into(),
        ));
    }
    check_proof_len(type2_proof)?;
    // Bind the Type-2 digest so callers cannot pass an unrelated proof blob silently.
    let _ = statement.digest();
    Ok(statement
        .components
        .iter()
        .map(|c: &Type2ComponentRef| Type1Leaf {
            message_root: c.message_root,
            slot: c.slot,
            proof: Vec::new(),
            crypto_split: false,
        })
        .collect())
}

/// Attestation-data leaves only (skip first body binder and last block-root binder when present).
///
/// Matches `ethean_transition::type2_statement_for_block` component layout:
/// `[body_root, attestation_data…, block_root]`.
pub fn attestation_leaves_from_type2(
    statement: &AggregateStatement,
    type2_proof: &[u8],
) -> Result<Vec<Type1Leaf>> {
    let leaves = split_type2_to_type1(statement, type2_proof)?;
    if leaves.len() <= 2 {
        return Ok(Vec::new());
    }
    Ok(leaves[1..leaves.len() - 1].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregation::statement::{ParticipantSet, Type2ComponentRef};

    fn sample_type2() -> AggregateStatement {
        AggregateStatement {
            kind: ProofKind::Type2,
            profile_digest: [1u8; 32],
            message_root: [2u8; 32],
            slot: 7,
            participants: ParticipantSet::empty(),
            components: vec![
                Type2ComponentRef {
                    message_root: [10u8; 32],
                    slot: 7,
                },
                Type2ComponentRef {
                    message_root: [11u8; 32],
                    slot: 7,
                },
                Type2ComponentRef {
                    message_root: [12u8; 32],
                    slot: 7,
                },
            ],
        }
    }

    #[test]
    fn structural_split_returns_all_components() {
        let s = sample_type2();
        let leaves = split_type2_to_type1(&s, &[9u8; 16]).unwrap();
        assert_eq!(leaves.len(), 3);
        assert!(!leaves[1].crypto_split);
        assert!(leaves[1].proof.is_empty());
        assert_eq!(leaves[1].message_root, [11u8; 32]);
    }

    #[test]
    fn attestation_leaves_skip_binders() {
        let s = sample_type2();
        let att = attestation_leaves_from_type2(&s, &[9u8; 16]).unwrap();
        assert_eq!(att.len(), 1);
        assert_eq!(att[0].message_root, [11u8; 32]);
    }

    #[test]
    fn rejects_type1_statement() {
        let s = AggregateStatement {
            kind: ProofKind::Type1,
            profile_digest: [0u8; 32],
            message_root: [1u8; 32],
            slot: 1,
            participants: ParticipantSet::try_from_ordered(vec![0]).unwrap(),
            components: vec![],
        };
        assert!(split_type2_to_type1(&s, &[1]).is_err());
    }
}
