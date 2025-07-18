//! Consensus module for Beam Chain
//!
//! Implements state transitions, block processing, and consensus mechanisms.

pub mod state_transition;
pub mod block_processing;
pub mod validator_management;
pub mod attestation_processing;

pub use state_transition::{StateTransitionProcessor, StateTransitionConfig, StateTransitionError};
pub use block_processing::{BlockProcessor, BlockProcessingConfig, BlockProcessingError, BlockProcessingResult};
pub use validator_management::{ValidatorManager, ValidatorConfig, ValidatorError, ValidatorPerformance};

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
