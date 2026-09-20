//! Dev/CI mock leanVM prover: speaks length-prefixed ELVM frames on stdio.
//!
//! Not a production SNARK prover. Uses `ethean-crypto` test-aggregate bindings
//! so local IPC spawn round-trips can go green without linking Plonky3.

use ethean_crypto::{
    encode_len_prefixed, encode_type1_leaves, prove_type1, prove_type2, read_len_prefixed,
    split_type2_to_type1, verify_type1, verify_type2, AggregateStatement, IpcFrame, IpcOp,
    ProofKind, LEANVM_REV,
};
use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    // Defense in depth: never re-enter process IPC from inside the mock.
    std::env::remove_var("ETHEAN_LEANVM_PROVER");
    std::env::remove_var("ETHEAN_LEANVM_IPC_PROBE");
    std::env::set_var("ETHEAN_LEANVM_IPC_SERVING", "1");
    match serve_once() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ethean-leanvm-mock: {e}");
            ExitCode::FAILURE
        }
    }
}

fn serve_once() -> Result<(), String> {
    let mut stdin = io::stdin().lock();
    let req_bytes = read_len_prefixed(&mut stdin).map_err(|e| e.to_string())?;
    let request = IpcFrame::decode(&req_bytes).map_err(|e| e.to_string())?;
    if request.pin_rev != LEANVM_REV {
        return Err(format!(
            "pin mismatch: got {}, want {LEANVM_REV}",
            request.pin_rev
        ));
    }
    let statement =
        AggregateStatement::decode_wire(&request.statement).map_err(|e| e.to_string())?;
    let response = match request.op {
        IpcOp::ProveRequest => handle_prove(statement)?,
        IpcOp::VerifyRequest => handle_verify(statement, &request.proof)?,
        IpcOp::SplitRequest => handle_split(statement, &request.proof)?,
        other => return Err(format!("unsupported op {other:?}")),
    };
    let encoded = response.encode().map_err(|e| e.to_string())?;
    let framed = encode_len_prefixed(&encoded).map_err(|e| e.to_string())?;
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(&framed)
        .map_err(|e| format!("stdout write: {e}"))?;
    stdout.flush().map_err(|e| format!("stdout flush: {e}"))?;
    Ok(())
}

fn handle_prove(statement: AggregateStatement) -> Result<IpcFrame, String> {
    let proof = match statement.kind {
        ProofKind::Type1 => prove_type1(&statement).map_err(|e| e.to_string())?,
        ProofKind::Type2 => prove_type2(&statement).map_err(|e| e.to_string())?,
    };
    Ok(IpcFrame {
        op: IpcOp::ProveResponse,
        pin_rev: LEANVM_REV.to_string(),
        statement: statement.encode_wire(),
        proof,
        ok: true,
    })
}

fn handle_verify(statement: AggregateStatement, proof: &[u8]) -> Result<IpcFrame, String> {
    let ok = match statement.kind {
        ProofKind::Type1 => verify_type1(&statement, proof).is_ok(),
        ProofKind::Type2 => verify_type2(&statement, proof).is_ok(),
    };
    Ok(IpcFrame {
        op: IpcOp::VerifyResponse,
        pin_rev: LEANVM_REV.to_string(),
        statement: statement.encode_wire(),
        proof: proof.to_vec(),
        ok,
    })
}

fn handle_split(statement: AggregateStatement, type2_proof: &[u8]) -> Result<IpcFrame, String> {
    // Structural-only: crypto_split stays false until a real leanVM decomposes SNARKs.
    let leaves = split_type2_to_type1(&statement, type2_proof).map_err(|e| e.to_string())?;
    Ok(IpcFrame {
        op: IpcOp::SplitResponse,
        pin_rev: LEANVM_REV.to_string(),
        statement: statement.encode_wire(),
        proof: encode_type1_leaves(&leaves),
        ok: true,
    })
}
