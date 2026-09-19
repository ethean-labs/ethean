//! P2P networking runtime (Phase 10).

#![forbid(unsafe_code)]

pub mod error;

pub use error::{NetworkError, Result};

/// Placeholder network facade.
#[derive(Debug, Default, Clone, Copy)]
pub struct NetworkFacade;
