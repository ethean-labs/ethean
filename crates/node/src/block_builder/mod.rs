//! Block builder helpers for proposer duties (selection → transition → publish).

mod assemble;
mod attestations;
mod publish;
mod selection;
mod transition;
mod type2;
mod type2_envelope;

pub use assemble::{assemble_signed_block, encode_proposal_gossip, ProposalGossip};
pub use attestations::body_from_pool;
pub use publish::{decide_publish, PublishDecision};
pub use selection::select_parent;
pub use transition::{plan_from_pool, PlanTransition};
pub use type2::{production_type2_ready, try_attach_type2_proof, Type2ProveResult};
pub use type2_envelope::{
    assert_sidecar_invariant, wire_proof_bytes, ProposerType2Policy, PROPOSER_TYPE2_POLICY,
};
