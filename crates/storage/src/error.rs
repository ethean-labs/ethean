//! Storage errors.

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StorageError {
    #[error("storage not implemented (Phase 11)")]
    NotImplemented,
}

pub type Result<T> = std::result::Result<T, StorageError>;
