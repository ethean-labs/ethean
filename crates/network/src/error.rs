//! Network runtime errors.

use thiserror::Error;

/// Errors from the Lean networking runtime.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum NetworkError {
    #[error("handshake: {0}")]
    Handshake(String),

    #[error("admission denied")]
    AdmissionDenied,

    #[error("transport not wired (QUIC Phase 10 open gate): {0}")]
    TransportPending(&'static str),

    #[error("wire: {0}")]
    Wire(String),
}

/// Result alias for this crate.
pub type Result<T> = std::result::Result<T, NetworkError>;
