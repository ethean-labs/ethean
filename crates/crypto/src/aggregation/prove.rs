//! Aggregate proof production (fail-closed without leanVM).

use crate::aggregation::statement::AggregateStatement;
use crate::error::{CryptoError, Result};

/// Produce a Type-1 proof for `statement`.
pub fn prove_type1(statement: &AggregateStatement) -> Result<Vec<u8>> {
    if statement.kind != crate::aggregation::statement::ProofKind::Type1 {
        return Err(CryptoError::InvalidAggregate(
            "prove_type1 requires Type-1 statement".into(),
        ));
    }
    statement.validate_shape()?;
    prove_bound(statement)
}

/// Produce a Type-2 proof for `statement`.
pub fn prove_type2(statement: &AggregateStatement) -> Result<Vec<u8>> {
    if statement.kind != crate::aggregation::statement::ProofKind::Type2 {
        return Err(CryptoError::InvalidAggregate(
            "prove_type2 requires Type-2 statement".into(),
        ));
    }
    statement.validate_shape()?;
    prove_bound(statement)
}

fn prove_bound(statement: &AggregateStatement) -> Result<Vec<u8>> {
    #[cfg(all(feature = "leanvm-backend", feature = "test-aggregate"))]
    {
        if crate::backend_leanvm::LEANVM_FFI_LINKED {
            return crate::backend_leanvm::prove(statement);
        }
        return Ok(crate::aggregation::verify::test_proof_bytes(statement));
    }
    #[cfg(all(feature = "leanvm-backend", not(feature = "test-aggregate")))]
    {
        return crate::backend_leanvm::prove(statement);
    }
    #[cfg(all(not(feature = "leanvm-backend"), feature = "test-aggregate"))]
    {
        Ok(crate::aggregation::verify::test_proof_bytes(statement))
    }
    #[cfg(all(not(feature = "leanvm-backend"), not(feature = "test-aggregate")))]
    {
        let _ = statement;
        Err(CryptoError::BackendUnavailable(
            "leanVM prover unavailable; refuse fake aggregate production",
        ))
    }
}
