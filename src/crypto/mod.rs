//! Cryptography module placeholder
//!
//! Will contain WOTS, BLS, Poseidon hash implementations.

pub mod bls;
pub mod wots;
pub mod hash;

/// Crypto result type
pub type Result<T> = std::result::Result<T, Error>;

/// Crypto errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Key generation failed")]
    KeyGeneration,
}

// Placeholder exports
pub use bls::BlsSignature;
pub use wots::WotsSignature;
