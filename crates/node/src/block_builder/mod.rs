//! Block builder helpers for proposer duties (selection → transition → publish).

mod assemble;
mod attestations;
mod publish;
mod selection;
mod transition;

pub use assemble::{assemble_signed_block, encode_proposal_gossip, ProposalGossip};
pub use attestations::body_from_pool;
pub use publish::{decide_publish, PublishDecision};
pub use selection::select_parent;
pub use transition::{plan_from_pool, PlanTransition};
