use super::*;
use crate::aggregation::{ParticipantSet, ProofKind};
use crate::leanvm_ipc_frame::IpcOp;
use crate::verify_type1;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

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
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::remove_var(PROVER_ENV);
    std::env::remove_var(PROBE_ENV);
    let s = LeanVmIpcStatus::probe();
    assert!(!s.binary_present);
    assert!(s.frame_abi_ready);
    assert!(s.spawn_exchange_wired);
    assert!(!s.protocol_ready);
    assert!(!s.probe_requested);
    assert!(!s.ready());
}

#[test]
fn probe_env_without_binary_stays_not_ready() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::remove_var(PROVER_ENV);
    std::env::set_var(PROBE_ENV, "1");
    let s = LeanVmIpcStatus::probe();
    assert!(s.probe_requested);
    assert!(!s.protocol_ready);
    std::env::remove_var(PROBE_ENV);
}

#[test]
fn prove_ipc_fails_closed_without_env() {
    let _guard = ENV_LOCK.lock().unwrap();
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
    let _guard = ENV_LOCK.lock().unwrap();
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
    std::env::set_var(PROVER_ENV, &mock);
    let via_env = prove_ipc(&statement).expect("prove_ipc via mock env");
    assert_eq!(via_env, proof);
    std::env::set_var(PROBE_ENV, "1");
    let live = LeanVmIpcStatus::probe();
    assert!(live.probe_requested);
    assert!(live.protocol_ready);
    assert!(live.ready());
    std::env::remove_var(PROBE_ENV);
    std::env::remove_var(PROVER_ENV);
}

fn mock_prover_path() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop();
    p.push("target");
    p.push("debug");
    #[cfg(windows)]
    p.push("ethean-leanvm-mock.exe");
    #[cfg(not(windows))]
    p.push("ethean-leanvm-mock");
    p
}
