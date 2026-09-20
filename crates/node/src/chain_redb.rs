//! `ethean.redb` — durable SSZ blobs keyed by root under `--data-dir`.

use crate::persist_paths::PersistPaths;
use crate::{Error, Result};
use ethean_primitives::Hash32;
use ethean_storage::SCHEMA_ID;
use ethean_types::State;
use redb::{Database, ReadableTable, TableDefinition};
use tracing::info;

const META: TableDefinition<&str, &[u8]> = TableDefinition::new("ethean_meta");
const STATES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("ethean_states");
const BLOCKS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("ethean_blocks");

const KEY_SCHEMA: &str = "schema";
const KEY_HEAD: &str = "head";
const KEY_GENESIS: &str = "genesis";

/// Head restored from `ethean.redb`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedbHead {
    pub head_root: Hash32,
    pub state: State,
}

fn map_redb(err: impl std::fmt::Display) -> Error {
    Error::Config(format!("ethean.redb: {err}"))
}

fn open_db(paths: &PersistPaths) -> Result<Database> {
    paths.ensure_dir()?;
    let path = paths.redb();
    if path.exists() {
        Database::open(&path).map_err(map_redb)
    } else {
        Database::create(&path).map_err(map_redb)
    }
}

/// Store genesis SSZ, head root, and head state (optional block blob).
pub fn save_head(
    paths: &PersistPaths,
    head_root: &Hash32,
    state: &State,
    genesis_ssz: Option<&[u8]>,
    block: Option<(&[u8], &[u8])>,
) -> Result<()> {
    let state_bytes = state
        .ssz_encode()
        .map_err(|e| Error::Config(format!("encode state for ethean.redb: {e}")))?;
    let db = open_db(paths)?;
    let txn = db.begin_write().map_err(map_redb)?;
    {
        let mut meta = txn.open_table(META).map_err(map_redb)?;
        meta.insert(KEY_SCHEMA, SCHEMA_ID.as_bytes())
            .map_err(map_redb)?;
        meta.insert(KEY_HEAD, head_root.as_slice()).map_err(map_redb)?;
        if let Some(g) = genesis_ssz {
            meta.insert(KEY_GENESIS, g).map_err(map_redb)?;
        }
        let mut states = txn.open_table(STATES).map_err(map_redb)?;
        states
            .insert(head_root.as_slice(), state_bytes.as_slice())
            .map_err(map_redb)?;
        if let Some((root, payload)) = block {
            if !payload.is_empty() {
                let mut blocks = txn.open_table(BLOCKS).map_err(map_redb)?;
                blocks.insert(root, payload).map_err(map_redb)?;
            }
        }
    }
    txn.commit().map_err(map_redb)?;
    Ok(())
}

/// Load the last head from `ethean.redb`.
pub fn load_head(paths: &PersistPaths) -> Result<Option<RedbHead>> {
    if !paths.redb().exists() {
        return Ok(None);
    }
    let db = open_db(paths)?;
    let txn = db.begin_read().map_err(map_redb)?;
    let meta = match txn.open_table(META) {
        Ok(t) => t,
        Err(_) => return Ok(None),
    };
    if let Some(schema) = meta.get(KEY_SCHEMA).map_err(map_redb)? {
        let got = std::str::from_utf8(schema.value()).unwrap_or("");
        if got != SCHEMA_ID {
            return Err(Error::Config(format!(
                "ethean.redb schema {got} is not {SCHEMA_ID}"
            )));
        }
    }
    let Some(head) = meta.get(KEY_HEAD).map_err(map_redb)? else {
        return Ok(None);
    };
    let head_bytes = head.value();
    if head_bytes.len() != 32 {
        return Err(Error::Config("ethean.redb head root is not 32 bytes".into()));
    }
    let mut head_root = [0u8; 32];
    head_root.copy_from_slice(head_bytes);
    let states = match txn.open_table(STATES) {
        Ok(t) => t,
        Err(_) => return Ok(None),
    };
    let Some(blob) = states.get(head_root.as_slice()).map_err(map_redb)? else {
        return Ok(None);
    };
    let state = State::ssz_decode(blob.value())
        .map_err(|e| Error::Config(format!("decode ethean.redb state: {e}")))?;
    info!(
        path = %paths.redb().display(),
        slot = state.slot.get(),
        "loaded head from ethean.redb"
    );
    Ok(Some(RedbHead { head_root, state }))
}

/// Persist genesis SSZ on first open (no head yet).
pub fn save_genesis(paths: &PersistPaths, genesis_ssz: &[u8]) -> Result<()> {
    let db = open_db(paths)?;
    let txn = db.begin_write().map_err(map_redb)?;
    {
        let mut meta = txn.open_table(META).map_err(map_redb)?;
        meta.insert(KEY_SCHEMA, SCHEMA_ID.as_bytes())
            .map_err(map_redb)?;
        meta.insert(KEY_GENESIS, genesis_ssz).map_err(map_redb)?;
    }
    txn.commit().map_err(map_redb)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persist_paths::PersistPaths;
    use ethean_genesis::GenesisBuilder;
    use ethean_primitives::{Bytes52, HASH32_ZERO};

    #[test]
    fn head_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let paths = PersistPaths::new(dir.path());
        let built = GenesisBuilder::new(42)
            .push_validator(Bytes52::ZERO, Bytes52::ZERO)
            .build()
            .unwrap();
        let gssz = built.state.ssz_encode().unwrap();
        save_head(&paths, &HASH32_ZERO, &built.state, Some(&gssz), None).unwrap();
        let loaded = load_head(&paths).unwrap().unwrap();
        assert_eq!(loaded.head_root, HASH32_ZERO);
        assert_eq!(loaded.state.config.genesis_time, 42);
        assert!(paths.redb().exists());
    }
}
