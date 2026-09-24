//! Wire codec and topic errors.

use thiserror::Error;

/// Errors from Lean network wire codecs and topic helpers.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WireError {
    #[error("invalid topic: {0}")]
    InvalidTopic(String),

    #[error("snappy: {0}")]
    Snappy(String),

    #[error("payload exceeds limit: got {got}, max {max}")]
    PayloadTooLarge { got: usize, max: usize },

    #[error("invalid status: {0}")]
    InvalidStatus(String),

    #[error("invalid req/resp: {0}")]
    InvalidReqResp(String),

    #[error("trailing bytes after decode")]
    TrailingBytes,

    #[error("varint: {0}")]
    Varint(String),

    #[error("codec: {0}")]
    Codec(String),
}

/// Result alias for this crate.
pub type Result<T> = std::result::Result<T, WireError>;
