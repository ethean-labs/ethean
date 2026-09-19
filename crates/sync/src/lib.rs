//! Chain sync protocols (Phase 11).

#![forbid(unsafe_code)]

pub mod error;

pub use error::{Result, SyncError};

/// Placeholder sync facade.
#[derive(Debug, Default, Clone, Copy)]
pub struct SyncFacade;
