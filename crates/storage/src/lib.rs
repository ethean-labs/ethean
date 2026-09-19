//! Durable persistence contracts (Phase 11 implementation).

#![forbid(unsafe_code)]

pub mod error;

pub use error::{Result, StorageError};

/// Placeholder until Phase 11 tables land.
#[derive(Debug, Default, Clone, Copy)]
pub struct StorageFacade;
