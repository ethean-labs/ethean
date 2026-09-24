//! Fork-choice snapshot published by the node into [`crate::state::ApiSnapshot`].

use crate::dto::{CheckpointBody, ForkChoiceBody, ForkChoiceNodeBody};
use ethean_primitives::{Hash32, HASH32_ZERO};

/// Live fork-choice + finalized SSZ pair served by `/lean/v0`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ForkChoiceView {
    pub fork_choice: ForkChoiceBody,
    /// SSZ of the finalized `State` (canonical form: header `state_root` zeroed).
    pub finalized_state_ssz: Option<Vec<u8>>,
    /// SSZ of the finalized `SignedBlock` (blank proof at genesis).
    pub finalized_block_ssz: Option<Vec<u8>>,
    /// Chain has a sealed genesis / head and can answer checkpoint routes.
    pub chain_ready: bool,
}

impl ForkChoiceView {
    /// Fresh-node genesis tree: one node, all checkpoints on that root.
    pub fn genesis(
        genesis_root: Hash32,
        validator_count: u64,
        finalized_state_ssz: Vec<u8>,
        finalized_block_ssz: Vec<u8>,
    ) -> Self {
        let node = ForkChoiceNodeBody {
            root: genesis_root,
            slot: 0,
            parent_root: HASH32_ZERO,
            proposer_index: 0,
            weight: 0,
        };
        Self {
            fork_choice: ForkChoiceBody {
                nodes: vec![node],
                head: genesis_root,
                justified: CheckpointBody::genesis(genesis_root),
                finalized: CheckpointBody::genesis(genesis_root),
                safe_target: genesis_root,
                validator_count,
            },
            finalized_state_ssz: Some(finalized_state_ssz),
            finalized_block_ssz: Some(finalized_block_ssz),
            chain_ready: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_view_has_one_node() {
        let root = [1u8; 32];
        let v = ForkChoiceView::genesis(root, 4, vec![1], vec![2]);
        assert_eq!(v.fork_choice.nodes.len(), 1);
        assert_eq!(v.fork_choice.head, root);
        assert_eq!(v.fork_choice.justified.root, root);
        assert_eq!(v.fork_choice.finalized.root, root);
        assert_eq!(v.fork_choice.safe_target, root);
        assert!(v.fork_choice.validator_count > 0);
        assert!(v.chain_ready);
    }
}
