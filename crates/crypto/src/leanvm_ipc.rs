//! leanVM process-isolated prover IPC status (feature `leanvm-backend`).
//!
//! Frame codec + spawn exchange are wired. `protocol_ready` becomes true only after
//! an explicit live probe (`ETHEAN_LEANVM_IPC_PROBE`) succeeds against the configured
//! binary (ops should point that at a pin-trusted leanVM, not only the mock).

use crate::aggregation::{AggregateStatement, LEANVM_REV};
use crate::error::{CryptoError, Result};
use crate::leanvm_ipc_frame::{IpcFrame, IpcOp, FRAME_CODEC_READY};
use crate::leanvm_ipc_spawn::{exchange_frame, SPAWN_EXCHANGE_WIRED, DEFAULT_IPC_WALL};
use std::path::PathBuf;

/// Env var naming a candidate leanVM prover executable.
pub const PROVER_ENV: &str = "ETHEAN_LEANVM_PROVER";

/// When truthy (`1`/`true`/`yes`/`on`), [`LeanVmIpcStatus::probe`] runs one pin-checked
/// prove round-trip and sets `protocol_ready` on success.
pub const PROBE_ENV: &str = "ETHEAN_LEANVM_IPC_PROBE";

fn env_flag(name: &str) -> bool {
    match std::env::var(name) {
        Ok(v) => matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

/// Snapshot of process-prover wiring for gates / boot logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanVmIpcStatus {
    /// Absolute or configured path when the env var is set.
    pub binary_path: Option<PathBuf>,
    /// True when the path exists on disk (does not imply a compatible ABI).
    pub binary_present: bool,
    /// Versioned request/response frame codec is compiled in.
    pub frame_abi_ready: bool,
    /// Length-prefixed spawn exchange is compiled in (still fail-closed on bad peers).
    pub spawn_exchange_wired: bool,
    /// Operator requested a live pin-checked round-trip during probe.
    pub probe_requested: bool,
    /// Live pin-checked round-trip against the configured binary succeeded.
    pub protocol_ready: bool,
}

impl LeanVmIpcStatus {
    /// Probe env + filesystem; executes the binary only when [`PROBE_ENV`] is set.
    pub fn probe() -> Self {
        let binary_path = std::env::var_os(PROVER_ENV).map(PathBuf::from);
        let binary_present = binary_path
            .as_ref()
            .map(|p| p.is_file())
            .unwrap_or(false);
        let probe_requested = env_flag(PROBE_ENV);
        let protocol_ready = if binary_present && probe_requested {
            binary_path
                .as_ref()
                .map(|p| try_roundtrip_prove(p).is_ok())
                .unwrap_or(false)
        } else {
            false
        };
        Self {
            binary_path,
            binary_present,
            frame_abi_ready: FRAME_CODEC_READY,
            spawn_exchange_wired: SPAWN_EXCHANGE_WIRED,
            probe_requested,
            protocol_ready,
        }
    }

    /// True when production IPC prove/verify may be treated as green at boot.
    pub fn ready(&self) -> bool {
        self.binary_present && self.protocol_ready
    }
}

/// Prove via process IPC (attempts spawn when binary present; fail-closed otherwise).
pub fn prove_ipc(statement: &AggregateStatement) -> Result<Vec<u8>> {
    statement.validate_shape()?;
    let status = LeanVmIpcStatus::probe();
    let Some(path) = status.binary_path.as_ref() else {
        return Err(CryptoError::BackendUnavailable(
            "leanVM prover binary not configured (set ETHEAN_LEANVM_PROVER)",
        ));
    };
    if !status.binary_present {
        return Err(CryptoError::BackendUnavailable(
            "leanVM prover binary not configured (set ETHEAN_LEANVM_PROVER)",
        ));
    }
    if !status.spawn_exchange_wired {
        return Err(CryptoError::BackendUnavailable(
            "leanVM IPC spawn exchange not wired",
        ));
    }
    let request = IpcFrame::prove_request(statement)?;
    let response = exchange_frame(path, &request, DEFAULT_IPC_WALL)?;
    accept_prove_response(&request, response)
}

/// Verify via process IPC (attempts spawn when binary present; fail-closed otherwise).
pub fn verify_ipc(statement: &AggregateStatement, proof: &[u8]) -> Result<bool> {
    statement.validate_shape()?;
    let status = LeanVmIpcStatus::probe();
    let Some(path) = status.binary_path.as_ref() else {
        return Err(CryptoError::BackendUnavailable(
            "leanVM IPC not ready; refuse always-true verify",
        ));
    };
    if !status.binary_present || !status.spawn_exchange_wired {
        return Err(CryptoError::BackendUnavailable(
            "leanVM IPC not ready; refuse always-true verify",
        ));
    }
    let request = IpcFrame::verify_request(statement, proof)?;
    let response = exchange_frame(path, &request, DEFAULT_IPC_WALL)?;
    accept_verify_response(&request, response)
}

fn accept_prove_response(request: &IpcFrame, response: IpcFrame) -> Result<Vec<u8>> {
    if response.op != IpcOp::ProveResponse {
        return Err(CryptoError::InvalidAggregate(
            "leanVM IPC expected ProveResponse".into(),
        ));
    }
    if response.pin_rev != LEANVM_REV {
        return Err(CryptoError::InvalidAggregate(
            "leanVM IPC prove response pin mismatch".into(),
        ));
    }
    if response.statement != request.statement {
        return Err(CryptoError::InvalidAggregate(
            "leanVM IPC prove response statement mismatch".into(),
        ));
    }
    if !response.ok || response.proof.is_empty() {
        return Err(CryptoError::InvalidAggregate(
            "leanVM IPC prove response not ok or empty proof".into(),
        ));
    }
    Ok(response.proof)
}

fn accept_verify_response(request: &IpcFrame, response: IpcFrame) -> Result<bool> {
    if response.op != IpcOp::VerifyResponse {
        return Err(CryptoError::InvalidAggregate(
            "leanVM IPC expected VerifyResponse".into(),
        ));
    }
    if response.pin_rev != LEANVM_REV {
        return Err(CryptoError::InvalidAggregate(
            "leanVM IPC verify response pin mismatch".into(),
        ));
    }
    if response.statement != request.statement || response.proof != request.proof {
        return Err(CryptoError::InvalidAggregate(
            "leanVM IPC verify response binding mismatch".into(),
        ));
    }
    Ok(response.ok)
}

/// Attempt one Type-1 prove round-trip against `prover` (dev/CI helper).
///
/// Also used by [`LeanVmIpcStatus::probe`] when [`PROBE_ENV`] is set.
pub fn try_roundtrip_prove(prover: &std::path::Path) -> Result<Vec<u8>> {
    use crate::aggregation::{ParticipantSet, ProofKind};
    let statement = AggregateStatement {
        kind: ProofKind::Type1,
        profile_digest: [9u8; 32],
        message_root: [8u8; 32],
        slot: 1,
        participants: ParticipantSet::try_from_ordered(vec![0, 1]).unwrap(),
        components: vec![],
    };
    let request = IpcFrame::prove_request(&statement)?;
    let response = exchange_frame(prover, &request, DEFAULT_IPC_WALL)?;
    accept_prove_response(&request, response)
}

#[cfg(test)]
#[path = "leanvm_ipc_tests.rs"]
mod tests;
