//! Slashing stub — Beacon surround/double-vote detectors removed for Phase 03.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SlashingError {
    #[error("slashing: {0}")]
    Failed(String),
    #[error("stub: {0}")]
    Stub(String),
}

#[derive(Debug, Clone, Default)]
pub struct SlashingDetector;

impl SlashingDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn check_attestation(&self) -> Result<(), SlashingError> {
        Err(SlashingError::Stub(
            "slashing deferred to Phase 05".into(),
        ))
    }
}
