//! leanVM aggregate backend surface (feature `leanvm-backend`).
//!
//! leanVM is a separate workspace (`lean-multisig` / crates `lean_vm`, `lean_prover`,
//! `rec_aggregation`). It is not yet linked as a Cargo dependency here: edition
//! 2024, Plonky3, and process-sandbox prover ownership remain open gates.
//!
//! Flip [`LEANVM_FFI_LINKED`] only when prove/verify call real linked symbols.

use crate::aggregation::{AggregateStatement, LEANVM_REV};
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
    Err(CryptoError::BackendUnavailable(
        "leanVM FFI not linked (pin e2592df4); refuse fake proofs",
    ))
}

/// Verify an aggregate proof via leanVM (fail closed until linked).
pub fn verify(statement: &AggregateStatement, proof: &[u8]) -> Result<bool> {
    let _ = (statement, proof);
    Err(CryptoError::BackendUnavailable(
        "leanVM FFI not linked (pin e2592df4); refuse always-true verify",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_is_recorded() {
        assert_eq!(pinned_rev().len(), 40);
        assert!(!LEANVM_FFI_LINKED);
        assert!(pinned_rev().starts_with("e2592df4"));
    }
}
