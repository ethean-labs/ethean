//! leanVM aggregate backend surface (feature `leanvm-backend`).
//!
//! leanVM lives in a separate workspace (`lean-multisig` / crates `lean_vm`,
//! `lean_prover`, `rec_aggregation`). It is not a Cargo dependency here yet:
//! edition 2024, Plonky3, and process-sandbox prover ownership remain open.
//!
//! Flip [`LEANVM_FFI_LINKED`] only after prove/verify call real linked symbols
//! and round-trip tests pass against [`LEANVM_REV`].

use crate::aggregation::{AggregateStatement, LEANVM_REV};
use crate::error::{CryptoError, Result};

/// Must stay false until FFI / process prover symbols are linked and tested.
pub const LEANVM_FFI_LINKED: bool = false;

/// Compile-time / link-time snapshot for observability (never claims ready early).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeanVmLinkStatus {
    /// Upstream pin from Phase 08 locks.
    pub pinned_rev: &'static str,
    /// `leanvm-backend` Cargo feature is compiled in.
    pub feature_enabled: bool,
    /// Real FFI symbols are linked (`LEANVM_FFI_LINKED`).
    pub ffi_linked: bool,
}

impl LeanVmLinkStatus {
    /// Probe this build; `ready` is true only when FFI is linked.
    pub fn probe() -> Self {
        Self {
            pinned_rev: LEANVM_REV,
            feature_enabled: true,
            ffi_linked: LEANVM_FFI_LINKED,
        }
    }

    /// True only when production prove/verify may run.
    pub fn ready(self) -> bool {
        self.feature_enabled && self.ffi_linked
    }
}

/// Upstream pin recorded in Phase 08 locks.
pub fn pinned_rev() -> &'static str {
    LEANVM_REV
}

/// Produce an aggregate proof via leanVM (fail closed until linked or IPC-ready).
pub fn prove(statement: &AggregateStatement) -> Result<Vec<u8>> {
    statement.validate_shape()?;
    if LEANVM_FFI_LINKED {
        return Err(CryptoError::BackendUnavailable(
            "leanVM FFI flag set but prove symbols not wired",
        ));
    }
    // Prefer process isolation (Phase 08); IPC stub still fails closed.
    crate::leanvm_ipc::prove_ipc(statement)
}

/// Verify an aggregate proof via leanVM (fail closed until linked or IPC-ready).
pub fn verify(statement: &AggregateStatement, proof: &[u8]) -> Result<bool> {
    statement.validate_shape()?;
    if LEANVM_FFI_LINKED {
        let _ = proof;
        return Err(CryptoError::BackendUnavailable(
            "leanVM FFI flag set but verify symbols not wired",
        ));
    }
    crate::leanvm_ipc::verify_ipc(statement, proof)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_is_recorded() {
        assert_eq!(pinned_rev().len(), 40);
        assert!(!LEANVM_FFI_LINKED);
        assert!(pinned_rev().starts_with("e2592df4"));
        let s = LeanVmLinkStatus::probe();
        assert!(s.feature_enabled);
        assert!(!s.ready());
    }
}
