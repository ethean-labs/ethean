//! Result of a successful state transition.

use ethean_primitives::Hash32;
use ethean_types::State;

/// Post-state and derived roots after applying slots and/or a block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionOutcome {
    pub post_state: State,
    /// `hash_tree_root(post_state)` when computed; always set after full block transition.
    pub post_state_root: Hash32,
}
