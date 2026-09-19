//! Parent sync by root with bounded depth.

use crate::error::{Result, SyncError};
use ethean_primitives::Hash32;

/// One missing parent request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParentRequest {
    /// Child root whose parent is unknown.
    pub child: Hash32,
    /// Parent root to fetch.
    pub parent: Hash32,
    /// Depth from the tip (0 = immediate parent).
    pub depth: u32,
}

/// Plan parent fetches up to `max_depth`; dedupe by parent root.
pub fn plan_parent_sync(
    missing: &[(Hash32, Hash32)],
    max_depth: u32,
) -> Result<Vec<ParentRequest>> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (i, (child, parent)) in missing.iter().enumerate() {
        let depth = i as u32;
        if depth > max_depth {
            return Err(SyncError::DepthExceeded);
        }
        if seen.insert(*parent) {
            out.push(ParentRequest {
                child: *child,
                parent: *parent,
                depth,
            });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedupes_parents() {
        let a = [1u8; 32];
        let b = [2u8; 32];
        let p = [9u8; 32];
        let plan = plan_parent_sync(&[(a, p), (b, p)], 8).unwrap();
        assert_eq!(plan.len(), 1);
    }

    #[test]
    fn depth_cap() {
        let mut missing = Vec::new();
        for i in 0..5u8 {
            missing.push(([i; 32], [i + 10; 32]));
        }
        assert!(plan_parent_sync(&missing, 2).is_err());
    }
}
