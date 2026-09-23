//! Result of a successful state transition.

use std::time::Duration;

use ethean_primitives::Hash32;
use ethean_types::State;

/// Phase timings measured while applying a block (leanMetrics state
/// transition families). Zero when a phase did not run.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TransitionTimings {
    pub slots_processed: u64,
    pub slots: Duration,
    pub block: Duration,
    pub attestations_processed: u64,
    pub attestations: Duration,
}

/// Post-state and derived roots after applying slots and/or a block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionOutcome {
    pub post_state: State,
    /// `hash_tree_root(post_state)` when computed; always set after full block transition.
    pub post_state_root: Hash32,
    /// Phase timings of this transition.
    pub timings: TransitionTimings,
}
