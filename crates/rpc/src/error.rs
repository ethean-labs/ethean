//! RPC errors.

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RpcError {
    #[error("rpc not implemented (Phase 12)")]
    NotImplemented,
}

pub type Result<T> = std::result::Result<T, RpcError>;
