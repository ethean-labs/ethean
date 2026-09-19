//! leanVM aggregate backend surface (feature `leanvm-backend`).
//!
//! leanVM is a separate workspace (`lean-multisig` / crates `lean_vm`, `lean_prover`,
//! `rec_aggregation`). It is not yet linked as a Cargo dependency here: edition
//! 2024, Plonky3, and process-sandbox prover ownership remain open gates.
//!
//! Flip [`LEANVM_FFI_LINKED`] only when prove/verify call real linked symbols.

use crate::aggregation::bindings::LEANVM_REV;
use crate::aggregation::statement::AggregateStatement;
use crate::error::{CryptoError, Result};

/// Must stay false until FFI / process prover symbols are linked and tested.
pub const LEANVM_FFI_LINKED: bool = false;

/// Upstream pin recorded in Phase 08 locks.
pub fn pinned_rev() -> &'static str {
    LEANVM_REV
}

/// Produce an aggregate proof via leanVM (fail closed until linked).
pub fn prove(statement: &AggregateStatement) -> Result<Vec<u8>> {
    let _ = statement;
    Err(CryptoError::BackendUnavailable(format!(
        "leanVM FFI not linked (pin {LEANVM_REV}); refuse fake proofs \
         (LEANVM_FFI_LINKED={LEANVM_FFI_LINKED})"
    )))
}

/// Verify an aggregate proof via leanVM (fail closed until linked).
pub fn verify(statement: &AggregateStatement, proof: &[u8]) -> Result<bool> {
    let _ = (statement, proof);
    Err(CryptoError::BackendUnavailable(format!(
        "leanVM FFI not linked (pin {LEANVM_REV}); refuse always-true verify \
         (LEANVM_FFI_LINKED={LEANVM_FFI_LINKED})"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_is_recorded() {
        assert_eq!(pinned_rev().len(), 40);
        assert!(!LEANVM_FFI_LINKED);
    }
}
