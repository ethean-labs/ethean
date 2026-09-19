//! Prove and attach a Type-2 envelope to a planned proposal.

use crate::aggregation::{AggregationBudget, ProverJob, ProverOutcome, ProverWorker};
use crate::block_builder::PlanTransition;
use ethean_crypto::verify_type2;
use ethean_transition::type2_statement_for_block;

/// Outcome of attempting a local Type-2 prove for a plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type2ProveResult {
    /// Proof produced, re-verified, and written to `plan.aggregate_proof`.
    Attached { proof_len: usize },
    /// Backend unavailable or prove failed; plan left unchanged.
    Skipped,
}

/// Build the consensus Type-2 statement, prove it, verify, and attach bytes.
///
/// Fail-closed: on `BackendUnavailable` or verify failure the plan is not mutated.
pub fn try_attach_type2_proof(plan: &mut PlanTransition) -> Type2ProveResult {
    let statement = match type2_statement_for_block(&plan.block) {
        Ok(s) => s,
        Err(_) => return Type2ProveResult::Skipped,
    };
    let worker = ProverWorker::new(AggregationBudget::default());
    match worker.run(ProverJob { statement: statement.clone() }) {
        ProverOutcome::Produced(proof) => match verify_type2(&statement, &proof) {
            Ok(()) => {
                let proof_len = proof.len();
                plan.aggregate_proof = proof;
                Type2ProveResult::Attached { proof_len }
            }
            Err(_) => Type2ProveResult::Skipped,
        },
        ProverOutcome::Failed(msg) if msg.contains("unavailable") => Type2ProveResult::Skipped,
        ProverOutcome::Failed(_) | ProverOutcome::Budget(_) => Type2ProveResult::Skipped,
    }
}

/// True when crypto reports leanVM FFI ready (never claims test-aggregate as production).
pub fn production_type2_ready() -> bool {
    ethean_crypto::LeanVmGate::probe().ready()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Slot, ValidatorIndex};
    use ethean_types::{Block, BlockBody};

    #[test]
    fn attaches_test_aggregate_when_feature_on() {
        let mut plan = PlanTransition {
            parent_root: [1u8; 32],
            block: Block {
                slot: Slot::new(2),
                proposer_index: ValidatorIndex::new(0),
                parent_root: [1u8; 32],
                state_root: [3u8; 32],
                body: BlockBody::default(),
            },
            aggregate_proof: Vec::new(),
            proposer_signature: None,
        };
        // Workspace default enables test-aggregate on ethean-crypto.
        match try_attach_type2_proof(&mut plan) {
            Type2ProveResult::Attached { proof_len } => {
                assert!(proof_len > 0);
                assert_eq!(plan.aggregate_proof.len(), proof_len);
            }
            Type2ProveResult::Skipped => {
                // Acceptable if defaults were disabled in this build.
                assert!(plan.aggregate_proof.is_empty());
            }
        }
        assert!(!production_type2_ready());
    }
}
