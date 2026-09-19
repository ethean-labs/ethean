//! Prune below finalized floor while retaining recovery dependencies.

use crate::error::{Result, StorageError};
use ethean_primitives::Hash32;

/// Prune policy: drop blocks with slot strictly below `finalized_slot - retain_slots`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrunePolicy {
    /// Finalized slot floor.
    pub finalized_slot: u64,
    /// Extra slots of history to keep below finalized.
    pub retain_slots: u64,
}

impl PrunePolicy {
    /// Lowest slot that must remain available.
    pub fn floor_slot(&self) -> u64 {
        self.finalized_slot.saturating_sub(self.retain_slots)
    }

    /// Refuse pruning a root still required for recovery (pending parent, proofs, signer).
    pub fn may_delete(
        &self,
        slot: u64,
        root: &Hash32,
        protected: &[Hash32],
    ) -> Result<bool> {
        if protected.iter().any(|r| r == root) {
            return Err(StorageError::PruneUnsafe(
                "root is a recovery dependency".into(),
            ));
        }
        Ok(slot < self.floor_slot())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protects_listed_roots() {
        let p = PrunePolicy {
            finalized_slot: 100,
            retain_slots: 10,
        };
        let root = [1u8; 32];
        assert!(p.may_delete(50, &root, &[root]).is_err());
        assert_eq!(p.may_delete(50, &root, &[]).unwrap(), true);
        assert_eq!(p.may_delete(95, &root, &[]).unwrap(), false);
    }
}
