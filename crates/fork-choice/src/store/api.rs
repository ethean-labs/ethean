//! Read-only fork-choice snapshot used by the Lean HTTP API.

use ethean_primitives::Hash32;
use ethean_types::{Block, Checkpoint, State};

use crate::store::ForkChoiceStore;

/// One visible fork-choice node (`slot >= finalized.slot`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForkChoiceNode {
    pub root: Hash32,
    pub slot: u64,
    pub parent_root: Hash32,
    pub proposer_index: u64,
    pub weight: u64,
}

/// Read-only tree view: nodes at/after finalized, plus checkpoints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForkChoiceSnapshot {
    pub nodes: Vec<ForkChoiceNode>,
    pub head: Hash32,
    pub justified: Checkpoint,
    pub finalized: Checkpoint,
    pub safe_target: Hash32,
    pub validator_count: u64,
}

impl ForkChoiceStore {
    /// Nodes with `slot >= latest_finalized.slot` and weights from known votes.
    /// Missing weights default to 0 (never fabricated).
    pub fn api_nodes(&self) -> Vec<ForkChoiceNode> {
        let weights = self.block_weights_from_known();
        let finalized_slot = self.latest_finalized.slot.get();
        let mut nodes: Vec<ForkChoiceNode> = self
            .blocks
            .iter()
            .filter(|(_, b)| b.slot.get() >= finalized_slot)
            .map(|(root, block)| ForkChoiceNode {
                root: *root,
                slot: block.slot.get(),
                parent_root: block.parent_root,
                proposer_index: block.proposer_index.get(),
                weight: weights.get(root).copied().unwrap_or(0),
            })
            .collect();
        nodes.sort_by_key(|n| n.slot);
        nodes
    }

    /// Combined read-only snapshot for `/lean/v0/fork_choice`.
    pub fn api_snapshot(&self) -> ForkChoiceSnapshot {
        let validator_count = self
            .block_states
            .get(&self.head)
            .map(|s| s.validators.len() as u64)
            .unwrap_or(0);
        ForkChoiceSnapshot {
            nodes: self.api_nodes(),
            head: self.head,
            justified: self.latest_justified,
            finalized: self.latest_finalized,
            safe_target: self.safe_target,
            validator_count,
        }
    }

    /// Block at `root`, if known.
    pub fn block(&self, root: &Hash32) -> Option<&Block> {
        self.blocks.get(root)
    }

    /// Post-state at `root`, if known.
    pub fn block_state(&self, root: &Hash32) -> Option<&State> {
        self.block_states.get(root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_store, ForkChoiceOpts};
    use ethean_primitives::{Bytes52, Slot, ValidatorIndex, HASH32_ZERO};
    use ethean_profile::lstar_devnet;
    use ethean_types::{
        BlockBody, BlockHeader, Checkpoint, GenesisConfig, State, Validator,
    };

    fn genesis_pair() -> (State, ethean_types::Block) {
        let val = Validator::new(Bytes52::ZERO, Bytes52::ZERO, ValidatorIndex::ZERO).unwrap();
        let mut state = State {
            config: GenesisConfig::new(1_700_000_000),
            slot: Slot::ZERO,
            latest_block_header: BlockHeader::default(),
            latest_justified: Checkpoint::genesis(),
            latest_finalized: Checkpoint::genesis(),
            historical_block_hashes: Vec::new(),
            justified_slots: Vec::new(),
            validators: vec![val],
            justifications_roots: Vec::new(),
            justifications_validators: Vec::new(),
        };
        let canonical_root = {
            state.latest_block_header.state_root = HASH32_ZERO;
            state.hash_tree_root().unwrap()
        };
        let block = ethean_types::Block {
            slot: Slot::ZERO,
            proposer_index: ValidatorIndex::ZERO,
            parent_root: HASH32_ZERO,
            state_root: canonical_root,
            body: BlockBody::default(),
        };
        (state, block)
    }

    #[test]
    fn api_snapshot_one_genesis_node() {
        let (state, block) = genesis_pair();
        let profile = lstar_devnet().unwrap();
        let store = create_store(state, block, &profile, ForkChoiceOpts::STRUCTURAL).unwrap();
        let snap = store.api_snapshot();
        assert_eq!(snap.nodes.len(), 1);
        assert_eq!(snap.nodes[0].slot, 0);
        assert_eq!(snap.nodes[0].weight, 0);
        assert_eq!(snap.head, snap.nodes[0].root);
        assert!(snap.validator_count > 0);
    }
}
