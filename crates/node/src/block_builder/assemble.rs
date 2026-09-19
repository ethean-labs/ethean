//! Assemble a Type-2 `SignedBlock` envelope from a planned proposal.

use crate::block_builder::PlanTransition;
use ethean_network::LeanGossipTopics;
use ethean_primitives::Hash32;
use ethean_types::{MultiMessageAggregate, SignedBlock};

/// Gossip-ready proposal bytes for the `/block/` topic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalGossip {
    /// Lean block topic string.
    pub topic: String,
    /// SSZ-encoded `SignedBlock`.
    pub payload: Vec<u8>,
    /// Tree root of the inner block (not the signed envelope).
    pub block_root: Hash32,
    /// True when the envelope carries a non-empty Type-2 proof.
    pub has_type2_proof: bool,
}

/// Wrap a plan as `SignedBlock` (proof may be empty → remote uses structural STF).
pub fn assemble_signed_block(plan: &PlanTransition) -> Result<SignedBlock, String> {
    let proof = MultiMessageAggregate::new(plan.aggregate_proof.clone()).map_err(|e| e.to_string())?;
    Ok(SignedBlock::new(plan.block.clone(), proof))
}

/// Encode a planned proposal for Lean block gossip.
pub fn encode_proposal_gossip(
    plan: &PlanTransition,
    fork_name: &str,
) -> Result<ProposalGossip, String> {
    let signed = assemble_signed_block(plan)?;
    let block_root = plan.block_root()?;
    let payload = signed.ssz_encode().map_err(|e| e.to_string())?;
    let topics = LeanGossipTopics::from_fork_name(fork_name).map_err(|e| e.to_string())?;
    Ok(ProposalGossip {
        topic: topics.block,
        payload,
        block_root,
        has_type2_proof: !plan.aggregate_proof.is_empty(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Slot, ValidatorIndex, HASH32_ZERO};
    use ethean_types::{Block, BlockBody};

    fn sample_plan(proof: Vec<u8>) -> PlanTransition {
        PlanTransition {
            parent_root: [1u8; 32],
            block: Block {
                slot: Slot::new(2),
                proposer_index: ValidatorIndex::new(0),
                parent_root: [1u8; 32],
                state_root: [3u8; 32],
                body: BlockBody::default(),
            },
            aggregate_proof: proof,
        }
    }

    #[test]
    fn assembles_empty_proof_envelope() {
        let plan = sample_plan(Vec::new());
        let signed = assemble_signed_block(&plan).expect("signed");
        assert!(signed.proof.proof.is_empty());
        assert_eq!(signed.block.state_root, [3u8; 32]);
    }

    #[test]
    fn encodes_lstar_block_topic() {
        let plan = sample_plan(vec![9, 9, 9]);
        let gossip = encode_proposal_gossip(&plan, "lstar").expect("gossip");
        assert!(gossip.topic.ends_with("/block/ssz_snappy"));
        assert!(gossip.has_type2_proof);
        assert_ne!(gossip.block_root, HASH32_ZERO);
        assert!(!gossip.payload.is_empty());
        let decoded = SignedBlock::ssz_decode(&gossip.payload).expect("decode");
        assert_eq!(decoded.proof.proof, vec![9, 9, 9]);
    }
}
