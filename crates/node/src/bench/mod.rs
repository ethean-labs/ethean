//! Legacy benchmark hooks (BLS-era). Lean metrics land in a later phase.

use crate::crypto::bls::BlsAggregator;

/// Placeholder benchmark suite retained for API compatibility during migration.
pub struct BeamChainBenchmark {
    pub bls_aggregator: BlsAggregator,
}

impl BeamChainBenchmark {
    pub fn new() -> Self {
        Self {
            bls_aggregator: BlsAggregator::new(),
        }
    }

    pub fn run_full_benchmark(&mut self) -> String {
        "benchmark suite deferred (Lean metrics / Phase 12+)".into()
    }
}

impl Default for BeamChainBenchmark {
    fn default() -> Self {
        Self::new()
    }
}
