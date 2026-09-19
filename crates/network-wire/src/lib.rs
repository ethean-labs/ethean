//! Wire codecs and gossip topic schemas (Phase 10).

#![forbid(unsafe_code)]

pub mod error;

pub use error::{Result, WireError};

/// Placeholder wire facade.
#[derive(Debug, Default, Clone, Copy)]
pub struct WireFacade;
