//! Forward range sync batching (max 1024 blocks per request).

use crate::error::{Result, SyncError};
use ethean_network_wire::MAX_BLOCKS_PER_REQUEST;

/// One range batch to request from peers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeBatch {
    pub start_slot: u64,
    pub count: u64,
    pub step: u64,
}

/// Split `[start, end]` into ordered batches of at most MAX_BLOCKS_PER_REQUEST.
pub fn plan_range(start_slot: u64, end_slot: u64, step: u64) -> Result<Vec<RangeBatch>> {
    if step == 0 {
        return Err(SyncError::InvalidRange("step must be >= 1".into()));
    }
    if end_slot < start_slot {
        return Err(SyncError::InvalidRange("end before start".into()));
    }
    let mut batches = Vec::new();
    let mut slot = start_slot;
    while slot <= end_slot {
        let remaining_slots = (end_slot - slot) / step + 1;
        let count = remaining_slots.min(MAX_BLOCKS_PER_REQUEST);
        batches.push(RangeBatch {
            start_slot: slot,
            count,
            step,
        });
        let advance = count.saturating_mul(step);
        let next = slot.saturating_add(advance);
        if next <= slot {
            break;
        }
        slot = next;
    }
    Ok(batches)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_large_range() {
        let batches = plan_range(0, 2000, 1).unwrap();
        assert!(batches.len() >= 2);
        assert!(batches.iter().all(|b| b.count <= MAX_BLOCKS_PER_REQUEST));
    }
}
