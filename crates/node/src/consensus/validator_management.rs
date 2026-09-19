//! Lean validator registry helpers (XMSS keys). No Beacon deposit/exit economics.

use crate::storage::StateStore;
use ethean_primitives::{Bytes52, Epoch, ValidatorIndex};
use ethean_types::{State, Validator};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidatorError {
    #[error("validator not found: {0}")]
    NotFound(u64),
    #[error("invalid validator: {0}")]
    Invalid(String),
    #[error("registry full")]
    RegistryFull,
}

/// Operational knobs for API/client wiring (not Lean consensus params).
#[derive(Debug, Clone, Default)]
pub struct ValidatorConfig {
    pub max_validators: u64,
}

/// Registry helper until Lean duties land in a dedicated crate.
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

    pub fn add_validator(
        &self,
        attestation_public_key: Bytes52,
        proposal_public_key: Bytes52,
        index: ValidatorIndex,
    ) -> Result<ValidatorIndex, ValidatorError> {
        let _ = Validator::new(attestation_public_key, proposal_public_key, index)
            .map_err(|e| ValidatorError::Invalid(e.to_string()))?;
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

    /// Lean registry has no epoch activation fields.
    pub fn is_active(&self, _validator: &Validator, _epoch: Epoch) -> bool {
        true
    }
}
