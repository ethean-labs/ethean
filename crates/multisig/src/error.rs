//! Errors surfaced by aggregate proof verification and proving.

use thiserror::Error;

/// Why an aggregate proof was rejected or could not be produced.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MultisigError {
    /// Proof bytes are empty or exceed the consensus byte limit.
    #[error("proof length {len} outside 1..={max}")]
    ProofLength { len: usize, max: usize },
    /// The LZ4 size prefix announces more data than the proof can plausibly hold.
    #[error("proof declares {declared} decompressed bytes, above the {max} byte bound")]
    DecompressionBound { declared: usize, max: usize },
    /// LZ4 or postcard decoding failed, or the key sets do not fit the proof.
    #[error("proof could not be decoded")]
    Malformed,
    /// A public key is not a canonical 52-byte XMSS key.
    #[error("public key {index} is not a valid XMSS key")]
    InvalidPublicKey { index: usize },
    /// A signature is not a canonical XMSS signature.
    #[error("signature {index} is not a valid XMSS signature")]
    InvalidSignature { index: usize },
    /// A component carries no public keys.
    #[error("component {component} has no public keys")]
    EmptyComponent { component: usize },
    /// Too many components, keys or children for the aggregation circuit.
    #[error("{what}: {actual} exceeds limit {max}")]
    LimitExceeded {
        what: &'static str,
        actual: usize,
        max: usize,
    },
    /// Slot does not fit the 32-bit XMSS epoch.
    #[error("slot {0} exceeds the XMSS epoch range")]
    SlotOutOfRange(u64),
    /// The proof binds a different number of components than expected.
    #[error("proof binds {got} components, expected {expected}")]
    ComponentCount { expected: usize, got: usize },
    /// A component is bound to a different message or slot than expected.
    #[error("component {component} is bound to another message or slot")]
    BindingMismatch { component: usize },
    /// The SNARK verifier rejected the proof.
    #[error("aggregate proof rejected: {0}")]
    Rejected(String),
    /// The verifier or prover panicked on this input.
    #[error("leanMultisig panicked while processing the proof")]
    Panicked,
    /// Proving failed (bad inputs or prover error).
    #[error("proving failed: {0}")]
    ProverFailed(String),
    /// The out-of-process prover is unavailable, crashed or timed out.
    #[error("prover process: {0}")]
    ProverUnavailable(String),
}

pub type Result<T> = std::result::Result<T, MultisigError>;
