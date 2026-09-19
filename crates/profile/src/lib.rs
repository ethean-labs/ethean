//! Immutable Lean Consensus profile surface for Ethean.

#![forbid(unsafe_code)]

mod error;
mod fork;
mod limits;
mod preset;
mod profile;
mod validation;

pub use error::ProfileError;
pub use fork::ForkId;
pub use limits::{limits_of, ProfileLimits};
pub use preset::lstar_devnet;
pub use profile::ChainProfile;
pub use validation::{require_lstar_fork, validate_profile};
