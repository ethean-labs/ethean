//! SSZ files under `--data-dir` (`genesis.ssz`, `state.ssz`, `blocks/*.ssz`).

use crate::persist_paths::{write_atomic, PersistPaths};
use crate::{Error, Result};
use ethean_primitives::Hash32;
use ethean_types::State;
use std::fs;
use tracing::info;

/// Encode and write genesis state SSZ (once per chain identity).
pub fn save_genesis_ssz(paths: &PersistPaths, state: &State) -> Result<()> {
    paths.ensure_dir()?;
    let bytes = state
        .ssz_encode()
        .map_err(|e| Error::Config(format!("encode genesis.ssz: {e}")))?;
    write_atomic(&paths.genesis_ssz(), &bytes)?;
    info!(path = %paths.genesis_ssz().display(), bytes = bytes.len(), "wrote genesis.ssz");
    Ok(())
}

/// Decode genesis state from `genesis.ssz` when present.
pub fn load_genesis_ssz_file(paths: &PersistPaths) -> Result<Option<State>> {
    let path = paths.genesis_ssz();
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path)
        .map_err(|e| Error::Config(format!("read {}: {e}", path.display())))?;
    let state = ethean_genesis::load_genesis_ssz(&bytes, None)
        .map_err(|e| Error::Config(format!("decode genesis.ssz: {e}")))?;
    info!(
        path = %path.display(),
        slot = state.slot.get(),
        validators = state.validators.len(),
        "loaded genesis.ssz"
    );
    Ok(Some(state))
}

/// Encode and write the current head state.
pub fn save_state_ssz(paths: &PersistPaths, state: &State) -> Result<()> {
    paths.ensure_dir()?;
    let bytes = state
        .ssz_encode()
        .map_err(|e| Error::Config(format!("encode state.ssz: {e}")))?;
    write_atomic(&paths.state_ssz(), &bytes)?;
    Ok(())
}

/// Decode head state from `state.ssz`.
pub fn load_state_ssz(paths: &PersistPaths) -> Result<Option<State>> {
    let path = paths.state_ssz();
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path)
        .map_err(|e| Error::Config(format!("read {}: {e}", path.display())))?;
    let state = State::ssz_decode(&bytes)
        .map_err(|e| Error::Config(format!("decode state.ssz: {e}")))?;
    info!(
        path = %path.display(),
        slot = state.slot.get(),
        justified = state.latest_justified.slot.get(),
        finalized = state.latest_finalized.slot.get(),
        "loaded state.ssz"
    );
    Ok(Some(state))
}

/// Persist the canonical head root as hex (`head.root`).
pub fn save_head_root(paths: &PersistPaths, root: &Hash32) -> Result<()> {
    paths.ensure_dir()?;
    let hex = hex32(root);
    write_atomic(&paths.head_root(), hex.as_bytes())?;
    Ok(())
}

/// Load `head.root` if present.
pub fn load_head_root(paths: &PersistPaths) -> Result<Option<Hash32>> {
    let path = paths.head_root();
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)
        .map_err(|e| Error::Config(format!("read {}: {e}", path.display())))?;
    Ok(Some(parse_hex32(raw.trim())?))
}

/// Write a signed/block SSZ blob keyed by block root hex.
pub fn save_block_ssz(paths: &PersistPaths, root: &Hash32, payload: &[u8]) -> Result<()> {
    if payload.is_empty() {
        return Ok(());
    }
    paths.ensure_dir()?;
    write_atomic(&paths.block_ssz(&hex32(root)), payload)
}

pub fn hex32(h: &Hash32) -> String {
    h.iter().map(|b| format!("{b:02x}")).collect()
}

fn parse_hex32(s: &str) -> Result<Hash32> {
    let s = s.trim().trim_start_matches("0x");
    if s.len() != 64 {
        return Err(Error::Config(format!(
            "head.root expected 32-byte hex, got len {}",
            s.len()
        )));
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)
            .map_err(|e| Error::Config(format!("head.root hex: {e}")))?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Bytes52, HASH32_ZERO};
    use ethean_genesis::GenesisBuilder;

    #[test]
    fn genesis_and_state_ssz_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let paths = PersistPaths::new(dir.path());
        let built = GenesisBuilder::new(1_700_000_000)
            .push_validator(Bytes52::ZERO, Bytes52::ZERO)
            .build()
            .unwrap();
        save_genesis_ssz(&paths, &built.state).unwrap();
        let loaded = load_genesis_ssz_file(&paths).unwrap().unwrap();
        assert_eq!(loaded.config.genesis_time, 1_700_000_000);
        save_state_ssz(&paths, &built.state).unwrap();
        save_head_root(&paths, &HASH32_ZERO).unwrap();
        assert_eq!(load_head_root(&paths).unwrap().unwrap(), HASH32_ZERO);
        assert_eq!(load_state_ssz(&paths).unwrap().unwrap().slot.get(), 0);
    }
}
