//! Core types for Ethean
//!
//! Modular type definitions - each file handles specific types.

pub mod block;
pub mod state; 
pub mod validator;
pub mod attestation;
pub mod checkpoint;
pub mod execution;

// Re-export key types
pub use block::{BeaconBlock, BeaconBlockHeader, BlockHash, Slot};
pub use state::{BeaconState, StateRoot};
pub use validator::{Validator, ValidatorIndex, ValidatorSet};
pub use attestation::{Attestation, AttestationData};
pub use checkpoint::{Checkpoint, Epoch};
pub use execution::{ExecutionPayload, ExecutionPayloadHeader};

/// Result type for type operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid slot: {0}")]
    InvalidSlot(u64),
    
    #[error("Invalid epoch: {0}")]
    InvalidEpoch(u64),
    
    #[error("Invalid validator index: {0}")]
    InvalidValidatorIndex(usize),
    
    #[error("Serialization failed: {0}")]
    Serialization(String),
}
