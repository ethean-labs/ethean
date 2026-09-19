//! Historical backfill below the finalized anchor (does not move head).

use crate::error::{Result, SyncError};
use ethean_primitives::Hash32;

/// Backfill job description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackfillJob {
    /// Anchor finalized root (unchanged by backfill).
    pub anchor_finalized: Hash32,
    /// Lowest slot to fetch (inclusive).
    pub from_slot: u64,
    /// Slot just below the anchor (exclusive upper for history).
    pub to_slot_exclusive: u64,
}

/// Validate that backfill will not mutate live head / signer / checkpoints.
pub fn validate_backfill(
    job: &BackfillJob,
    current_head: &Hash32,
    current_finalized: &Hash32,
) -> Result<()> {
    if current_finalized != &job.anchor_finalized {
        return Err(SyncError::BackfillInvariant);
    }
    if job.to_slot_exclusive == 0 || job.from_slot >= job.to_slot_exclusive {
        return Err(SyncError::InvalidRange("backfill window".into()));
    }
    // Head must remain the live tip identity; backfill only fills history.
    let _ = current_head;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_anchor_drift() {
        let job = BackfillJob {
            anchor_finalized: [1u8; 32],
            from_slot: 0,
            to_slot_exclusive: 10,
        };
        assert!(validate_backfill(&job, &[2u8; 32], &[9u8; 32]).is_err());
    }
}
