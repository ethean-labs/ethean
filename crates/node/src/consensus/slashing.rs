//! Slashing detectors — Beacon surround/double-vote paths removed.
//! Lean equivocation policy is not specified for Phase 05; stub only.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SlashingError {
    #[error("slashing: {0}")]
    Failed(String),
    #[error("not implemented: {0}")]
    NotImplemented(String),
}

#[derive(Debug, Clone, Default)]
pub struct SlashingDetector;

impl SlashingDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn check_attestation(&self) -> Result<(), SlashingError> {
        Err(SlashingError::NotImplemented(
            "Lean equivocation checks not in Phase 05".into(),
        ))
    }
}
