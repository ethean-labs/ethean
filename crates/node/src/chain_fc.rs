//! Optional live [`ForkChoiceStore`] helpers on [`ChainOwner`].

use crate::chain_owner::ChainOwner;
use ethean_fork_choice::{create_store, ForkChoiceOpts, ForkChoiceStore};
use ethean_primitives::{Hash32, HASH32_ZERO};
use ethean_types::{Block, BlockBody, State};
use tracing::{debug, warn};

impl ChainOwner {
    /// Build a structural fork-choice store from the sealed genesis head, if possible.
    pub fn try_init_fork_choice(&mut self) {
        if self.fc.is_some() {
            return;
        }
        let (Some(state), Some(profile)) = (self.head_state.clone(), self.profile.clone()) else {
            return;
        };
        let Ok(anchor) = genesis_anchor_block(&state) else {
            debug!("fork-choice init skipped: cannot rebuild genesis block");
            return;
        };
        match create_store(state, anchor, &profile, ForkChoiceOpts::STRUCTURAL) {
            Ok(store) => {
                self.fc = Some(store);
                self.sync_from_fork_choice();
                debug!("fork-choice store initialized (structural)");
            }
            Err(e) => warn!(error = %e, "fork-choice create_store failed"),
        }
    }

    /// Import an applied block into the live store (structural path).
    pub fn fc_on_block(&mut self, block: Block, post_state: State) {
        let Some(fc) = self.fc.as_mut() else {
            return;
        };
        if let Err(e) = fc.on_block(block, post_state) {
            debug!(error = %e, "fork-choice on_block skipped");
            return;
        }
        self.sync_from_fork_choice();
    }

    /// Advance fork-choice time to the absolute interval for this duty tick.
    pub fn fc_on_tick(&mut self, slot: u64, interval: u8, has_proposal: bool) {
        let Some(fc) = self.fc.as_mut() else {
            return;
        };
        let ips = fc.intervals_per_slot.max(1);
        let target = slot.saturating_mul(ips).saturating_add(u64::from(interval));
        if let Err(e) = fc.on_tick_with(target, has_proposal) {
            debug!(error = %e, target, "fork-choice on_tick skipped");
            return;
        }
        self.sync_from_fork_choice();
    }

    /// Copy safe-target and reorg counter from the store onto the owner view.
    pub fn sync_from_fork_choice(&mut self) {
        let Some(fc) = self.fc.as_ref() else {
            return;
        };
        self.safe_target = fc.safe_target();
        self.reorg_total = fc.reorg_total;
    }

    /// Slot of `safe_target`, preferring the live store block tree.
    pub fn safe_target_slot(&self) -> u64 {
        if let Some(fc) = self.fc.as_ref() {
            if let Some(block) = fc.blocks.get(&self.safe_target) {
                return block.slot.get();
            }
        }
        self.head_state
            .as_ref()
            .map(|s| {
                if self.safe_target == s.latest_justified.root {
                    s.latest_justified.slot.get()
                } else if self.safe_target == self.head_root {
                    s.slot.get()
                } else {
                    s.latest_justified.slot.get()
                }
            })
            .unwrap_or(0)
    }
}

fn genesis_anchor_block(state: &State) -> Result<Block, ()> {
    if state.slot.get() != 0 {
        return Err(());
    }
    let empty = BlockBody::default();
    let body_root = empty.hash_tree_root().map_err(|_| ())?;
    let h = &state.latest_block_header;
    if h.body_root != HASH32_ZERO && h.body_root != body_root {
        return Err(());
    }
    let state_root = if h.state_root == HASH32_ZERO {
        state.hash_tree_root().map_err(|_| ())?
    } else {
        h.state_root
    };
    let computed = state.hash_tree_root().map_err(|_| ())?;
    if state_root != computed {
        return Err(());
    }
    Ok(Block {
        slot: h.slot,
        proposer_index: h.proposer_index,
        parent_root: h.parent_root,
        state_root,
        body: empty,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Bytes52, Slot, ValidatorIndex};
    use ethean_profile::lstar_devnet;
    use ethean_types::{BlockHeader, Checkpoint, GenesisConfig, Validator};

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
    fn init_store_sets_safe_target() {
        let mut owner = ChainOwner::new(2);
        owner.head_state = Some(genesis_state());
        owner.profile = Some(lstar_devnet().unwrap());
        owner.head_root = owner
            .head_state
            .as_ref()
            .unwrap()
            .latest_block_header
            .hash_tree_root();
        owner.try_init_fork_choice();
        assert!(owner.fc.is_some());
        assert_ne!(owner.safe_target, Hash32::default());
    }
}
