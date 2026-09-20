//! LMD-GHOST-style head walk from the justified checkpoint (lstar store).

use std::collections::{HashMap, HashSet};

use ethean_primitives::Hash32;
use ethean_types::Checkpoint;

use crate::error::ForkChoiceError;
use crate::head::tie_break::best_child;
use crate::store::ForkChoiceStore;

impl ForkChoiceStore {
    /// Greedy weighted walk from `start_root` using the given votes.
    ///
    /// When `min_score` is set, children below the threshold are ignored
    /// (safe-target walk).
    pub(crate) fn compute_weighted_head(
        &self,
        start_root: Hash32,
        attestations: &HashMap<ethean_primitives::ValidatorIndex, ethean_types::AttestationData>,
        min_score: Option<u64>,
    ) -> Result<Hash32, ForkChoiceError> {
        if !self.blocks.contains_key(&start_root) {
            return Err(ForkChoiceError::MissingBlockOrState);
        }
        let start_slot = self.blocks[&start_root].slot.get();
        let weights = self.accumulate_ancestor_weights(attestations, start_slot);

        let mut children_map: HashMap<Hash32, Vec<Hash32>> = HashMap::new();
        for (root, block) in &self.blocks {
            if let Some(min) = min_score {
                if weights.get(root).copied().unwrap_or(0) < min {
                    continue;
                }
            }
            children_map
                .entry(block.parent_root)
                .or_default()
                .push(*root);
        }

        let mut head = start_root;
        while let Some(children) = children_map.get(&head) {
            if children.is_empty() {
                break;
            }
            head = best_child(children, &weights);
        }
        Ok(head)
    }

    /// Recompute head from justified root + known votes; refresh finalized from head state.
    pub(crate) fn update_head(&mut self) -> Result<(), ForkChoiceError> {
        let votes = self.relevant_known_votes();
        let new_head = self.compute_weighted_head(self.latest_justified.root, &votes, None)?;
        let previous_head = self.head;

        let head_state = self
            .block_states
            .get(&new_head)
            .ok_or(ForkChoiceError::MissingBlockOrState)?;
        let finalized_slot = head_state.latest_finalized.slot;
        let mut finalized_root = new_head;
        while let Some(block) = self.blocks.get(&finalized_root) {
            if block.slot <= finalized_slot {
                break;
            }
            let parent = block.parent_root;
            if !self.blocks.contains_key(&parent) {
                break;
            }
            finalized_root = parent;
        }

        let latest_finalized = if self
            .blocks
            .get(&finalized_root)
            .map(|b| b.slot == finalized_slot)
            .unwrap_or(false)
        {
            Checkpoint {
                root: finalized_root,
                slot: finalized_slot,
            }
        } else {
            self.latest_finalized
        };

        if new_head != previous_head {
            if let (Some(prev_block), Some(new_block)) = (
                self.blocks.get(&previous_head),
                self.blocks.get(&new_head),
            ) {
                let prev_cp = Checkpoint {
                    root: previous_head,
                    slot: prev_block.slot,
                };
                let new_cp = Checkpoint {
                    root: new_head,
                    slot: new_block.slot,
                };
                // Extension keeps the old head on the ancestry; a competing tip is a reorg.
                if !self.checkpoint_is_ancestor(prev_cp, new_cp) {
                    self.reorg_total = self.reorg_total.saturating_add(1);
                }
            }
        }

        self.head = new_head;
        self.latest_finalized = latest_finalized;
        Ok(())
    }

    /// Deepest block backed by a 2/3 supermajority of pending votes.
    pub(crate) fn update_safe_target(&mut self) -> Result<(), ForkChoiceError> {
        let head_state = self
            .block_states
            .get(&self.head)
            .ok_or(ForkChoiceError::MissingBlockOrState)?;
        let num_validators = head_state.validators.len() as u64;
        let min_target_score = if num_validators == 0 {
            0
        } else {
            (num_validators * 2 + 2) / 3
        };
        let votes = self.relevant_new_votes();
        self.safe_target = self.compute_weighted_head(
            self.latest_justified.root,
            &votes,
            Some(min_target_score),
        )?;
        Ok(())
    }

    /// Promote pending votes into the counted pool and recompute the head.
    pub(crate) fn accept_new_attestations(&mut self) -> Result<(), ForkChoiceError> {
        for (validator, data) in self.latest_new_attestations.drain() {
            match self.latest_known_attestations.get(&validator) {
                Some(existing) if !should_replace_vote(existing, &data) => {}
                _ => {
                    self.latest_known_attestations.insert(validator, data);
                }
            }
        }
        self.promote_new_payloads();
        self.update_head()
    }

    /// Roots on the ancestry of `root` (inclusive), walking to genesis.
    pub(crate) fn ancestry_set(&self, root: Hash32) -> HashSet<Hash32> {
        let mut set = HashSet::new();
        let mut current = root;
        while set.insert(current) {
            match self.blocks.get(&current) {
                Some(block) => {
                    if block.parent_root == current {
                        break;
                    }
                    current = block.parent_root;
                }
                None => break,
            }
        }
        set
    }
}

fn should_replace_vote(
    existing: &ethean_types::AttestationData,
    candidate: &ethean_types::AttestationData,
) -> bool {
    if candidate.slot > existing.slot {
        return true;
    }
    if candidate.slot < existing.slot {
        return false;
    }
    candidate.hash_tree_root() > existing.hash_tree_root()
}
