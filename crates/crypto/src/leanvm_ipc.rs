//! leanVM process-isolated prover IPC status (feature `leanvm-backend`).
//!
//! Frame codec + spawn exchange are wired. `protocol_ready` stays false until a
//! pin-checked round-trip against a real leanVM binary succeeds in CI/ops.

use crate::aggregation::{AggregateStatement, LEANVM_REV};
use crate::error::{CryptoError, Result};
use crate::leanvm_ipc_frame::{IpcFrame, IpcOp, FRAME_CODEC_READY};
use crate::leanvm_ipc_spawn::{exchange_frame, SPAWN_EXCHANGE_WIRED, DEFAULT_IPC_WALL};
use std::path::PathBuf;

/// Env var naming a candidate leanVM prover executable.
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
    /// Length-prefixed spawn exchange is compiled in (still fail-closed on bad peers).
    pub spawn_exchange_wired: bool,
    /// Live pin-checked round-trip against leanVM succeeded (ops/CI sets this path later).
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
            spawn_exchange_wired: SPAWN_EXCHANGE_WIRED,
            // Flip only after a successful pin-checked round-trip in ops/CI.
            protocol_ready: false,
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
/// Does not flip [`LeanVmIpcStatus::protocol_ready`]; callers decide trust policy.
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
mod tests {
    use super::*;
    use crate::aggregation::{ParticipantSet, ProofKind};
    use crate::leanvm_ipc_frame::IpcOp;
    use crate::verify_type1;

    fn sample() -> AggregateStatement {
        AggregateStatement {
            kind: ProofKind::Type1,
            profile_digest: [1u8; 32],
            message_root: [2u8; 32],
            slot: 1,
            participants: ParticipantSet::try_from_ordered(vec![0]).unwrap(),
            components: vec![],
        }
    }

    #[test]
    fn probe_without_env_is_not_ready() {
        std::env::remove_var(PROVER_ENV);
        let s = LeanVmIpcStatus::probe();
        assert!(!s.binary_present);
        assert!(s.frame_abi_ready);
        assert!(s.spawn_exchange_wired);
        assert!(!s.protocol_ready);
        assert!(!s.ready());
    }

    #[test]
    fn prove_ipc_fails_closed_without_env() {
        std::env::remove_var(PROVER_ENV);
        assert!(prove_ipc(&sample()).is_err());
    }

    #[test]
    fn accept_prove_response_checks_pin_and_op() {
        let req = IpcFrame::prove_request(&sample()).unwrap();
        let mut bad = req.clone();
        bad.op = IpcOp::ProveResponse;
        bad.ok = true;
        bad.proof = vec![1];
        bad.pin_rev = "0".repeat(40);
        assert!(accept_prove_response(&req, bad).is_err());

        let good = IpcFrame {
            op: IpcOp::ProveResponse,
            pin_rev: LEANVM_REV.to_string(),
            statement: req.statement.clone(),
            proof: vec![9, 9],
            ok: true,
        };
        assert_eq!(accept_prove_response(&req, good).unwrap(), vec![9, 9]);
    }

    #[test]
    fn roundtrip_against_workspace_mock_if_built() {
        let mock = mock_prover_path();
        if !mock.is_file() {
            eprintln!("skip: build ethean-leanvm-mock first ({})", mock.display());
            return;
        }
        let proof = try_roundtrip_prove(&mock).expect("mock prove round-trip");
        assert!(!proof.is_empty());
        let statement = AggregateStatement {
            kind: ProofKind::Type1,
            profile_digest: [9u8; 32],
            message_root: [8u8; 32],
            slot: 1,
            participants: ParticipantSet::try_from_ordered(vec![0, 1]).unwrap(),
            components: vec![],
        };
        verify_type1(&statement, &proof).expect("mock proof verifies");
        // Also exercise env-based prove_ipc path.
        std::env::set_var(PROVER_ENV, &mock);
        let via_env = prove_ipc(&statement).expect("prove_ipc via mock env");
        assert_eq!(via_env, proof);
        std::env::remove_var(PROVER_ENV);
    }

    fn mock_prover_path() -> std::path::PathBuf {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.pop(); // crates
        p.pop(); // repo root
        p.push("target");
        p.push("debug");
        #[cfg(windows)]
        p.push("ethean-leanvm-mock.exe");
        #[cfg(not(windows))]
        p.push("ethean-leanvm-mock");
        p
    }
}
