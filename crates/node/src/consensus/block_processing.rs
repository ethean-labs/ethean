//! Block processing — delegates to `ethean-transition`.

use crate::consensus::state_transition::{StateTransitionError, StateTransitionProcessor};
use ethean_types::{Block, State};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlockProcessingError {
    #[error("invalid block: {0}")]
    Invalid(String),
    #[error(transparent)]
    Transition(#[from] StateTransitionError),
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
    transition: StateTransitionProcessor,
}

impl BlockProcessor {
    pub fn new(transition: StateTransitionProcessor) -> Self {
        Self { transition }
    }

    /// Structural process only (no XMSS). Verified gossip path uses `ethean_transition::apply_block`.
    pub fn process_block(
        &self,
        state: &State,
        block: &Block,
    ) -> Result<BlockProcessingResult, BlockProcessingError> {
        let out = self.transition.state_transition(state, block)?;
        Ok(BlockProcessingResult {
            state_root: out.post_state_root,
        })
    }

    pub fn validate_block(&self, _block: &Block) -> Result<(), BlockProcessingError> {
        Ok(())
    }
}
