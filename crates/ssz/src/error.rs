//! SSZ encode/decode and merkleization errors.

use thiserror::Error;

/// Errors raised by the minimal SSZ codec.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SszError {
    #[error("buffer too short: need {need} bytes, have {have}")]
    BufferTooShort { need: usize, have: usize },

    #[error("unexpected trailing bytes: {0}")]
    TrailingBytes(usize),

    #[error("invalid boolean byte: {0}")]
    InvalidBool(u8),

    #[error("list length {got} exceeds limit {limit}")]
    ListTooLong { got: usize, limit: usize },

    #[error("offset out of range: {offset} (payload {payload_len})")]
    OffsetOutOfRange { offset: usize, payload_len: usize },

    #[error("offsets not strictly increasing")]
    OffsetsNotIncreasing,

    #[error("offset before fixed section end: {offset} < {fixed_end}")]
    OffsetBeforeFixed { offset: usize, fixed_end: usize },

    #[error("empty offset list for variable elements")]
    EmptyOffsetList,

    #[error("byte length {got} exceeds limit {limit}")]
    BytesTooLong { got: usize, limit: usize },

    #[error("invalid bitlist encoding")]
    InvalidBitlist,
}
