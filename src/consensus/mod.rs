//! Beam Chain consensus module
//!
//! This module implements the core consensus mechanisms for the Beam Chain,
//! including attestation processing, validator management, and state transitions.

pub mod attestation_processing;
pub mod block_processing;
pub mod performance;
pub mod state_transition;
pub mod types;
pub mod validator_management;

pub use attestation_processing::*;
pub use block_processing::*;
pub use performance::*;
pub use state_transition::*;
pub use types::*;
pub use validator_management::*;

/// Consensus result type
pub type Result<T> = std::result::Result<T, Error>;

/// Consensus errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid block: {0}")]
    InvalidBlock(String),
    
    #[error("Fork choice error: {0}")]
    ForkChoice(String),
    
    #[error("State transition error: {0}")]
    StateTransition(#[from] StateTransitionError),
    
    #[error("Block processing error: {0}")]
    BlockProcessing(#[from] BlockProcessingError),
    
    #[error("Validator error: {0}")]
    Validator(#[from] ValidatorError),
    
    #[error("Storage error: {0}")]
    Storage(String),
}
