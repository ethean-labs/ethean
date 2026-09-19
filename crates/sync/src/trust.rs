//! Explicit operator trust boundary for checkpoints.

use crate::checkpoint::CheckpointBundle;
use crate::error::{Result, SyncError};
use ethean_primitives::Hash32;

/// How an operator pins checkpoint trust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrustPolicy {
    /// Exact finalized root must match the operator pin.
    PinnedRoot(Hash32),
    /// Require N matching sources (quorum); sources compared by finalized_root.
    Quorum { threshold: usize },
}

/// Decide trust after structural validation. Never claims canonicality alone.
pub fn evaluate_trust(
    bundle: &CheckpointBundle,
    policy: &TrustPolicy,
    agreeing_sources: usize,
) -> Result<()> {
    bundle.validate_structure()?;
    match policy {
        TrustPolicy::PinnedRoot(root) => {
            if &bundle.finalized_root != root {
                return Err(SyncError::UntrustedCheckpoint(
                    "finalized root != operator pin".into(),
                ));
            }
        }
        TrustPolicy::Quorum { threshold } => {
            if agreeing_sources < *threshold {
                return Err(SyncError::UntrustedCheckpoint(format!(
                    "quorum {agreeing_sources} < {threshold}"
                )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> CheckpointBundle {
        CheckpointBundle {
            network_fingerprint: [1u8; 32],
            genesis_root: [2u8; 32],
            finalized_root: [3u8; 32],
            finalized_slot: 10,
            finalized_state_root: [4u8; 32],
            source_label: "https://checkpoint.example".into(),
            timestamp_unix: 100,
        }
    }

    #[test]
    fn pinned_root_must_match() {
        let b = sample();
        assert!(evaluate_trust(&b, &TrustPolicy::PinnedRoot([3u8; 32]), 0).is_ok());
        assert!(evaluate_trust(&b, &TrustPolicy::PinnedRoot([9u8; 32]), 0).is_err());
    }
}
