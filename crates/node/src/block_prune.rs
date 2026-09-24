//! Prune durable block blobs older than a finalized keep window.

use crate::persist_paths::PersistPaths;
use crate::serve_cache_seed::slot_from_block_ssz;
use crate::{Error, Result};
use ethean_primitives::Hash32;
use redb::{Database, ReadableTable, TableDefinition};
use std::fs;
use tracing::info;

const BLOCKS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("ethean_blocks");

/// Slots retained below the finalized checkpoint before pruning.
pub const KEEP_BELOW_FINALIZED: u64 = 256;

/// Summary of one prune pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlockPruneReport {
    /// Floor slot: blobs with `slot < floor` are eligible.
    pub floor_slot: u64,
    /// `blocks/*.ssz` files removed.
    pub files_removed: u32,
    /// `ethean.redb` block rows removed.
    pub redb_removed: u32,
}

fn map_redb(err: impl std::fmt::Display) -> Error {
    Error::Config(format!("ethean.redb prune: {err}"))
}

/// First slot that must be kept: `finalized.saturating_sub(keep)`.
pub fn prune_floor(finalized_slot: u64, keep_below_finalized: u64) -> u64 {
    finalized_slot.saturating_sub(keep_below_finalized)
}

/// Resolve keep window: CLI flag, then `ETHEAN_PRUNE_KEEP_SLOTS`, else [`KEEP_BELOW_FINALIZED`].
pub fn resolve_prune_keep_slots(cli: Option<u64>) -> u64 {
    if let Some(v) = cli {
        return v;
    }
    if let Ok(raw) = std::env::var("ETHEAN_PRUNE_KEEP_SLOTS") {
        if let Ok(v) = raw.trim().parse::<u64>() {
            return v;
        }
    }
    KEEP_BELOW_FINALIZED
}

/// Delete SSZ files and redb rows whose block slot is strictly below `floor_slot`.
pub fn prune_below_floor(paths: &PersistPaths, floor_slot: u64) -> Result<BlockPruneReport> {
    let mut report = BlockPruneReport {
        floor_slot,
        ..Default::default()
    };
    if floor_slot == 0 {
        return Ok(report);
    }
    report.files_removed = prune_ssz_files(paths, floor_slot)?;
    report.redb_removed = prune_redb_blocks(paths, floor_slot)?;
    if report.files_removed > 0 || report.redb_removed > 0 {
        info!(
            floor_slot,
            files_removed = report.files_removed,
            redb_removed = report.redb_removed,
            "pruned durable blocks below finalized keep window"
        );
    }
    Ok(report)
}

fn prune_ssz_files(paths: &PersistPaths, floor_slot: u64) -> Result<u32> {
    let dir = paths.blocks_dir();
    if !dir.exists() {
        return Ok(0);
    }
    let entries = fs::read_dir(&dir)
        .map_err(|e| Error::Config(format!("read blocks dir {}: {e}", dir.display())))?;
    let mut removed = 0u32;
    for entry in entries {
        let entry = entry
            .map_err(|e| Error::Config(format!("read blocks entry {}: {e}", dir.display())))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("ssz") {
            continue;
        }
        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let Some(slot) = slot_from_block_ssz(&bytes) else {
            continue;
        };
        if slot >= floor_slot {
            continue;
        }
        match fs::remove_file(&path) {
            Ok(()) => removed = removed.saturating_add(1),
            Err(e) => {
                return Err(Error::Config(format!("remove {}: {e}", path.display())));
            }
        }
    }
    Ok(removed)
}

fn prune_redb_blocks(paths: &PersistPaths, floor_slot: u64) -> Result<u32> {
    if !paths.redb().exists() {
        return Ok(0);
    }
    let db = Database::open(paths.redb()).map_err(map_redb)?;
    let mut doomed: Vec<Hash32> = Vec::new();
    {
        let txn = db.begin_read().map_err(map_redb)?;
        let table = match txn.open_table(BLOCKS) {
            Ok(t) => t,
            Err(_) => return Ok(0),
        };
        for item in table.iter().map_err(map_redb)? {
            let (k, v) = item.map_err(map_redb)?;
            let key = k.value();
            if key.len() != 32 {
                continue;
            }
            let Some(slot) = slot_from_block_ssz(v.value()) else {
                continue;
            };
            if slot < floor_slot {
                let mut root = [0u8; 32];
                root.copy_from_slice(key);
                doomed.push(root);
            }
        }
    }
    if doomed.is_empty() {
        return Ok(0);
    }
    let txn = db.begin_write().map_err(map_redb)?;
    {
        let mut table = txn.open_table(BLOCKS).map_err(map_redb)?;
        for root in &doomed {
            let _ = table.remove(root.as_slice()).map_err(map_redb)?;
        }
    }
    txn.commit().map_err(map_redb)?;
    Ok(doomed.len() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persist_ssz;
    use ethean_primitives::{Slot, ValidatorIndex};
    use ethean_types::{Block, BlockBody, MultiMessageAggregate, SignedBlock};

    fn signed_at(slot: u64) -> (Hash32, Vec<u8>) {
        let block = Block {
            slot: Slot::new(slot),
            proposer_index: ValidatorIndex::new(0),
            parent_root: [1u8; 32],
            state_root: [2u8; 32],
            body: BlockBody::default(),
        };
        let root = [slot as u8; 32];
        let signed = SignedBlock::new(block, MultiMessageAggregate::default());
        (root, signed.ssz_encode().unwrap())
    }

    #[test]
    fn floor_saturates() {
        assert_eq!(prune_floor(10, 256), 0);
        assert_eq!(prune_floor(300, 256), 44);
    }

    #[test]
    fn resolve_prefers_cli_over_default() {
        assert_eq!(resolve_prune_keep_slots(Some(64)), 64);
        // Without CLI and without a valid env override, default keep applies.
        let _ = std::env::remove_var("ETHEAN_PRUNE_KEEP_SLOTS");
        assert_eq!(resolve_prune_keep_slots(None), KEEP_BELOW_FINALIZED);
    }

    #[test]
    fn prunes_old_ssz_keeps_recent() {
        let dir = tempfile::tempdir().unwrap();
        let paths = PersistPaths::new(dir.path());
        paths.ensure_dir().unwrap();
        let (old_root, old_enc) = signed_at(1);
        let (new_root, new_enc) = signed_at(100);
        persist_ssz::save_block_ssz(&paths, &old_root, &old_enc).unwrap();
        persist_ssz::save_block_ssz(&paths, &new_root, &new_enc).unwrap();
        let report = prune_below_floor(&paths, 50).unwrap();
        assert_eq!(report.files_removed, 1);
        assert!(!paths.block_ssz(&persist_ssz::hex32(&old_root)).exists());
        assert!(paths.block_ssz(&persist_ssz::hex32(&new_root)).exists());
    }
}
