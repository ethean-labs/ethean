//! Lean node benchmark hooks (crypto timing deferred to Phase 12 metrics).

/// Placeholder suite retained during migration; no BLS/WOTS production path.
#[derive(Debug, Default)]
pub struct LeanBenchmark;

impl LeanBenchmark {
    /// Construct an empty suite.
    pub fn new() -> Self {
        Self
    }

    /// Run placeholder benchmark (returns a status string).
    pub fn run_full_benchmark(&mut self) -> String {
        "benchmark suite deferred (Lean metrics / Phase 12+)".into()
    }
}

/// Historical alias during migration.
pub type BeamChainBenchmark = LeanBenchmark;
