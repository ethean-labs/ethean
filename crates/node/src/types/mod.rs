//! Core types for Ethean
//!
//! Modular type definitions — each file handles specific types.

pub mod attestation;
pub mod block;
pub mod checkpoint;
pub mod execution;
pub mod state;
pub mod validator;

pub use ethean_primitives::{Epoch, Hash32, Slot, ValidatorIndex};

pub use attestation::{Attestation, AttestationData};
pub use block::{BeaconBlock, BeaconBlockHeader, BlockHash};
pub use checkpoint::Checkpoint;
pub use execution::{ExecutionPayload, ExecutionPayloadHeader};
pub use state::{BeaconState, StateRoot};
pub use validator::{Validator, ValidatorSet};

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
