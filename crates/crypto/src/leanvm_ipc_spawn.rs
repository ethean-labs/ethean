//! Spawn a leanVM prover process and exchange one length-prefixed ELVM frame.
//!
//! Wall-clock deadline kills the child on wedge. Does **not** flip
//! `protocol_ready` — that waits for a successful pin-checked round-trip against
//! a real leanVM binary.

use crate::aggregation::MAX_PROOF_BYTES;
use crate::error::{CryptoError, Result};
use crate::leanvm_ipc_frame::IpcFrame;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Default wall budget for one prove/verify IPC exchange.
pub const DEFAULT_IPC_WALL: Duration = Duration::from_secs(30);

/// True when this crate will attempt process spawn (still fail-closed on bad peers).
pub const SPAWN_EXCHANGE_WIRED: bool = true;

/// Write `frame` to `child` stdin and read one length-prefixed response frame.
pub fn exchange_frame(
    prover: &Path,
    request: &IpcFrame,
    wall: Duration,
) -> Result<IpcFrame> {
    if !SPAWN_EXCHANGE_WIRED {
        return Err(CryptoError::BackendUnavailable(
            "leanVM IPC spawn exchange not wired",
        ));
    }
    if !prover.is_file() {
        return Err(CryptoError::BackendUnavailable(
            "leanVM prover binary missing on disk",
        ));
    }
    let req_bytes = request.encode()?;
    let mut child = Command::new(prover)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| CryptoError::InvalidAggregate(format!("leanVM spawn failed: {e}")))?;

    let start = Instant::now();
    if let Err(e) = write_request(&mut child, &req_bytes) {
        reap_kill(&mut child);
        return Err(e);
    }

    let Some(stdout) = child.stdout.take() else {
        reap_kill(&mut child);
        return Err(CryptoError::InvalidAggregate(
            "leanVM child stdout missing".into(),
        ));
    };

    let remaining = wall.saturating_sub(start.elapsed());
    match read_len_prefixed_with_deadline(stdout, remaining) {
        Ok(resp_bytes) => {
            let _ = child.wait();
            IpcFrame::decode(&resp_bytes)
        }
        Err(e) => {
            reap_kill(&mut child);
            Err(e)
        }
    }
}

fn write_request(child: &mut std::process::Child, req_bytes: &[u8]) -> Result<()> {
    let stdin = child.stdin.as_mut().ok_or_else(|| {
        CryptoError::InvalidAggregate("leanVM child stdin missing".into())
    })?;
    write_len_prefixed(stdin, req_bytes)?;
    // Close stdin so the child sees EOF after the request.
    let _ = child.stdin.take();
    Ok(())
}

fn reap_kill(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn write_len_prefixed(out: &mut impl Write, payload: &[u8]) -> Result<()> {
    let len = u32::try_from(payload.len()).map_err(|_| {
        CryptoError::InvalidAggregate("leanVM IPC frame exceeds u32 length".into())
    })?;
    out.write_all(&len.to_le_bytes())
        .map_err(|e| CryptoError::InvalidAggregate(format!("leanVM stdin write: {e}")))?;
    out.write_all(payload)
        .map_err(|e| CryptoError::InvalidAggregate(format!("leanVM stdin write: {e}")))?;
    out.flush()
        .map_err(|e| CryptoError::InvalidAggregate(format!("leanVM stdin flush: {e}")))?;
    Ok(())
}

fn read_len_prefixed_with_deadline(
    mut stdout: impl Read + Send + 'static,
    wall: Duration,
) -> Result<Vec<u8>> {
    if wall.is_zero() {
        return Err(CryptoError::InvalidAggregate(
            "leanVM IPC wall deadline exhausted before read".into(),
        ));
    }
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(read_len_prefixed(&mut stdout));
    });
    match rx.recv_timeout(wall) {
        Ok(inner) => inner,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(CryptoError::InvalidAggregate(
            "leanVM IPC wall deadline exceeded; child killed".into(),
        )),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Err(CryptoError::InvalidAggregate(
            "leanVM IPC reader thread disconnected".into(),
        )),
    }
}

/// Read one u32-LE length-prefixed payload from `input`.
pub fn read_len_prefixed(input: &mut impl Read) -> Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    input
        .read_exact(&mut len_buf)
        .map_err(|e| CryptoError::InvalidAggregate(format!("leanVM stdout length: {e}")))?;
    let len = u32::from_le_bytes(len_buf) as usize;
    let max = MAX_PROOF_BYTES.saturating_add(65_536);
    if len > max {
        return Err(CryptoError::InvalidAggregate(
            "leanVM IPC response length unreasonable".into(),
        ));
    }
    let mut payload = vec![0u8; len];
    input
        .read_exact(&mut payload)
        .map_err(|e| CryptoError::InvalidAggregate(format!("leanVM stdout payload: {e}")))?;
    Ok(payload)
}

/// Encode a length-prefixed payload (mock provers / tests).
pub fn encode_len_prefixed(payload: &[u8]) -> Result<Vec<u8>> {
    let len = u32::try_from(payload.len()).map_err(|_| {
        CryptoError::InvalidAggregate("leanVM IPC frame exceeds u32 length".into())
    })?;
    let mut out = Vec::with_capacity(4 + payload.len());
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregation::{AggregateStatement, ParticipantSet, ProofKind};
    use crate::leanvm_ipc_frame::IpcFrame;

    fn sample_stmt() -> AggregateStatement {
        AggregateStatement {
            kind: ProofKind::Type1,
            profile_digest: [3u8; 32],
            message_root: [4u8; 32],
            slot: 2,
            participants: ParticipantSet::try_from_ordered(vec![0]).unwrap(),
            components: vec![],
        }
    }

    #[test]
    fn len_prefix_roundtrip_buffer() {
        let payload = b"hello-elvm";
        let framed = encode_len_prefixed(payload).unwrap();
        assert_eq!(&framed[..4], &(payload.len() as u32).to_le_bytes());
        assert_eq!(&framed[4..], payload);
    }

    #[test]
    fn spawn_missing_binary_fails() {
        let req = IpcFrame::prove_request(&sample_stmt()).unwrap();
        let err = exchange_frame(
            Path::new("definitely-missing-ethean-leanvm-prover.bin"),
            &req,
            Duration::from_millis(200),
        );
        assert!(err.is_err());
    }

    #[cfg(windows)]
    #[test]
    fn spawn_non_prover_fails_closed() {
        let where_bin = Path::new(r"C:\Windows\System32\where.exe");
        if !where_bin.is_file() {
            return;
        }
        let req = IpcFrame::prove_request(&sample_stmt()).unwrap();
        let err = exchange_frame(where_bin, &req, Duration::from_millis(800));
        assert!(err.is_err());
    }
}
