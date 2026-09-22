//! Node-side handle to the `ethean-prover` child process.
//!
//! One request is in flight at a time (leanVM forbids concurrent proofs in a
//! process). A request that times out or hits a broken pipe kills the child;
//! the next request respawns it and repeats the revision handshake.

use std::io::BufWriter;
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::Mutex;
use std::time::Duration;

use ethean_crypto::{PublicKey, Signature};

use crate::error::{MultisigError, Result};
use crate::protocol::{read_frame, write_frame, Request, Response};
use crate::prove::KeyedProof;
use crate::LEANVM_REV;

/// Environment variable naming the prover binary.
pub const PROVER_BIN_ENV: &str = "ETHEAN_PROVER_BIN";
/// Binary name looked up next to the running executable.
pub const PROVER_BIN_NAME: &str = "ethean-prover";

/// How to launch and supervise the prover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProverConfig {
    pub binary: PathBuf,
    /// Upper bound for one prove call, including a cold start.
    pub request_timeout: Duration,
    /// Upper bound for the startup handshake.
    pub handshake_timeout: Duration,
}

impl ProverConfig {
    pub fn new(binary: PathBuf) -> Self {
        Self {
            binary,
            request_timeout: Duration::from_secs(120),
            handshake_timeout: Duration::from_secs(30),
        }
    }

    /// `$ETHEAN_PROVER_BIN`, else `ethean-prover` beside the current executable.
    pub fn discover() -> Option<Self> {
        if let Some(path) = std::env::var_os(PROVER_BIN_ENV) {
            let path = PathBuf::from(path);
            return path.is_file().then(|| Self::new(path));
        }
        let sibling = std::env::current_exe()
            .ok()?
            .parent()?
            .join(PROVER_BIN_NAME);
        sibling.is_file().then(|| Self::new(sibling))
    }
}

struct Worker {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    replies: Receiver<std::io::Result<Vec<u8>>>,
}

impl Worker {
    fn spawn(binary: &PathBuf) -> Result<Self> {
        let mut child = Command::new(binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| {
                MultisigError::ProverUnavailable(format!("spawn {}: {e}", binary.display()))
            })?;
        let stdin = BufWriter::new(child.stdin.take().expect("piped stdin"));
        let mut stdout = child.stdout.take().expect("piped stdout");
        let (tx, replies) = mpsc::channel();
        std::thread::Builder::new()
            .name("ethean-prover-reader".into())
            .spawn(move || loop {
                let frame = read_frame(&mut stdout).and_then(|f| {
                    f.ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "prover exited")
                    })
                });
                let stop = frame.is_err();
                if tx.send(frame).is_err() || stop {
                    break;
                }
            })
            .map_err(|e| MultisigError::ProverUnavailable(e.to_string()))?;
        Ok(Self {
            child,
            stdin,
            replies,
        })
    }

    fn exchange(&mut self, id: u64, request: &Request, timeout: Duration) -> Result<Response> {
        write_frame(&mut self.stdin, &request.encode(id))
            .map_err(|e| MultisigError::ProverUnavailable(format!("write: {e}")))?;
        let frame = match self.replies.recv_timeout(timeout) {
            Ok(Ok(frame)) => frame,
            Ok(Err(e)) => return Err(MultisigError::ProverUnavailable(format!("read: {e}"))),
            Err(RecvTimeoutError::Timeout) => {
                return Err(MultisigError::ProverUnavailable(format!(
                    "timed out after {timeout:?}"
                )))
            }
            Err(RecvTimeoutError::Disconnected) => {
                return Err(MultisigError::ProverUnavailable("reader stopped".into()))
            }
        };
        let (reply_id, response) = Response::decode(&frame)?;
        if reply_id != id {
            return Err(MultisigError::ProverUnavailable(
                "response id mismatch".into(),
            ));
        }
        Ok(response)
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Supervised connection to the prover process.
pub struct ProverClient {
    config: ProverConfig,
    next_id: AtomicU64,
    worker: Mutex<Option<Worker>>,
}

impl std::fmt::Debug for ProverClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProverClient")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl ProverClient {
    /// Create a client; the process starts lazily on the first request.
    pub fn new(config: ProverConfig) -> Self {
        Self {
            config,
            next_id: AtomicU64::new(1),
            worker: Mutex::new(None),
        }
    }

    pub fn config(&self) -> &ProverConfig {
        &self.config
    }

    fn call(&self, request: Request) -> Result<Vec<u8>> {
        let mut slot = self
            .worker
            .lock()
            .map_err(|_| MultisigError::ProverUnavailable("lock poisoned".into()))?;
        if slot.is_none() {
            let mut worker = Worker::spawn(&self.config.binary)?;
            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            match worker.exchange(id, &Request::Ping, self.config.handshake_timeout)? {
                Response::Pong(rev) if rev == LEANVM_REV => {}
                Response::Pong(rev) => {
                    return Err(MultisigError::ProverUnavailable(format!(
                        "prover leanVM revision {rev} does not match {LEANVM_REV}"
                    )))
                }
                other => {
                    return Err(MultisigError::ProverUnavailable(format!(
                        "bad handshake: {other:?}"
                    )))
                }
            }
            *slot = Some(worker);
        }
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let worker = slot.as_mut().expect("worker present");
        match worker.exchange(id, &request, self.config.request_timeout) {
            Ok(Response::Proof(bytes)) => Ok(bytes),
            Ok(Response::Error(msg)) => Err(MultisigError::ProverFailed(msg)),
            Ok(other) => Err(MultisigError::ProverUnavailable(format!(
                "unexpected reply {other:?}"
            ))),
            Err(e) => {
                *slot = None;
                Err(e)
            }
        }
    }

    /// Aggregate raw signatures and child proofs over one `(message, slot)`.
    pub fn aggregate_type1(
        &self,
        children: Vec<KeyedProof>,
        raw: Vec<(PublicKey, Signature)>,
        message: [u8; 32],
        slot: u64,
    ) -> Result<Vec<u8>> {
        self.call(Request::AggregateType1 {
            message,
            slot,
            raw,
            children,
        })
    }

    /// Merge ordered Type-1 proofs into a block's Type-2 proof.
    pub fn merge_type2(&self, components: Vec<KeyedProof>) -> Result<Vec<u8>> {
        self.call(Request::MergeType2 { components })
    }

    /// Split the component bound to `message` out of a Type-2 proof.
    pub fn split_type2(
        &self,
        proof: Vec<u8>,
        public_keys_per_component: Vec<Vec<PublicKey>>,
        message: [u8; 32],
    ) -> Result<Vec<u8>> {
        self.call(Request::SplitType2 {
            message,
            proof,
            public_keys_per_component,
        })
    }
}
