//! Storage errors.

use thiserror::Error;

/// Durable storage failures (fail closed on corruption / schema mismatch).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StorageError {
    #[error("schema id mismatch: expected {expected}, got {got}")]
    SchemaMismatch { expected: &'static str, got: String },

    #[error("schema version mismatch: expected {expected}, got {got}")]
    SchemaVersionMismatch { expected: u32, got: u32 },

    #[error("missing key: {0}")]
    MissingKey(String),

    #[error("checksum mismatch for {0}")]
    ChecksumMismatch(String),

    #[error("batch not flushed; refusing to publish")]
    NotDurable,

    #[error("prune would remove recovery dependency: {0}")]
    PruneUnsafe(String),

    #[error("legacy JSON or Panro layout refused: {0}")]
    LegacyRefused(String),

    #[error("corruption quarantined: {0}")]
    Corruption(String),

    /// Durable backend not available (feature off or not wired).
    #[error("storage backend pending: {0}")]
    BackendPending(&'static str),
}

/// Result alias for this crate.
pub type Result<T> = std::result::Result<T, StorageError>;
