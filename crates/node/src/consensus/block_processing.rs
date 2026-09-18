//! Block processing stub.

use crate::consensus::state_transition::{StateTransitionError, StateTransitionProcessor};
use ethean_types::{Block, State};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlockProcessingError {
    #[error("invalid block: {0}")]
    Invalid(String),
    #[error(transparent)]
    Transition(#[from] StateTransitionError),
    #[error("stub: {0}")]
    Stub(String),
}

#[derive(Debug, Clone, Default)]
pub struct BlockProcessingConfig;

#[derive(Debug, Clone, Default)]
pub struct BlockProcessingResult {
    pub state_root: [u8; 32],
}

#[derive(Debug, Clone, Default)]
pub struct ProcessingStats;

pub struct BlockProcessor {
    _transition: StateTransitionProcessor,
}

impl BlockProcessor {
    pub fn new(transition: StateTransitionProcessor) -> Self {
        Self {
            _transition: transition,
        }
    }

    pub fn process_block(
        &self,
        _state: &State,
        _block: &Block,
    ) -> Result<BlockProcessingResult, BlockProcessingError> {
        Err(BlockProcessingError::Stub(
            "block processing deferred to Phase 05".into(),
        ))
    }

    pub fn validate_block(&self, _block: &Block) -> Result<(), BlockProcessingError> {
        Ok(())
    }
}
