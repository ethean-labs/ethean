//! Blocks-by-root and blocks-by-range request shapes.

use ethean_primitives::Hash32;

use crate::error::{Result, WireError};
use crate::limits::MAX_BLOCKS_PER_REQUEST;

/// Blocks-by-root request: ordered unique roots, ≤ 1024.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlocksByRootRequest {
    /// Requested block roots in preference order.
    pub roots: Vec<Hash32>,
}

impl BlocksByRootRequest {
    /// Construct after validating count and uniqueness.
    pub fn new(roots: Vec<Hash32>) -> Result<Self> {
        if roots.is_empty() {
            return Err(WireError::InvalidReqResp("empty roots".into()));
        }
        if roots.len() as u64 > MAX_BLOCKS_PER_REQUEST {
            return Err(WireError::InvalidReqResp(format!(
                "roots {} exceeds {}",
                roots.len(),
                MAX_BLOCKS_PER_REQUEST
            )));
        }
        for i in 0..roots.len() {
            for j in 0..i {
                if roots[i] == roots[j] {
                    return Err(WireError::InvalidReqResp("duplicate root".into()));
                }
            }
        }
        Ok(Self { roots })
    }
}

/// Blocks-by-range request: start slot, count, step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlocksByRangeRequest {
    /// First slot (inclusive).
    pub start_slot: u64,
    /// Number of slots to cover (not necessarily blocks produced).
    pub count: u64,
    /// Slot step (≥ 1).
    pub step: u64,
}

impl BlocksByRangeRequest {
    /// Validate count/step against the pinned maximum.
    pub fn new(start_slot: u64, count: u64, step: u64) -> Result<Self> {
        if count == 0 || count > MAX_BLOCKS_PER_REQUEST {
            return Err(WireError::InvalidReqResp(format!(
                "count {count} invalid (max {MAX_BLOCKS_PER_REQUEST})"
            )));
        }
        if step == 0 {
            return Err(WireError::InvalidReqResp("step must be >= 1".into()));
        }
        Ok(Self {
            start_slot,
            count,
            step,
        })
    }

    /// Enumerate requested slots (may include empty slots).
    pub fn slots(&self) -> Vec<u64> {
        let mut out = Vec::with_capacity(self.count as usize);
        let mut slot = self.start_slot;
        for _ in 0..self.count {
            out.push(slot);
            slot = slot.saturating_add(self.step);
        }
        out
    }
}

/// Response code for a single req/resp chunk (lean-style scaffolding).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ResponseCode {
    /// Success chunk follows.
    Success = 0,
    /// Resource unavailable / missing.
    ResourceUnavailable = 1,
    /// Invalid request.
    InvalidRequest = 2,
    /// Server error.
    ServerError = 3,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_too_many_roots() {
        let roots = vec![[0u8; 32]; 1025];
        assert!(BlocksByRootRequest::new(roots).is_err());
    }

    #[test]
    fn range_slots_step() {
        let r = BlocksByRangeRequest::new(10, 3, 2).unwrap();
        assert_eq!(r.slots(), vec![10, 12, 14]);
    }
}
