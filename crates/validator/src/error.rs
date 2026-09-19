//! Signer error types.

use thiserror::Error;

/// Durable signer failures (never soft-accept conflicting duties).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SignerError {
    #[error("conflicting duty for the same role and slot")]
    ConflictingDuty,

    #[error("cross-role key use is forbidden")]
    CrossRoleKeyUse,

    #[error("key not found")]
    KeyNotFound,

    #[error("key exhausted or outside activation window")]
    LifetimeExhausted,

    #[error("reservation was not flushed; refusing to sign")]
    ReservationNotDurable,

    #[error("uncertain leaf after crash; leaf burned")]
    LeafBurned,

    #[error("crypto backend error: {0}")]
    Crypto(String),

    #[error("store error: {0}")]
    Store(String),
}

/// Result alias for this crate.
pub type Result<T> = std::result::Result<T, SignerError>;
