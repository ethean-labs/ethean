//! Fork choice stub (modified 3SF / lstar lands in Phase 05–06).

use ethean_primitives::{Hash32, Slot};
use ethean_types::{Checkpoint, State};
use thiserror::Error;

pub type Hash = Hash32;

#[derive(Debug, Error)]
pub enum ForkChoiceError {
    #[error("fork choice: {0}")]
    Failed(String),
    #[error("stub: {0}")]
    Stub(String),
}

#[derive(Debug, Clone)]
pub struct LMDGHOSTForkChoice {
    pub latest_justified: Checkpoint,
    pub latest_finalized: Checkpoint,
    pub head: Hash,
}

impl LMDGHOSTForkChoice {
    pub fn new(genesis_block: Hash, genesis_checkpoint: Checkpoint) -> Self {
        Self {
            latest_justified: genesis_checkpoint,
            latest_finalized: genesis_checkpoint,
            head: genesis_block,
        }
    }

    pub fn get_head(&self) -> Hash {
        self.head
    }

    pub fn get_justified_checkpoint(&self) -> Checkpoint {
        self.latest_justified
    }

    pub fn get_finalized_checkpoint(&self) -> Checkpoint {
        self.latest_finalized
    }

    pub fn on_block(
        &mut self,
        _block_hash: Hash,
        _parent_hash: Hash,
        _slot: Slot,
        _state_root: Hash,
        _state: &State,
    ) -> Result<(), ForkChoiceError> {
        Err(ForkChoiceError::Stub(
            "fork choice on_block deferred to Phase 05".into(),
        ))
    }
}
