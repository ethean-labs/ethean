//! Checkpoint bundle structure (trust decided separately).

use crate::error::{Result, SyncError};
use ethean_primitives::Hash32;

/// Checkpoint bundle after structural checks (not yet trusted).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointBundle {
    pub network_fingerprint: Hash32,
    pub genesis_root: Hash32,
    pub finalized_root: Hash32,
    pub finalized_slot: u64,
    pub finalized_state_root: Hash32,
    pub source_label: String,
    pub timestamp_unix: u64,
}

impl CheckpointBundle {
    /// Structural validation only — does not imply canonical membership.
    pub fn validate_structure(&self) -> Result<()> {
        if self.source_label.is_empty() {
            return Err(SyncError::InvalidCheckpoint("empty source".into()));
        }
        if self.finalized_root == [0u8; 32] || self.finalized_state_root == [0u8; 32] {
            return Err(SyncError::InvalidCheckpoint("zero root".into()));
        }
        if self.genesis_root == [0u8; 32] {
            return Err(SyncError::InvalidCheckpoint("zero genesis".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_finalized() {
        let b = CheckpointBundle {
            network_fingerprint: [1u8; 32],
            genesis_root: [2u8; 32],
            finalized_root: [0u8; 32],
            finalized_slot: 1,
            finalized_state_root: [3u8; 32],
            source_label: "op".into(),
            timestamp_unix: 1,
        };
        assert!(b.validate_structure().is_err());
    }
}
