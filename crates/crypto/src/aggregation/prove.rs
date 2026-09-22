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

/// Prefer FFI → process IPC (when `ETHEAN_LEANVM_PROVER` exists) → test-aggregate.
///
/// Spawned prover children set `ETHEAN_LEANVM_IPC_SERVING` so they never re-enter IPC
/// (avoids mock/`prove_type1` recursion).
fn prove_bound(statement: &AggregateStatement) -> Result<Vec<u8>> {
    #[cfg(feature = "leanvm-backend")]
    if crate::backend_leanvm::LEANVM_FFI_LINKED {
        return crate::backend_leanvm::prove(statement);
    }

    let serving = std::env::var_os(crate::leanvm_ipc_spawn::SERVING_ENV).is_some();
    let ipc = crate::leanvm_ipc::LeanVmIpcStatus::probe();
    if !serving && ipc.binary_present {
        return crate::leanvm_ipc::prove_ipc(statement);
    }

    #[cfg(any(test, feature = "test-aggregate"))]
    {
        Ok(crate::aggregation::verify::test_proof_bytes(statement))
    }
    #[cfg(not(any(test, feature = "test-aggregate")))]
    {
        let _ = statement;
        Err(CryptoError::BackendUnavailable(
            "leanVM prover unavailable; refuse fake aggregate production",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregation::statement::{ParticipantSet, ProofKind};

    fn type1() -> AggregateStatement {
        AggregateStatement {
            kind: ProofKind::Type1,
            profile_digest: [1u8; 32],
            message_root: [2u8; 32],
            slot: 3,
            participants: ParticipantSet::try_from_ordered(vec![0]).unwrap(),
            components: vec![],
        }
    }

    #[test]
    fn without_prover_env_uses_test_aggregate_when_enabled() {
        std::env::remove_var(crate::leanvm_ipc::PROVER_ENV);
        #[cfg(any(test, feature = "test-aggregate"))]
        {
            let proof = prove_type1(&type1()).expect("test-aggregate prove");
            assert!(!proof.is_empty());
        }
        #[cfg(not(any(test, feature = "test-aggregate")))]
        {
            assert!(prove_type1(&type1()).is_err());
        }
    }
}
