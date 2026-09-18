//! Primitive conversion and arithmetic errors.

use thiserror::Error;

/// Errors produced by primitive constructors and checked arithmetic.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PrimitiveError {
    #[error("arithmetic overflow")]
    Overflow,

    #[error("arithmetic underflow")]
    Underflow,

    #[error("divisor or period must be non-zero")]
    ZeroDivisor,

    #[error("invalid byte length: expected {expected}, got {got}")]
    InvalidLength { expected: usize, got: usize },
}
