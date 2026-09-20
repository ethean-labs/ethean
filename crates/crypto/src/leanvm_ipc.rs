//! leanVM process-isolated prover IPC status (feature `leanvm-backend`).
//!
//! Phase 08 requires proving outside the chain-owner process. Frame codec is
//! ready ([`crate::leanvm_ipc_frame`]); spawn stays fail-closed until a sandbox
//! round-trip against the pinned leanVM rev lands.

use crate::aggregation::AggregateStatement;
use crate::error::{CryptoError, Result};
use crate::leanvm_ipc_frame::{IpcFrame, FRAME_CODEC_READY};
use std::path::PathBuf;

/// Env var naming a candidate leanVM prover executable (observability only today).
pub const PROVER_ENV: &str = "ETHEAN_LEANVM_PROVER";

/// Snapshot of process-prover wiring for gates / boot logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanVmIpcStatus {
    /// Absolute or configured path when the env var is set.
    pub binary_path: Option<PathBuf>,
    /// True when the path exists on disk (does not imply a compatible ABI).
    pub binary_present: bool,
    /// Versioned request/response frame codec is compiled in.
    pub frame_abi_ready: bool,
    /// Framed prove/verify spawn + round-trip is implemented and version-checked.
    pub protocol_ready: bool,
}

impl LeanVmIpcStatus {
    /// Probe env + filesystem; never executes the binary.
    pub fn probe() -> Self {
        let binary_path = std::env::var_os(PROVER_ENV).map(PathBuf::from);
        let binary_present = binary_path
            .as_ref()
            .map(|p| p.is_file())
            .unwrap_or(false);
        Self {
            binary_path,
            binary_present,
            frame_abi_ready: FRAME_CODEC_READY,
            // Flip only after spawn + round-trip against LEANVM_REV.
            protocol_ready: false,
        }
    }

    /// True when production IPC prove/verify may run.
    pub fn ready(&self) -> bool {
        self.binary_present && self.protocol_ready
    }
}

/// Prove via process IPC (fail closed until protocol_ready).
pub fn prove_ipc(statement: &AggregateStatement) -> Result<Vec<u8>> {
    statement.validate_shape()?;
    let status = LeanVmIpcStatus::probe();
    if !status.binary_present {
        return Err(CryptoError::BackendUnavailable(
            "leanVM prover binary not configured (set ETHEAN_LEANVM_PROVER)",
        ));
    }
    // Always build the framed request so the statement wire path is exercised.
    let _request = IpcFrame::prove_request(statement)?.encode()?;
    if !status.protocol_ready {
        return Err(CryptoError::BackendUnavailable(
            "leanVM IPC frame ready but spawn/round-trip not wired; refuse process spawn",
        ));
    }
    Err(CryptoError::BackendUnavailable(
        "leanVM IPC protocol_ready but prove handler not wired",
    ))
}

/// Verify via process IPC (fail closed until protocol_ready).
pub fn verify_ipc(statement: &AggregateStatement, proof: &[u8]) -> Result<bool> {
    statement.validate_shape()?;
    let status = LeanVmIpcStatus::probe();
    let _request = IpcFrame::verify_request(statement, proof)?.encode()?;
    if !status.ready() {
        return Err(CryptoError::BackendUnavailable(
            "leanVM IPC not ready; refuse always-true verify",
        ));
    }
    Err(CryptoError::BackendUnavailable(
        "leanVM IPC protocol_ready but verify handler not wired",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregation::{ParticipantSet, ProofKind};

    #[test]
    fn probe_without_env_is_not_ready() {
        std::env::remove_var(PROVER_ENV);
        let s = LeanVmIpcStatus::probe();
        assert!(!s.binary_present);
        assert!(s.frame_abi_ready);
        assert!(!s.protocol_ready);
        assert!(!s.ready());
    }

    #[test]
    fn prove_ipc_fails_closed() {
        std::env::remove_var(PROVER_ENV);
        let statement = AggregateStatement {
            kind: ProofKind::Type1,
            profile_digest: [1u8; 32],
            message_root: [2u8; 32],
            slot: 1,
            participants: ParticipantSet::try_from_ordered(vec![0]).unwrap(),
            components: vec![],
        };
        assert!(prove_ipc(&statement).is_err());
    }
}
