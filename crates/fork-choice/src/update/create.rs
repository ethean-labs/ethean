//! Create an initial fork-choice store from an anchor block and state.

use ethean_profile::ChainProfile;
use ethean_types::{Block, Checkpoint, State};

use crate::error::ForkChoiceError;
use crate::opts::ForkChoiceOpts;
use crate::store::ForkChoiceStore;

/// Initialize the store from an anchor state and block (leanSpec `create_store`).
pub fn create_store(
    anchor_state: State,
    anchor_block: Block,
    profile: &ChainProfile,
    opts: ForkChoiceOpts,
) -> Result<ForkChoiceStore, ForkChoiceError> {
    let computed = anchor_state
        .hash_tree_root()
        .map_err(|e| ForkChoiceError::Types(e.to_string()))?;
    if anchor_block.state_root != computed {
        return Err(ForkChoiceError::AnchorStateRootMismatch);
    }
    let anchor_root = anchor_block
        .hash_tree_root()
        .map_err(|e| ForkChoiceError::Types(e.to_string()))?;
    let checkpoint = Checkpoint {
        root: anchor_root,
        slot: anchor_block.slot,
    };
    let time = anchor_block.slot.get() * profile.intervals_per_slot;
    let genesis_time = anchor_state.config.genesis_time;

    let mut blocks = std::collections::HashMap::new();
    blocks.insert(anchor_root, anchor_block);
    let mut block_states = std::collections::HashMap::new();
    block_states.insert(anchor_root, anchor_state);

    Ok(ForkChoiceStore {
        time,
        intervals_per_slot: profile.intervals_per_slot,
        genesis_time,
        milliseconds_per_interval: profile.milliseconds_per_interval,
        gossip_disparity_intervals: profile.gossip_disparity_intervals,
        historical_roots_limit: profile.historical_roots_limit,
        opts,
        head: anchor_root,
        safe_target: anchor_root,
        latest_justified: checkpoint,
        latest_finalized: checkpoint,
        blocks,
        block_states,
        latest_new_attestations: std::collections::HashMap::new(),
        latest_known_attestations: std::collections::HashMap::new(),
        latest_new_payloads: std::collections::HashMap::new(),
        latest_known_payloads: std::collections::HashMap::new(),
        reorg_total: 0,
    })
}
