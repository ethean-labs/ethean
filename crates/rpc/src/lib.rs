//! HTTP / JSON-RPC surface (Phase 12).

#![forbid(unsafe_code)]

pub mod error;

pub use error::{Result, RpcError};

/// Placeholder RPC facade.
#[derive(Debug, Default, Clone, Copy)]
pub struct RpcFacade;
