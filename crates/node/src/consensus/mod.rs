//! Consensus module surface (Lean transition + fork choice).
//!
//! Phase 05: state transition wrappers over `ethean-transition`.
//! Phase 06: fork-choice store re-exported from `ethean-fork-choice`.

pub mod attestation_processing;
pub mod block_processing;
pub mod slashing;
pub mod state_transition;
pub mod validator_management;

pub use attestation_processing::*;
pub use block_processing::*;
pub use ethean_fork_choice::{
    create_store, ForkChoiceError, ForkChoiceOpts, ForkChoiceStore,
};
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
    ForkChoice(#[from] ForkChoiceError),

    #[error("State transition error: {0}")]
    StateTransition(#[from] StateTransitionError),

    #[error("Block processing error: {0}")]
    BlockProcessing(#[from] BlockProcessingError),

    #[error("Validator error: {0}")]
    Validator(#[from] ValidatorError),

    #[error("Storage error: {0}")]
    Storage(String),
}
