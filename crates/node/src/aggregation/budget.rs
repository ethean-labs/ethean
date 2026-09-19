//! CPU / wall-time budgets for aggregation (slot-aware).

use std::time::Duration;

/// Per-slot aggregation resource budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AggregationBudget {
    /// Maximum wall time for one prove attempt.
    pub max_wall_time: Duration,
    /// Maximum concurrent prove jobs.
    pub max_inflight: usize,
    /// Maximum retained pool entries per key.
    pub max_variants_per_key: usize,
}

impl Default for AggregationBudget {
    fn default() -> Self {
        Self {
            // Leave headroom inside a 4s slot / 5 intervals.
            max_wall_time: Duration::from_millis(800),
            max_inflight: 2,
            max_variants_per_key: 4,
        }
    }
}

/// Raised when a budget gate rejects work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetExhausted {
    /// Which gate fired.
    pub gate: &'static str,
}

impl AggregationBudget {
    /// Check wall-time allowance for a job age.
    pub fn allow_wall(&self, elapsed: Duration) -> Result<(), BudgetExhausted> {
        if elapsed > self.max_wall_time {
            Err(BudgetExhausted {
                gate: "max_wall_time",
            })
        } else {
            Ok(())
        }
    }
}
