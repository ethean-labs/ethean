//! Checkpoint sync (leanSpec `node/sync/checkpoint_sync.py`): fetch the
//! finalized state and block pair from a peer's `/lean/v0` API, verify that
//! they belong together, and start the chain from that anchor.

use std::time::Duration;

use ethean_primitives::Hash32;
use ethean_types::{SignedBlock, State};
use tracing::info;

use crate::chain_owner::ChainOwner;
use crate::checkpoint_http::http_get;

/// leanSpec checkpoint request timeout.
pub const CHECKPOINT_TIMEOUT: Duration = Duration::from_secs(60);

/// A verified finalized state and its block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointAnchor {
    pub state: State,
    pub signed_block: SignedBlock,
    pub block_root: Hash32,
    pub block_bytes: Vec<u8>,
}

/// `.../states/finalized` -> `.../blocks/finalized` (hive passes the state URL).
pub fn blocks_url(states_url: &str) -> Result<String, String> {
    if states_url.contains("/states/finalized") {
        Ok(states_url.replace("/states/finalized", "/blocks/finalized"))
    } else if states_url.contains("/blocks/finalized") {
        Ok(states_url.to_string())
    } else {
        Ok(format!(
            "{}/lean/v0/blocks/finalized",
            states_url.trim_end_matches('/')
        ))
    }
}

/// `.../blocks/finalized` or a bare base URL -> `.../states/finalized`.
pub fn states_url(url: &str) -> String {
    if url.contains("/states/finalized") {
        url.to_string()
    } else if url.contains("/blocks/finalized") {
        url.replace("/blocks/finalized", "/states/finalized")
    } else {
        format!("{}/lean/v0/states/finalized", url.trim_end_matches('/'))
    }
}

/// Decode and cross-check a state / signed-block pair.
pub fn verify_pair(state_bytes: &[u8], block_bytes: &[u8]) -> Result<CheckpointAnchor, String> {
    let state = State::ssz_decode(state_bytes).map_err(|e| format!("state: {e}"))?;
    let signed_block = SignedBlock::ssz_decode(block_bytes).map_err(|e| format!("block: {e}"))?;
    if state.validators.is_empty() {
        return Err("checkpoint state has an empty validator registry".into());
    }
    if signed_block.block.slot != state.slot {
        return Err(format!(
            "block slot {} != state slot {}",
            signed_block.block.slot.get(),
            state.slot.get()
        ));
    }
    let state_root = state.hash_tree_root().map_err(|e| e.to_string())?;
    if signed_block.block.state_root != state_root {
        return Err("block state_root does not commit to the state".into());
    }
    let block_root = signed_block
        .block
        .hash_tree_root()
        .map_err(|e| e.to_string())?;
    Ok(CheckpointAnchor {
        state,
        signed_block,
        block_root,
        block_bytes: block_bytes.to_vec(),
    })
}

/// Fetch and verify the anchor pair behind `url`.
pub fn fetch_checkpoint(url: &str) -> Result<CheckpointAnchor, String> {
    let s_url = states_url(url);
    let b_url = blocks_url(&s_url)?;
    let (status, state_bytes) = http_get(&s_url, CHECKPOINT_TIMEOUT)?;
    if status != 200 {
        return Err(format!("{s_url}: HTTP {status}"));
    }
    let (status, block_bytes) = http_get(&b_url, CHECKPOINT_TIMEOUT)?;
    if status != 200 {
        return Err(format!("{b_url}: HTTP {status}"));
    }
    verify_pair(&state_bytes, &block_bytes)
}

/// Start the chain from the anchor when it is ahead of the local head.
pub fn apply_anchor(owner: &mut ChainOwner, anchor: CheckpointAnchor) -> bool {
    let local_slot = owner.head_state.as_ref().map(|s| s.slot.get()).unwrap_or(0);
    let local_has_blocks = owner
        .head_state
        .as_ref()
        .is_some_and(|s| s.latest_block_header.slot.get() > 0);
    if local_has_blocks && anchor.state.slot.get() <= local_slot {
        info!(
            anchor_slot = anchor.state.slot.get(),
            local_slot, "checkpoint anchor is not ahead of the local head; keeping local chain"
        );
        return false;
    }
    info!(
        slot = anchor.state.slot.get(),
        root = %crate::persist_ssz::hex32(&anchor.block_root),
        validators = anchor.state.validators.len(),
        "starting from checkpoint anchor"
    );
    owner.head_state = Some(anchor.state);
    owner.remember_durable_block(anchor.block_root, anchor.block_bytes);
    owner.advance_head(anchor.block_root, anchor.signed_block.block.parent_root);
    owner.known_payloads.clear();
    true
}

#[cfg(test)]
#[path = "checkpoint_sync_tests.rs"]
mod tests;
