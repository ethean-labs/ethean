//! Attestation intake stub — no Beacon committees or rewards.
//!
//! Full attestation application lives in `ethean-transition` during block processing.
//! Standalone gossip attestation handling lands with Phase 06 fork choice.

use crate::consensus::validator_management::{ValidatorError, ValidatorManager};
use ethean_types::{Attestation, AttestationData, State};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AttestationError {
    #[error("invalid attestation: {0}")]
    Invalid(String),
    #[error(transparent)]
    Validator(#[from] ValidatorError),
    #[error("deferred: {0}")]
    Deferred(String),
}

#[derive(Debug, Clone, Default)]
pub struct AttestationConfig;

#[derive(Debug, Clone, Default)]
pub struct AttestationResult {
    pub accepted: bool,
}

#[derive(Debug, Clone, Default)]
pub struct AttestationStats {
    pub processed: u64,
}

pub struct AttestationProcessor {
    pub config: AttestationConfig,
    _validator_manager: ValidatorManager,
}

impl AttestationProcessor {
    pub fn new(config: AttestationConfig, validator_manager: ValidatorManager) -> Self {
        Self {
            config,
            _validator_manager: validator_manager,
        }
    }

    pub fn process(
        &mut self,
        _state: &State,
        _attestation: &Attestation,
    ) -> Result<AttestationResult, AttestationError> {
        Err(AttestationError::Deferred(
            "standalone attestation gossip deferred to Phase 06".into(),
        ))
    }

    pub fn validate_data(&self, data: &AttestationData) -> Result<(), AttestationError> {
        if data.source.slot.get() > data.target.slot.get() {
            return Err(AttestationError::Invalid(
                "source slot after target".into(),
            ));
        }
        Ok(())
    }
}
