//! Rebuild a live fork-choice store from durable genesis + block blobs.

use crate::chain_fc::genesis_anchor_block;
use crate::chain_owner::ChainOwner;
use ethean_fork_choice::{create_store, ForkChoiceOpts};
use ethean_primitives::Hash32;
use ethean_profile::ChainProfile;
use ethean_transition::{apply_block_unverified, TransitionContext};
use ethean_types::{Block, SignedBlock, State};
use tracing::{debug, info, warn};

impl ChainOwner {
    /// Rebuild the structural store by replaying durable blocks from genesis.
    ///
    /// Falls back to anchoring at the restored tip when the chain is incomplete
    /// (pruned ancestors) but the tip blob is present.
    pub fn rebuild_fork_choice_from_durable(
        &mut self,
        genesis: State,
        profile: &ChainProfile,
        blobs: &[(Hash32, Vec<u8>)],
    ) {
        if self.fc.is_some() {
            return;
        }
        if self.try_replay_from_genesis(genesis.clone(), profile, blobs) {
            return;
        }
        self.try_anchor_at_tip(profile, blobs);
    }

    fn try_replay_from_genesis(
        &mut self,
        genesis: State,
        profile: &ChainProfile,
        blobs: &[(Hash32, Vec<u8>)],
    ) -> bool {
        let Ok(anchor) = genesis_anchor_block(&genesis) else {
            return false;
        };
        let mut store = match create_store(
            genesis,
            anchor,
            profile,
            ForkChoiceOpts::STRUCTURAL,
        ) {
            Ok(s) => s,
            Err(e) => {
                debug!(error = %e, "fork-choice replay create_store failed");
                return false;
            }
        };

        let mut blocks = decode_sorted_blocks(blobs);
        if blocks.is_empty() && self.head_state.as_ref().is_some_and(|s| s.slot.get() == 0) {
            self.fc = Some(store);
            self.sync_from_fork_choice();
            return true;
        }

        let ctx = TransitionContext::new(profile.clone());
        let mut applied = 0u32;
        for block in blocks.drain(..) {
            let Ok(root) = block.hash_tree_root() else {
                continue;
            };
            if store.blocks.contains_key(&root) {
                continue;
            }
            let Some(pre) = store.block_states.get(&block.parent_root).cloned() else {
                debug!(root0 = root[0], "fork-choice replay skipped unknown parent");
                continue;
            };
            // Advance store time so far-future checks do not reject historical blobs.
            let need = block.slot.get().saturating_mul(store.intervals_per_slot.max(1));
            if store.time < need {
                let _ = store.on_tick_with(need, false);
            }
            let Ok(out) = apply_block_unverified(&pre, &block, &ctx) else {
                debug!(root0 = root[0], "fork-choice replay STF failed");
                continue;
            };
            if let Err(e) = store.on_block(block, out.post_state) {
                debug!(error = %e, root0 = root[0], "fork-choice replay on_block failed");
                continue;
            }
            applied += 1;
        }

        if applied == 0 && self.head_state.as_ref().is_some_and(|s| s.slot.get() > 0) {
            return false;
        }
        self.fc = Some(store);
        self.sync_from_fork_choice();
        info!(applied, "fork-choice store rebuilt from durable blocks");
        true
    }

    fn try_anchor_at_tip(&mut self, profile: &ChainProfile, blobs: &[(Hash32, Vec<u8>)]) {
        let Some(state) = self.head_state.clone() else {
            return;
        };
        let tip = self.head_root;
        let Some(block) = blobs
            .iter()
            .find_map(|(root, bytes)| {
                if *root != tip {
                    return None;
                }
                decode_block(bytes)
            })
            .or_else(|| {
                blobs.iter().find_map(|(_, bytes)| {
                    let b = decode_block(bytes)?;
                    let Ok(r) = b.hash_tree_root() else {
                        return None;
                    };
                    (r == tip).then_some(b)
                })
            })
        else {
            debug!("fork-choice tip anchor skipped: tip blob missing");
            return;
        };
        match create_store(state, block, profile, ForkChoiceOpts::STRUCTURAL) {
            Ok(store) => {
                self.fc = Some(store);
                self.sync_from_fork_choice();
                warn!("fork-choice store anchored at restored tip (ancestors missing)");
            }
            Err(e) => warn!(error = %e, "fork-choice tip create_store failed"),
        }
    }
}

fn decode_sorted_blocks(blobs: &[(Hash32, Vec<u8>)]) -> Vec<Block> {
    let mut blocks: Vec<Block> = blobs.iter().filter_map(|(_, b)| decode_block(b)).collect();
    blocks.sort_by_key(|b| b.slot.get());
    blocks
}

fn decode_block(bytes: &[u8]) -> Option<Block> {
    if let Ok(signed) = SignedBlock::ssz_decode(bytes) {
        return Some(signed.block);
    }
    Block::ssz_decode(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Bytes52, Slot, ValidatorIndex, HASH32_ZERO};
    use ethean_profile::lstar_devnet;
    use ethean_types::{
        BlockBody, BlockHeader, Checkpoint, GenesisConfig, MultiMessageAggregate, Validator,
    };

    fn genesis_state() -> State {
        let val = Validator::new(Bytes52::ZERO, Bytes52::ZERO, ValidatorIndex::new(0)).unwrap();
        let mut st = State {
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
        st.latest_block_header.state_root = st.hash_tree_root().unwrap();
        st.latest_block_header.body_root = BlockBody::default().hash_tree_root().unwrap();
        st
    }

    #[test]
    fn rebuild_replays_child_from_genesis_blob() {
        let profile = lstar_devnet().unwrap();
        let genesis = genesis_state();
        let mut owner = ChainOwner::new(2);
        owner.profile = Some(profile.clone());
        owner.head_state = Some(genesis.clone());
        owner.try_init_fork_choice();
        let parent = owner.head_root;
        let pre = owner.head_state.clone().unwrap();
        let ctx = TransitionContext::new(profile.clone());

        let mut child = Block {
            slot: Slot::new(1),
            proposer_index: ValidatorIndex::new(0),
            parent_root: parent,
            state_root: HASH32_ZERO,
            body: BlockBody::default(),
        };
        let out = apply_block_unverified(&pre, &child, &ctx).unwrap();
        child.state_root = out.post_state.hash_tree_root().unwrap();
        let root = child.hash_tree_root().unwrap();
        let signed = SignedBlock::new(child.clone(), MultiMessageAggregate::default());
        let blob = signed.ssz_encode().unwrap();

        // Fresh owner at tip with durable blob — no live store yet.
        let mut resumed = ChainOwner::new(2);
        resumed.profile = Some(profile.clone());
        resumed.head_root = root;
        resumed.head_state = Some(out.post_state);
        resumed.rebuild_fork_choice_from_durable(genesis, &profile, &[(root, blob)]);
        assert!(resumed.fc.is_some());
        assert!(resumed.fc.as_ref().unwrap().blocks.contains_key(&root));
        assert!(resumed.fc.as_ref().unwrap().blocks.contains_key(&parent));
    }
}
