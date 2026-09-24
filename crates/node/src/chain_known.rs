//! Block roots this node has seen (leanSpec `build_block` `known_block_roots`).

use std::collections::HashSet;

use ethean_primitives::{Hash32, HASH32_ZERO};

use crate::chain_owner::ChainOwner;

impl ChainOwner {
    /// Head, durable blocks, the head chain history and every fork-choice block.
    pub fn known_block_roots(&self) -> HashSet<Hash32> {
        let mut roots = HashSet::new();
        if self.head_root != HASH32_ZERO {
            roots.insert(self.head_root);
        }
        roots.extend(self.durable_blocks.iter().map(|(r, _)| *r));
        if let Some(state) = self.head_state.as_ref() {
            roots.extend(
                state
                    .historical_block_hashes
                    .iter()
                    .copied()
                    .filter(|r| *r != HASH32_ZERO),
            );
        }
        if let Some(store) = self.fc.as_ref() {
            roots.extend(store.blocks.keys().copied());
        }
        roots
    }
}

impl ChainOwner {
    /// Parent of every block this node can decode (durable blobs and fork-choice store).
    fn parent_map(&self) -> std::collections::HashMap<Hash32, Hash32> {
        let mut parents = std::collections::HashMap::new();
        for (root, blob) in &self.durable_blocks {
            if let Ok(signed) = ethean_types::SignedBlock::ssz_decode(blob) {
                parents.insert(*root, signed.block.parent_root);
            }
        }
        if let Some(store) = self.fc.as_ref() {
            for (root, block) in &store.blocks {
                parents.insert(*root, block.parent_root);
            }
        }
        parents
    }

    /// Blocks dropped from the canonical chain when the head moves from
    /// `old_head` to `new_head` (distance from the old head to the common
    /// ancestor). Unknown ancestry counts the blocks that could be walked.
    pub fn reorg_depth(&self, old_head: Hash32, new_head: Hash32) -> u64 {
        let parents = self.parent_map();
        let mut new_branch = std::collections::HashSet::new();
        let mut cursor = new_head;
        for _ in 0..4096 {
            if !new_branch.insert(cursor) {
                break;
            }
            match parents.get(&cursor) {
                Some(parent) if *parent != HASH32_ZERO => cursor = *parent,
                _ => break,
            }
        }
        let mut depth = 0;
        let mut cursor = old_head;
        for _ in 0..4096 {
            if new_branch.contains(&cursor) {
                break;
            }
            depth += 1;
            match parents.get(&cursor) {
                Some(parent) if *parent != HASH32_ZERO => cursor = *parent,
                _ => break,
            }
        }
        depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_head_and_durable_roots() {
        let mut owner = ChainOwner::new(2);
        assert!(owner.known_block_roots().is_empty());
        owner.head_root = [1u8; 32];
        owner.remember_durable_block([2u8; 32], vec![0]);
        let roots = owner.known_block_roots();
        assert!(roots.contains(&[1u8; 32]) && roots.contains(&[2u8; 32]));
        assert_eq!(roots.len(), 2);
    }

    #[test]
    fn reorg_depth_counts_blocks_off_the_new_branch() {
        use ethean_primitives::{Slot, ValidatorIndex};
        use ethean_types::{Block, BlockBody, MultiMessageAggregate, SignedBlock};
        let mut owner = ChainOwner::new(2);
        let mut roots = Vec::new();
        let mut add = |parent: Hash32, slot: u64, proposer: u64| {
            let block = Block {
                slot: Slot::new(slot),
                proposer_index: ValidatorIndex::new(proposer),
                parent_root: parent,
                state_root: HASH32_ZERO,
                body: BlockBody::default(),
            };
            let root = block.hash_tree_root().unwrap();
            let signed = SignedBlock::new(block, MultiMessageAggregate::default());
            owner.remember_durable_block(root, signed.ssz_encode().unwrap());
            root
        };
        let a = add([1u8; 32], 1, 0);
        let b = add(a, 2, 0);
        let c = add(b, 3, 0);
        let x = add(a, 2, 1);
        roots.push((a, b, c, x));
        let (a, _b, c, x) = roots[0];
        assert_eq!(owner.reorg_depth(c, x), 2, "b and c leave the chain");
        assert_eq!(owner.reorg_depth(c, c), 0);
        assert_eq!(owner.reorg_depth(x, c), 1);
        assert_eq!(owner.reorg_depth(a, x), 0, "old head is an ancestor");
    }
}
