//! Finality gadget stub — Phase 06 wires store + 3SF checkpoints.
//!
//! Justification/finalization during block application lives in `ethean-transition`.

use ethean_types::Checkpoint;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FinalityError {
    #[error("finality: {0}")]
    Failed(String),
    #[error("stub: {0}")]
    Stub(String),
}

#[derive(Debug, Clone)]
pub struct FinalityGadget {
    pub latest_justified: Checkpoint,
    pub latest_finalized: Checkpoint,
}

impl FinalityGadget {
    pub fn new(genesis: Checkpoint) -> Self {
        Self {
            latest_justified: genesis,
            latest_finalized: genesis,
        }
    }

    pub fn get_latest_justified(&self) -> Checkpoint {
        self.latest_justified
    }

    pub fn get_latest_finalized(&self) -> Checkpoint {
        self.latest_finalized
    }
}

pub use ethean_types::Checkpoint;
