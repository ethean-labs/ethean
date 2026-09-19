//! Checkpoint ancestry helpers for the fork-choice store.

use ethean_types::Checkpoint;

use crate::store::ForkChoiceStore;

impl ForkChoiceStore {
    /// True when `ancestor` lies on the parent chain of `descendant`.
    pub(crate) fn checkpoint_is_ancestor(
        &self,
        ancestor: Checkpoint,
        descendant: Checkpoint,
    ) -> bool {
        if ancestor.slot > descendant.slot {
            return false;
        }
        let mut current_root = descendant.root;
        while let Some(block) = self.blocks.get(&current_root) {
            if block.slot == ancestor.slot {
                return current_root == ancestor.root;
            }
            if block.slot < ancestor.slot {
                return false;
            }
            current_root = block.parent_root;
        }
        false
    }
}
