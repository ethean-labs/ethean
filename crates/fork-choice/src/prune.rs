//! Prune stale attestation data after finalization advances (leanSpec).

use std::collections::HashMap;

use ethean_primitives::Hash32;

use crate::store::ForkChoiceStore;

impl ForkChoiceStore {
    /// Drop blocks and post-states that are not the finalized block or one of
    /// its descendants; returns how many were dropped.
    ///
    /// Not part of leanSpec (its store keeps every block, and the fixtures
    /// compare the full block set), so the store never calls this itself. A
    /// long-running node needs it: every post-state carries the whole
    /// `historical_block_hashes`, so keeping all of them grows quadratically.
    /// Nothing is pruned while the justified checkpoint does not descend from
    /// the finalized one, since head selection starts from it.
    pub fn prune_finalized_history(&mut self) -> usize {
        let finalized = self.latest_finalized;
        if !self.blocks.contains_key(&finalized.root)
            || !self.checkpoint_is_ancestor(finalized, self.latest_justified)
        {
            return 0;
        }
        let mut keep: HashMap<Hash32, bool> = HashMap::from([(finalized.root, true)]);
        let roots: Vec<Hash32> = self.blocks.keys().copied().collect();
        for root in roots {
            let mut path = Vec::new();
            let mut cursor = root;
            let verdict = loop {
                if let Some(v) = keep.get(&cursor) {
                    break *v;
                }
                match self.blocks.get(&cursor) {
                    Some(block) if block.slot > finalized.slot => {
                        path.push(cursor);
                        cursor = block.parent_root;
                    }
                    _ => break false,
                }
            };
            keep.insert(root, verdict);
            for r in path {
                keep.insert(r, verdict);
            }
        }
        let before = self.blocks.len();
        self.blocks
            .retain(|root, _| keep.get(root).copied().unwrap_or(false));
        self.block_states
            .retain(|root, _| self.blocks.contains_key(root));
        before - self.blocks.len()
    }
}

/// Drop votes and payloads whose head can no longer influence fork choice.
///
/// leanSpec filters attestation-keyed pools only — it does **not** delete
/// blocks from the store when finalization advances. A vote/payload survives
/// when its head is strictly above the finalized slot and a descendant of the
/// finalized checkpoint.
pub fn prune_stale_attestation_data(store: &mut ForkChoiceStore) {
    let finalized = store.latest_finalized;

    let drop_new: Vec<_> = store
        .latest_new_attestations
        .iter()
        .filter(|(_, data)| {
            !(data.head.slot > finalized.slot && store.checkpoint_is_ancestor(finalized, data.head))
        })
        .map(|(k, _)| *k)
        .collect();
    for k in drop_new {
        store.latest_new_attestations.remove(&k);
    }

    let drop_known: Vec<_> = store
        .latest_known_attestations
        .iter()
        .filter(|(_, data)| {
            !(data.head.slot > finalized.slot && store.checkpoint_is_ancestor(finalized, data.head))
        })
        .map(|(k, _)| *k)
        .collect();
    for k in drop_known {
        store.latest_known_attestations.remove(&k);
    }

    let drop_new_payloads: Vec<Hash32> = store
        .latest_new_payloads
        .iter()
        .filter(|(_, entry)| {
            !(entry.data.head.slot > finalized.slot
                && store.checkpoint_is_ancestor(finalized, entry.data.head))
        })
        .map(|(k, _)| *k)
        .collect();
    for k in drop_new_payloads {
        store.latest_new_payloads.remove(&k);
    }

    let drop_known_payloads: Vec<Hash32> = store
        .latest_known_payloads
        .iter()
        .filter(|(_, entry)| {
            !(entry.data.head.slot > finalized.slot
                && store.checkpoint_is_ancestor(finalized, entry.data.head))
        })
        .map(|(k, _)| *k)
        .collect();
    for k in drop_known_payloads {
        store.latest_known_payloads.remove(&k);
    }
}
