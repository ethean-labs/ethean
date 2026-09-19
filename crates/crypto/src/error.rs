//! Crypto error types.

use thiserror::Error;

/// Errors from the Lean crypto surface.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CryptoError {
    #[error("invalid public key length: expected {expected}, got {got}")]
    InvalidPublicKeyLength { expected: usize, got: usize },

    #[error("invalid signature length: expected {expected}, got {got}")]
    InvalidSignatureLength { expected: usize, got: usize },

    #[error("invalid message length: expected {expected}, got {got}")]
    InvalidMessageLength { expected: usize, got: usize },

    #[error("signature verification failed")]
    VerificationFailed,

    #[error("signing failed: {0}")]
    SigningFailed(String),

    #[error("key generation failed: {0}")]
    KeyGenerationFailed(String),

    #[error("XMSS backend unavailable: {0}")]
    BackendUnavailable(&'static str),

    #[error("unsupported epoch or exhausted lifetime")]
    LifetimeExhausted,

    #[error("aggregate API deferred to Phase 08")]
    AggregateDeferred,

    #[error("invalid aggregate statement or proof: {0}")]
    InvalidAggregate(String),

    #[error("parameter fingerprint mismatch")]
    FingerprintMismatch,
}

/// Result alias for this crate.
pub type Result<T> = std::result::Result<T, CryptoError>;
