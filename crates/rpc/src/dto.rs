//! JSON-ish response DTOs for Lean chain views (no Beacon field names).

use ethean_primitives::{Hash32, Slot};

/// Head view returned by GET /lean/v1/chain/head.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadView {
    pub slot: Slot,
    pub root: Hash32,
}

/// Finalized checkpoint view with explicit trust label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalizedView {
    pub slot: Slot,
    pub root: Hash32,
    /// Operator-visible trust source (never implied canonical by structure alone).
    pub trust_source: String,
}

/// Sync gate view for duties.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncView {
    pub syncing: bool,
    pub head_slot: Slot,
    pub peer_horizon_slot: Slot,
}

/// Fork-choice view returned by GET /lean/v1/chain/fork_choice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForkChoiceView {
    /// Live structural store is present on the node.
    pub live: bool,
    pub head_root: Hash32,
    pub safe_target_root: Hash32,
    pub safe_target_slot: u64,
    pub justified_root: Hash32,
    pub finalized_root: Hash32,
    pub reorg_total: u64,
    pub blocks: u64,
    pub pending_votes: u64,
    pub known_votes: u64,
}
