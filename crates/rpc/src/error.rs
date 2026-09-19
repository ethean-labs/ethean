//! RPC errors (Lean API only — no Beacon compatibility).

use thiserror::Error;

/// HTTP/JSON API failures.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RpcError {
    #[error("unknown route: {0}")]
    UnknownRoute(String),

    #[error("payload too large: {got} > {max}")]
    BodyTooLarge { got: usize, max: usize },

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden: admin requires loopback or auth token")]
    Forbidden,

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("service unavailable: {0}")]
    Unavailable(String),
}

/// Result alias for this crate.
pub type Result<T> = std::result::Result<T, RpcError>;
