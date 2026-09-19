//! Aggregation pool, selection, and isolated prover worker (Phase 08).

mod budget;
mod merge_pool;
mod pool;
mod recovery;
mod selection;
mod type2_split;
mod worker;

pub use budget::{AggregationBudget, BudgetExhausted};
pub use merge_pool::{merge_best_pool_variants, PoolMergeResult};
pub use pool::{AggregatePool, PoolEntry, PoolKey};
pub use recovery::discard_partial_proof;
pub use selection::{select_coverage, SelectionPolicy};
pub use type2_split::{seed_pool_from_signed_block, Type2SplitSeed};
pub use worker::{ProverJob, ProverOutcome, ProverWorker};

/// Module status for observability.
pub fn aggregation_ready() -> bool {
    // Production leanVM not wired; pool/worker operate in fail-closed prove mode.
    false
}
