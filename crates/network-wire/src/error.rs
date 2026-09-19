//! Wire errors.

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WireError {
    #[error("network-wire not implemented (Phase 10)")]
    NotImplemented,
}

pub type Result<T> = std::result::Result<T, WireError>;
