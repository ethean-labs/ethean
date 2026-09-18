//! Ethean Lean Consensus Client
//! 
//! Ultra-modular Rust implementation with small, focused modules.
//! Each module handles a single responsibility.

pub mod types;
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

// Re-export commonly used types
pub use types::{
    BeaconBlock, BeaconState, Validator, ValidatorIndex,
    Attestation, Checkpoint, Slot, Epoch
};

pub use client::EtheanClient;

/// Main result type for the application
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the application  
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Types error: {0}")]
    Types(#[from] types::Error),
    
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
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
