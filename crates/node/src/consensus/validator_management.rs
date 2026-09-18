//! Validator management stub — Lean registry has dual XMSS keys, no balances.
//! Beacon deposit/exit economics are deferred to Phase 05.

use crate::storage::StateStore;
use ethean_primitives::{Bytes52, Epoch, ValidatorIndex};
use ethean_types::{State, Validator};
use thiserror::Error;

/// Validator-layer errors.
#[derive(Debug, Error)]
pub enum ValidatorError {
    #[error("validator not found: {0}")]
    NotFound(u64),
    #[error("invalid deposit: {0}")]
    InvalidDeposit(String),
    #[error("registry full")]
    RegistryFull,
    #[error("stub: {0}")]
    Stub(String),
}

/// Operational knobs retained for API/client wiring (not Lean consensus params).
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    pub min_deposit_amount: u64,
    pub max_validators_per_epoch: u64,
    pub activation_delay: u64,
    pub exit_delay: u64,
    pub slashing_penalty_multiplier: u64,
    pub inactivity_penalty_per_epoch: u64,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            min_deposit_amount: 0,
            max_validators_per_epoch: 1000,
            activation_delay: 0,
            exit_delay: 0,
            slashing_penalty_multiplier: 0,
            inactivity_penalty_per_epoch: 0,
        }
    }
}

/// Placeholder manager until Lean duties land.
#[derive(Clone)]
pub struct ValidatorManager {
    pub config: ValidatorConfig,
    _state_store: StateStore,
}

impl ValidatorManager {
    pub fn new(config: ValidatorConfig, state_store: StateStore) -> Self {
        Self {
            config,
            _state_store: state_store,
        }
    }

    /// Register a Lean validator (XMSS keys). Beacon balances are not tracked.
    pub fn add_validator(
        &self,
        attestation_public_key: Bytes52,
        proposal_public_key: Bytes52,
        index: ValidatorIndex,
    ) -> Result<ValidatorIndex, ValidatorError> {
        let _ = Validator::new(attestation_public_key, proposal_public_key, index)
            .map_err(|e| ValidatorError::InvalidDeposit(e.to_string()))?;
        Ok(index)
    }

    pub fn get_validator<'a>(
        &self,
        state: &'a State,
        index: ValidatorIndex,
    ) -> Result<&'a Validator, ValidatorError> {
        state
            .validator(index.as_usize())
            .ok_or(ValidatorError::NotFound(index.get()))
    }

    /// Beacon-era activation check removed; Lean has no epoch activation fields.
    pub fn is_active(&self, _validator: &Validator, _epoch: Epoch) -> bool {
        true
    }
}
