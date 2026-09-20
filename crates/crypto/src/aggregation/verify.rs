//! Aggregate proof verification (cheap checks then fail-closed leanVM).

use crate::aggregation::statement::{check_proof_len, AggregateStatement};
use crate::error::{CryptoError, Result};
use crate::hash::domain_digest;

/// Cheap statement + length checks (no SNARK).
pub fn verify_statement_shape(statement: &AggregateStatement, proof: &[u8]) -> Result<()> {
    statement.validate_shape()?;
    check_proof_len(proof)?;
    Ok(())
}

/// Verify a Type-1 proof against consensus-derived `statement`.
pub fn verify_type1(statement: &AggregateStatement, proof: &[u8]) -> Result<()> {
    if statement.kind != crate::aggregation::statement::ProofKind::Type1 {
        return Err(CryptoError::InvalidAggregate(
            "verify_type1 requires Type-1 statement".into(),
        ));
    }
    verify_statement_shape(statement, proof)?;
    verify_bound_proof(statement, proof)
}

/// Verify a Type-2 (block envelope) proof against consensus-derived `statement`.
pub fn verify_type2(statement: &AggregateStatement, proof: &[u8]) -> Result<()> {
    if statement.kind != crate::aggregation::statement::ProofKind::Type2 {
        return Err(CryptoError::InvalidAggregate(
            "verify_type2 requires Type-2 statement".into(),
        ));
    }
    verify_statement_shape(statement, proof)?;
    verify_bound_proof(statement, proof)
}

fn verify_bound_proof(statement: &AggregateStatement, proof: &[u8]) -> Result<()> {
    #[cfg(feature = "leanvm-backend")]
    if crate::backend_leanvm::LEANVM_FFI_LINKED {
        return match crate::backend_leanvm::verify(statement, proof)? {
            true => Ok(()),
            false => Err(CryptoError::VerificationFailed),
        };
    }

    let serving = std::env::var_os(crate::leanvm_ipc_spawn::SERVING_ENV).is_some();
    let ipc = crate::leanvm_ipc::LeanVmIpcStatus::probe();
    if !serving && ipc.binary_present {
        return match crate::leanvm_ipc::verify_ipc(statement, proof)? {
            true => Ok(()),
            false => Err(CryptoError::VerificationFailed),
        };
    }

    #[cfg(feature = "test-aggregate")]
    {
        return verify_test_aggregate(statement, proof);
    }
    #[cfg(not(feature = "test-aggregate"))]
    {
        let _ = statement;
        let _ = proof;
        Err(CryptoError::BackendUnavailable(
            "leanVM production backend unavailable; refuse always-true aggregate verify",
        ))
    }
}

#[cfg(feature = "test-aggregate")]
fn verify_test_aggregate(statement: &AggregateStatement, proof: &[u8]) -> Result<()> {
    let expected = test_proof_bytes(statement);
    if proof == expected.as_slice() {
        Ok(())
    } else {
        Err(CryptoError::VerificationFailed)
    }
}

#[cfg(feature = "test-aggregate")]
pub(crate) fn test_proof_bytes(statement: &AggregateStatement) -> Vec<u8> {
    let digest = statement.digest();
    let mut out = Vec::with_capacity(64);
    out.extend_from_slice(b"ethean-test-agg-v1");
    out.extend_from_slice(&digest);
    // Pad to a non-trivial fixed length so empty proofs stay rejected by shape checks.
    let pad = domain_digest(b"ethean-crypto/v1/test-agg-pad", &digest);
    out.extend_from_slice(&pad);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregation::statement::{ParticipantSet, ProofKind, Type2ComponentRef};
    use crate::aggregation::{prove_type1, prove_type2};

    fn type1_stmt() -> AggregateStatement {
        AggregateStatement {
            kind: ProofKind::Type1,
            profile_digest: [9u8; 32],
            message_root: [2u8; 32],
            slot: 5,
            participants: ParticipantSet::try_from_ordered(vec![0, 2, 7]).unwrap(),
            components: vec![],
        }
    }

    #[test]
    fn empty_proof_rejected() {
        let s = type1_stmt();
        assert!(verify_statement_shape(&s, &[]).is_err());
    }

    #[test]
    fn test_aggregate_roundtrip() {
        let s = type1_stmt();
        let proof = prove_type1(&s).unwrap();
        verify_type1(&s, &proof).unwrap();
        let mut bad = proof.clone();
        bad[20] ^= 0xff;
        assert!(verify_type1(&s, &bad).is_err());
    }

    #[test]
    fn type2_roundtrip() {
        let s = AggregateStatement {
            kind: ProofKind::Type2,
            profile_digest: [3u8; 32],
            message_root: [4u8; 32],
            slot: 8,
            participants: ParticipantSet::try_from_ordered(vec![1, 3]).unwrap(),
            components: vec![
                Type2ComponentRef {
                    message_root: [4u8; 32],
                    slot: 8,
                },
                Type2ComponentRef {
                    message_root: [5u8; 32],
                    slot: 8,
                },
            ],
        };
        let proof = prove_type2(&s).unwrap();
        verify_type2(&s, &proof).unwrap();
    }

    #[test]
    fn mismatched_kind_rejected() {
        let s = type1_stmt();
        let proof = prove_type1(&s).unwrap();
        assert!(verify_type2(&s, &proof).is_err());
    }
}
