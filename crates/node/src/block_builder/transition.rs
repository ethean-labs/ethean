//! Transition planning hook before proposal signing.

use ethean_primitives::Hash32;
use ethean_types::Block;

/// Planned transition inputs for computing the post-state root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanTransition {
    /// Parent root the block extends.
    pub parent_root: Hash32,
    /// Block body constructed for this slot (pre-state-root).
    pub block: Block,
}

impl PlanTransition {
    /// Placeholder: real path calls `ethean_transition::apply_block_unverified`.
    /// Returns the block's declared state root for binding checks in later wiring.
    pub fn expected_state_root(&self) -> Hash32 {
        self.block.state_root
    }
}
