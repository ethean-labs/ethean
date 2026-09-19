//! Prune blocks that finalization has orphaned.

use crate::store::ForkChoiceStore;

/// Drop blocks that are not ancestors of the finalized checkpoint and sit at or
/// below the finalized slot. Keeps the finalized ancestry intact.
pub fn prune_finalized_away(store: &mut ForkChoiceStore) {
    let keep = store.ancestry_set(store.latest_finalized.root);
    let finalized_slot = store.latest_finalized.slot;
    let doomed: Vec<_> = store
        .blocks
        .iter()
        .filter(|(root, block)| block.slot <= finalized_slot && !keep.contains(*root))
        .map(|(root, _)| *root)
        .collect();
    for root in doomed {
        store.blocks.remove(&root);
        store.block_states.remove(&root);
    }
}
