//! Immutable Lean Consensus profile surface for Ethean.

#![forbid(unsafe_code)]

mod error;
mod preset;
mod profile;

pub use error::ProfileError;
pub use preset::lstar_devnet;
pub use profile::ChainProfile;
