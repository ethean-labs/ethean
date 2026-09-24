//! Durable `--data-dir`: genesis.json/ssz, head SSZ, and `ethean.redb`.

use crate::chain_owner::ChainOwner;
use crate::chain_redb;
use crate::chain_snap::{GenesisPin, HeadSnap};
use crate::genesis_bundle;
use crate::persist_ssz;
use crate::{Error, Result};
use ethean_genesis::BuiltGenesis;
use ethean_primitives::Hash32;
use ethean_profile::ChainProfile;
use ethean_types::State;
use std::fs;
use tracing::{info, warn};

pub use crate::persist_paths::{reset_chain_files, PersistPaths};

/// Genesis loaded or created for a data-dir.
pub struct DurableGenesis {
    pub genesis_time: u64,
    pub validators: usize,
    pub state: State,
}

/// Restored canonical head.
pub struct RestoredHead {
    pub head_root: Hash32,
    pub state: State,
}

/// Load existing genesis or create a fixed one and write the Ethean bundle.
pub fn open_or_init(
    paths: &PersistPaths,
    profile: &ChainProfile,
    validators: usize,
) -> Result<DurableGenesis> {
    paths.ensure_dir()?;
    if let Some(doc) = genesis_bundle::load_genesis_json(paths)? {
        let n = doc.validator_count;
        warn_validator_mismatch(n, validators);
        let state = doc.into_state()?;
        fill_missing_genesis_ssz(paths, profile, &state)?;
        return Ok(DurableGenesis {
            genesis_time: state.config.genesis_time,
            validators: n.max(1),
            state,
        });
    }
    if let Some(state) = persist_ssz::load_genesis_ssz_file(paths)? {
        let n = state.validators.len();
        warn_validator_mismatch(n, validators);
        return Ok(DurableGenesis {
            genesis_time: state.config.genesis_time,
            validators: n.max(1),
            state,
        });
    }
    if let Some(pin) = load_legacy_pin(paths)? {
        warn_validator_mismatch(pin.validators, validators);
        let built = crate::local_genesis::fixed_devnet_genesis(pin.validators, pin.genesis_time)?;
        write_genesis_bundle(paths, profile, &built)?;
        return Ok(DurableGenesis {
            genesis_time: pin.genesis_time,
            validators: pin.validators,
            state: built.state,
        });
    }
    let genesis_time = crate::local_genesis::recent_genesis_time_secs(4, profile.seconds_per_slot);
    let n = validators.max(1);
    let built = crate::local_genesis::fixed_devnet_genesis(n, genesis_time)?;
    write_genesis_bundle(paths, profile, &built)?;
    info!(
        genesis_time,
        validators = n,
        "created data-dir genesis bundle"
    );
    Ok(DurableGenesis {
        genesis_time,
        validators: n,
        state: built.state,
    })
}

fn write_genesis_bundle(
    paths: &PersistPaths,
    profile: &ChainProfile,
    built: &BuiltGenesis,
) -> Result<()> {
    genesis_bundle::save_genesis_json(paths, profile, built)?;
    persist_ssz::save_genesis_ssz(paths, &built.state)?;
    let bytes = built
        .state
        .ssz_encode()
        .map_err(|e| Error::Config(format!("encode genesis for ethean.redb: {e}")))?;
    chain_redb::save_genesis(paths, &bytes)
}

fn fill_missing_genesis_ssz(
    paths: &PersistPaths,
    profile: &ChainProfile,
    state: &State,
) -> Result<()> {
    if paths.genesis_ssz().exists() {
        return Ok(());
    }
    let root = state
        .hash_tree_root()
        .map_err(|e| Error::Config(format!("genesis state root: {e}")))?;
    let built = BuiltGenesis {
        state: state.clone(),
        state_root: root,
    };
    persist_ssz::save_genesis_ssz(paths, state)?;
    genesis_bundle::save_genesis_json(paths, profile, &built)?;
    let bytes = state
        .ssz_encode()
        .map_err(|e| Error::Config(format!("encode genesis for ethean.redb: {e}")))?;
    chain_redb::save_genesis(paths, &bytes)
}

fn warn_validator_mismatch(pinned: usize, requested: usize) {
    if pinned != requested {
        warn!(
            pinned,
            requested, "data-dir genesis validators differ; using stored count"
        );
    }
}

fn load_legacy_pin(paths: &PersistPaths) -> Result<Option<GenesisPin>> {
    let path = paths.legacy_pin();
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)
        .map_err(|e| Error::Config(format!("read {}: {e}", path.display())))?;
    let pin: GenesisPin = serde_json::from_str(&raw)
        .map_err(|e| Error::Config(format!("parse {}: {e}", path.display())))?;
    info!(
        genesis_time = pin.genesis_time,
        validators = pin.validators,
        "migrating legacy genesis pin"
    );
    Ok(Some(pin))
}

