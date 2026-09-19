//! Sync errors.

use thiserror::Error;

/// Sync and checkpoint failures.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SyncError {
    #[error("parent sync depth exceeded")]
    DepthExceeded,

    #[error("range request invalid: {0}")]
    InvalidRange(String),

    #[error("checkpoint untrusted: {0}")]
    UntrustedCheckpoint(String),

    #[error("checkpoint structure invalid: {0}")]
    InvalidCheckpoint(String),

    #[error("peer rotated after failure")]
    PeerRotated,

    #[error("backfill refused to mutate head or signer state")]
    BackfillInvariant,
}

/// Result alias for this crate.
pub type Result<T> = std::result::Result<T, SyncError>;
