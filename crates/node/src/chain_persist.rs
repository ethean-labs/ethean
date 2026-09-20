//! File-backed genesis pin + head resume (peer-like local data-dir).

use crate::chain_snap::{GenesisPin, HeadSnap};
use crate::chain_owner::ChainOwner;
use crate::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

const PIN_FILE: &str = "genesis_pin.json";
const HEAD_FILE: &str = "head_snap.json";

/// Paths under a durable data directory.
#[derive(Debug, Clone)]
pub struct PersistPaths {
    pub root: PathBuf,
}

impl PersistPaths {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn pin_path(&self) -> PathBuf {
        self.root.join(PIN_FILE)
    }

    pub fn head_path(&self) -> PathBuf {
        self.root.join(HEAD_FILE)
    }

    pub fn ensure_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.root).map_err(|e| {
            Error::Config(format!("create data-dir {}: {e}", self.root.display()))
        })
    }
}

/// Load pin or create a new fixed genesis time (lookback from now once).
pub fn load_or_create_pin(
    paths: &PersistPaths,
    validators: usize,
    seconds_per_slot: u64,
) -> Result<GenesisPin> {
    paths.ensure_dir()?;
    let path = paths.pin_path();
    if path.exists() {
        let raw = fs::read_to_string(&path)
            .map_err(|e| Error::Config(format!("read {}: {e}", path.display())))?;
        let pin: GenesisPin = serde_json::from_str(&raw)
            .map_err(|e| Error::Config(format!("parse {}: {e}", path.display())))?;
        if pin.validators != validators {
            warn!(
                pinned = pin.validators,
                requested = validators,
                "data-dir genesis pin validators differ; using pinned count"
            );
        }
        info!(
            genesis_time = pin.genesis_time,
            validators = pin.validators,
            path = %path.display(),
            "loaded fixed genesis pin"
        );
        return Ok(pin);
    }
    let genesis_time =
        crate::local_genesis::recent_genesis_time_secs(4, seconds_per_slot);
    let pin = GenesisPin {
        genesis_time,
        validators: validators.max(1),
        seconds_per_slot,
    };
    save_pin(paths, &pin)?;
    info!(
        genesis_time = pin.genesis_time,
        validators = pin.validators,
        path = %path.display(),
        "created fixed genesis pin"
    );
    Ok(pin)
}

/// Persist the pin (first start or explicit rewrite).
pub fn save_pin(paths: &PersistPaths, pin: &GenesisPin) -> Result<()> {
    paths.ensure_dir()?;
    let path = paths.pin_path();
    let raw = serde_json::to_string_pretty(pin)
        .map_err(|e| Error::Config(format!("encode genesis pin: {e}")))?;
    fs::write(&path, raw)
        .map_err(|e| Error::Config(format!("write {}: {e}", path.display())))?;
    Ok(())
}

/// Load head snapshot if present.
pub fn load_head(paths: &PersistPaths) -> Result<Option<HeadSnap>> {
    let path = paths.head_path();
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)
        .map_err(|e| Error::Config(format!("read {}: {e}", path.display())))?;
    let snap: HeadSnap = serde_json::from_str(&raw)
        .map_err(|e| Error::Config(format!("parse {}: {e}", path.display())))?;
    info!(
        slot = snap.state.slot,
        justified = snap.state.latest_justified.slot,
        finalized = snap.state.latest_finalized.slot,
        path = %path.display(),
        "loaded head snapshot"
    );
    Ok(Some(snap))
}

/// Write head snapshot from the chain owner.
pub fn save_head(paths: &PersistPaths, owner: &ChainOwner) -> Result<()> {
    let Some(state) = owner.head_state.as_ref() else {
        return Ok(());
    };
    paths.ensure_dir()?;
    let snap = HeadSnap::from_owner(&owner.head_root, state);
    let path = paths.head_path();
    let raw = serde_json::to_string_pretty(&snap)
        .map_err(|e| Error::Config(format!("encode head snap: {e}")))?;
    fs::write(&path, raw)
        .map_err(|e| Error::Config(format!("write {}: {e}", path.display())))?;
    Ok(())
}

/// Apply a loaded snapshot onto the owner (replaces genesis seal).
pub fn restore_owner(owner: &mut ChainOwner, snap: HeadSnap) -> Result<()> {
    let (head_root, state) = snap
        .into_parts()
        .map_err(|e| Error::Config(format!("restore head: {e}")))?;
    info!(
        slot = state.slot.get(),
        justified = state.latest_justified.slot.get(),
        finalized = state.latest_finalized.slot.get(),
        head_root = %format_root(&head_root),
        "restored chain head from data-dir"
    );
    owner.head_root = head_root;
    owner.head_state = Some(state);
    owner.bump_generation();
    Ok(())
}

/// Delete pin + head so the next start creates a fresh fixed genesis.
pub fn reset_chain_files(root: &Path) -> Result<()> {
    for name in [PIN_FILE, HEAD_FILE] {
        let p = root.join(name);
        if p.exists() {
            fs::remove_file(&p)
                .map_err(|e| Error::Config(format!("remove {}: {e}", p.display())))?;
            info!(path = %p.display(), "removed chain file");
        }
    }
    Ok(())
}

fn format_root(root: &ethean_primitives::Hash32) -> String {
    root.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::HASH32_ZERO;
    use ethean_types::State;

    #[test]
    fn pin_roundtrip_tempdir() {
        let dir = tempfile::tempdir().unwrap();
        let paths = PersistPaths::new(dir.path());
        let pin = load_or_create_pin(&paths, 4, 4).unwrap();
        let again = load_or_create_pin(&paths, 4, 4).unwrap();
        assert_eq!(pin, again);
        assert_eq!(pin.validators, 4);
    }

    #[test]
    fn head_roundtrip_tempdir() {
        let dir = tempfile::tempdir().unwrap();
        let paths = PersistPaths::new(dir.path());
        let mut owner = ChainOwner::new(0);
        owner.head_root = HASH32_ZERO;
        owner.head_state = Some(State::default());
        save_head(&paths, &owner).unwrap();
        let loaded = load_head(&paths).unwrap().unwrap();
        restore_owner(&mut owner, loaded).unwrap();
        assert_eq!(owner.head_root, HASH32_ZERO);
    }
}