/// Flush head to `state.ssz`, `head.root`, `blocks/*.ssz`, and `ethean.redb`.
pub fn save_head(paths: &PersistPaths, owner: &ChainOwner) -> Result<()> {
    let Some(state) = owner.head_state.as_ref() else {
        return Ok(());
    };
    persist_ssz::save_state_ssz(paths, state)?;
    persist_ssz::save_head_root(paths, &owner.head_root)?;

    let mut blocks: Vec<(Hash32, Vec<u8>)> = owner.durable_blocks.clone();
    if let Some(g) = owner.pending_block_gossip.as_ref() {
        if !blocks.iter().any(|(r, _)| *r == g.block_root) {
            blocks.push((g.block_root, g.payload.clone()));
        }
    }
    for (root, payload) in &blocks {
        persist_ssz::save_block_ssz(paths, root, payload)?;
    }

    let genesis_bytes = fs::read(paths.genesis_ssz()).ok();
    chain_redb::save_head(
        paths,
        &owner.head_root,
        state,
        genesis_bytes.as_deref(),
        &blocks,
    )
}

/// Load head: `ethean.redb`, then SSZ files, then legacy JSON snapshot.
pub fn load_head(paths: &PersistPaths) -> Result<Option<RestoredHead>> {
    if let Some(h) = chain_redb::load_head(paths)? {
        return Ok(Some(RestoredHead {
            head_root: h.head_root,
            state: h.state,
        }));
    }
    if let (Some(state), Some(head_root)) = (
        persist_ssz::load_state_ssz(paths)?,
        persist_ssz::load_head_root(paths)?,
    ) {
        return Ok(Some(RestoredHead { head_root, state }));
    }
    load_legacy_head(paths)
}

fn load_legacy_head(paths: &PersistPaths) -> Result<Option<RestoredHead>> {
    let path = paths.legacy_head();
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)
        .map_err(|e| Error::Config(format!("read {}: {e}", path.display())))?;
    let snap: HeadSnap = serde_json::from_str(&raw)
        .map_err(|e| Error::Config(format!("parse {}: {e}", path.display())))?;
    let (head_root, state) = snap
        .into_parts()
        .map_err(|e| Error::Config(format!("legacy head: {e}")))?;
    info!(slot = state.slot.get(), "loaded legacy head snapshot");
    Ok(Some(RestoredHead { head_root, state }))
}

/// Apply a loaded head onto the owner.
pub fn restore_owner(owner: &mut ChainOwner, head: RestoredHead) -> Result<()> {
    info!(
        slot = head.state.slot.get(),
        justified = head.state.latest_justified.slot.get(),
        finalized = head.state.latest_finalized.slot.get(),
        head_root = %persist_ssz::hex32(&head.head_root),
        "restored chain head from data-dir"
    );
    owner.head_root = head.head_root;
    owner.head_state = Some(head.state);
    owner.refresh_fc_view();
    owner.bump_generation();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::HASH32_ZERO;
    use ethean_profile::lstar_devnet;

    #[test]
    fn genesis_bundle_stable_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let paths = PersistPaths::new(dir.path());
        let profile = lstar_devnet().unwrap();
        let first = open_or_init(&paths, &profile, 4).unwrap();
        let again = open_or_init(&paths, &profile, 4).unwrap();
        assert_eq!(first.genesis_time, again.genesis_time);
        assert_eq!(first.validators, 4);
        assert!(paths.genesis_json().exists());
        assert!(paths.genesis_ssz().exists());
        assert!(paths.redb().exists());
    }

    #[test]
    fn head_roundtrip_redb_and_ssz() {
        let dir = tempfile::tempdir().unwrap();
        let paths = PersistPaths::new(dir.path());
        let profile = lstar_devnet().unwrap();
        let g = open_or_init(&paths, &profile, 2).unwrap();
        let mut owner = ChainOwner::new(0);
        owner.head_root = HASH32_ZERO;
        owner.head_state = Some(g.state);
        let blob = vec![1u8, 2, 3, 4];
        owner.remember_durable_block([7u8; 32], blob.clone());
        save_head(&paths, &owner).unwrap();
        let loaded = load_head(&paths).unwrap().unwrap();
        restore_owner(&mut owner, loaded).unwrap();
        assert_eq!(owner.head_root, HASH32_ZERO);
        assert!(paths.state_ssz().exists());
        let blocks = chain_redb::load_all_blocks(&paths).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].0, [7u8; 32]);
        assert_eq!(blocks[0].1, blob);
    }
}
