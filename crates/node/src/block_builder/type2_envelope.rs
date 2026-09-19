//! Proposer XMSS vs Type-2 proof embedding policy.
//!
//! leanMultisig has not pinned an on-wire merge of raw XMSS into the Type-2
//! proof blob. Until it does, keep the signature as a local sidecar and never
//! stuff it into `SignedBlock.proof` (remote verify would fail closed).

use crate::block_builder::PlanTransition;

/// How the local proposer signature relates to `SignedBlock.proof`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposerType2Policy {
    /// Signature stays off the proof field (current interop-safe default).
    Sidecar,
}

/// Active policy for this build (flip only after leanMultisig specifies encoding).
pub const PROPOSER_TYPE2_POLICY: ProposerType2Policy = ProposerType2Policy::Sidecar;

/// Proof bytes that may appear on the gossip `SignedBlock` envelope.
pub fn wire_proof_bytes(aggregate_proof: &[u8], _proposer_sig: Option<&[u8]>) -> Vec<u8> {
    match PROPOSER_TYPE2_POLICY {
        ProposerType2Policy::Sidecar => aggregate_proof.to_vec(),
    }
}

/// Refuse plans that already embedded the proposer signature inside the proof
/// under the Sidecar policy (guards against accidental concatenation).
pub fn assert_sidecar_invariant(plan: &PlanTransition) -> Result<(), String> {
    match PROPOSER_TYPE2_POLICY {
        ProposerType2Policy::Sidecar => {
            let Some(sig) = plan.proposer_signature.as_ref() else {
                return Ok(());
            };
            if sig.is_empty() || plan.aggregate_proof.is_empty() {
                return Ok(());
            }
            if plan
                .aggregate_proof
                .windows(sig.len())
                .any(|w| w == sig.as_slice())
            {
                return Err(
                    "proposer XMSS must not appear inside Type-2 proof under Sidecar policy"
                        .into(),
                );
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Slot, ValidatorIndex};
    use ethean_types::{Block, BlockBody};

    #[test]
    fn wire_proof_ignores_sidecar_sig() {
        let proof = vec![1, 2, 3];
        let sig = vec![9; 16];
        assert_eq!(wire_proof_bytes(&proof, Some(&sig)), proof);
    }

    #[test]
    fn rejects_sig_bytes_inside_proof() {
        let sig = vec![7u8; 8];
        let mut proof = vec![0u8; 4];
        proof.extend_from_slice(&sig);
        let plan = PlanTransition {
            parent_root: [0u8; 32],
            block: Block {
                slot: Slot::new(1),
                proposer_index: ValidatorIndex::new(0),
                parent_root: [0u8; 32],
                state_root: [0u8; 32],
                body: BlockBody::default(),
            },
            aggregate_proof: proof,
            proposer_signature: Some(sig),
        };
        assert!(assert_sidecar_invariant(&plan).is_err());
    }
}
