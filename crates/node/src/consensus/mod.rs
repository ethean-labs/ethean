//! Consensus module surface (Lean transition).
//!
//! Phase 03 replaces Beacon containers with `ethean-types`. Full 3SF / duty
//! logic lands in Phase 05; these modules keep a compile-stable stub API.

pub mod attestation_processing;
pub mod block_processing;
pub mod finality;
pub mod fork_choice;
pub mod performance;
pub mod slashing;
pub mod state_transition;
pub mod validator_management;

pub use attestation_processing::*;
pub use block_processing::*;
pub use finality::*;
pub use fork_choice::*;
pub use performance::*;
pub use slashing::*;
pub use state_transition::*;
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
