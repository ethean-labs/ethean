//! Thin wrapper over `ethean-transition` (Phase 05).

use ethean_primitives::Slot;
use ethean_profile::lstar_devnet;
use ethean_transition::{
    apply_block_unverified, process_slots as lean_process_slots, state_transition,
    TransitionContext, TransitionError, TransitionOutcome, TransitionOpts,
};
use ethean_types::{Block, State};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateTransitionError {
    #[error(transparent)]
    Transition(#[from] TransitionError),
    #[error("profile: {0}")]
    Profile(String),
}

impl From<StateTransitionError> for TransitionError {
    fn from(value: StateTransitionError) -> Self {
        match value {
            StateTransitionError::Transition(e) => e,
            StateTransitionError::Profile(s) => TransitionError::Types(s),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StateTransitionConfig;

/// Node-facing processor that delegates to `ethean-transition`.
pub struct StateTransitionProcessor {
    pub config: StateTransitionConfig,
    ctx: TransitionContext,
}

impl StateTransitionProcessor {
    pub fn new(config: StateTransitionConfig) -> Self {
        let profile = lstar_devnet().expect("lstar_devnet profile must validate");
        Self {
            config,
            ctx: TransitionContext::new(profile),
        }
    }

    pub fn with_context(config: StateTransitionConfig, ctx: TransitionContext) -> Self {
        Self { config, ctx }
    }

    pub fn process_slots(
        &self,
        state: &mut State,
        slot: Slot,
    ) -> Result<(), StateTransitionError> {
        lean_process_slots(state, slot)?;
        Ok(())
    }

    /// Structural block application (fixtures / local tests). Not a verified API.
    pub fn process_block(
        &self,
        state: &mut State,
        block: &Block,
    ) -> Result<TransitionOutcome, StateTransitionError> {
        let out = apply_block_unverified(state, block, &self.ctx)?;
        *state = out.post_state.clone();
        Ok(out)
    }

    pub fn state_transition(
        &self,
        pre: &State,
        block: &Block,
    ) -> Result<TransitionOutcome, StateTransitionError> {
        Ok(state_transition(pre, block, &self.ctx)?)
    }

    pub fn opts_unverified(&self) -> TransitionOpts {
        TransitionOpts::UNVERIFIED
    }
}
