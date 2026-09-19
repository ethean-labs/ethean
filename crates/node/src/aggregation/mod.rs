//! Aggregation pool, selection, and isolated prover worker (Phase 08).

mod budget;
mod pool;
mod recovery;
mod selection;
mod worker;

pub use budget::{AggregationBudget, BudgetExhausted};
pub use pool::{AggregatePool, PoolEntry, PoolKey};
pub use recovery::discard_partial_proof;
pub use selection::{select_coverage, SelectionPolicy};
pub use worker::{ProverJob, ProverOutcome, ProverWorker};

/// Module status for observability.
pub fn aggregation_ready() -> bool {
    // Production leanVM not wired; pool/worker operate in fail-closed prove mode.
    false
}
