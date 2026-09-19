//! Ethean Lean Consensus Client — node library
//!
//! Ultra-modular Rust implementation with small, focused modules.
//! Consensus containers live in `ethean-types` (Phase 03).

pub mod crypto;
pub mod consensus;
pub mod network;
pub mod storage;
pub mod integration;
pub mod bench;
pub mod api;
pub mod config;
pub mod utils;
pub mod cli;
pub mod client;
pub mod clock;

pub use ethean_primitives::{Epoch, Hash32, Slot, ValidatorIndex};
pub use ethean_profile::{lstar_devnet, ChainProfile, ForkId};
pub use ethean_types::{
    Attestation, Block, Checkpoint, SignedBlock, State, TypesError, Validator,
};
pub use ethean_genesis::{
    load_genesis_ssz, local_smoke_genesis, BuiltGenesis, ClockError, FakeTime, GenesisBuilder,
    GenesisError, SlotClock, SystemTimeSource, TimeSource,
};

pub use client::EtheanClient;

/// Main result type for the application
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the application
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Types error: {0}")]
    Types(#[from] TypesError),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Consensus error: {0}")]
    Consensus(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Integration error: {0}")]
    Integration(#[from] integration::IntegrationError),

    #[error("API error: {0}")]
    Api(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Profile error: {0}")]
    Profile(#[from] ethean_profile::ProfileError),

    #[error("Genesis error: {0}")]
    Genesis(#[from] ethean_genesis::GenesisError),
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
