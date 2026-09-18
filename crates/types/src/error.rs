//! Consensus container errors.

use thiserror::Error;

/// Errors from Lean type constructors and SSZ wrappers.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TypesError {
    #[error("list length {got} exceeds limit {limit}")]
    ListTooLong { got: usize, limit: usize },

    #[error("byte length {got} expected {expected}")]
    InvalidByteLength { got: usize, expected: usize },

    #[error("byte length {got} exceeds max {max}")]
    BytesTooLong { got: usize, max: usize },

    #[error("validator index {index} exceeds registry limit {limit}")]
    ValidatorIndexOutOfRange { index: u64, limit: u64 },

    #[error("SSZ: {0}")]
    Ssz(#[from] ethean_ssz::SszError),

    #[error("profile: {0}")]
    Profile(String),
}
