//! Sync errors.

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SyncError {
    #[error("sync not implemented (Phase 11)")]
    NotImplemented,
}

pub type Result<T> = std::result::Result<T, SyncError>;
