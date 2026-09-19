//! Isolated prover worker facade (never runs on the chain-owner task).

use crate::aggregation::budget::{AggregationBudget, BudgetExhausted};
use ethean_crypto::{prove_type1, prove_type2, AggregateStatement, CryptoError, ProofKind};
use std::time::Instant;

/// One prove request.
#[derive(Debug, Clone)]
pub struct ProverJob {
    /// Canonical statement (consensus-derived).
    pub statement: AggregateStatement,
}

/// Outcome of a prove attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProverOutcome {
    /// Proof bytes produced and must be re-verified by the caller before pool insert.
    Produced(Vec<u8>),
    /// Budget gate rejected the job.
    Budget(BudgetExhausted),
    /// Backend unavailable or invalid inputs.
    Failed(String),
}

/// Synchronous worker used until a dedicated process exists.
///
/// Chain owner must not call this on the tick path without a timeout wrapper.
#[derive(Debug, Default)]
pub struct ProverWorker {
    budget: AggregationBudget,
}

impl ProverWorker {
    /// Construct with an explicit budget.
    pub fn new(budget: AggregationBudget) -> Self {
        Self { budget }
    }

    /// Run one prove job; enforces wall-time budget around the call.
    pub fn run(&self, job: ProverJob) -> ProverOutcome {
        let start = Instant::now();
        let result = match job.statement.kind {
            ProofKind::Type1 => prove_type1(&job.statement),
            ProofKind::Type2 => prove_type2(&job.statement),
        };
        if let Err(e) = self.budget.allow_wall(start.elapsed()) {
            return ProverOutcome::Budget(e);
        }
        match result {
            Ok(bytes) => ProverOutcome::Produced(bytes),
            Err(CryptoError::BackendUnavailable(msg)) => {
                ProverOutcome::Failed(msg.to_string())
            }
            Err(e) => ProverOutcome::Failed(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_crypto::{ParticipantSet, ProofKind};

    #[test]
    fn produces_test_aggregate() {
        let worker = ProverWorker::default();
        let statement = AggregateStatement {
            kind: ProofKind::Type1,
            profile_digest: [1u8; 32],
            message_root: [2u8; 32],
            slot: 1,
            participants: ParticipantSet::try_from_ordered(vec![0, 1]).unwrap(),
            components: vec![],
        };
        match worker.run(ProverJob { statement }) {
            ProverOutcome::Produced(p) => assert!(!p.is_empty()),
            other => panic!("unexpected {other:?}"),
        }
    }
}
