//! State transition stub — no Beacon economics.

use ethean_types::{Block, State};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateTransitionError {
    #[error("transition failed: {0}")]
    Failed(String),
    #[error("stub: {0}")]
    Stub(String),
}

#[derive(Debug, Clone, Default)]
pub struct StateTransitionConfig;

pub struct StateTransitionProcessor {
    pub config: StateTransitionConfig,
}

impl StateTransitionProcessor {
    pub fn new(config: StateTransitionConfig) -> Self {
        Self { config }
    }

    pub fn process_slots(
        &self,
        _state: &mut State,
        _slot: ethean_primitives::Slot,
    ) -> Result<(), StateTransitionError> {
        Err(StateTransitionError::Stub(
            "process_slots deferred to Phase 05".into(),
        ))
    }

    pub fn process_block(
        &self,
        _state: &mut State,
        _block: &Block,
    ) -> Result<(), StateTransitionError> {
        Err(StateTransitionError::Stub(
            "process_block deferred to Phase 05".into(),
        ))
    }
}
