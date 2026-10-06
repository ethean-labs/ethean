//! Prune durable block blobs older than a finalized keep window.

use crate::persist_paths::PersistPaths;
use crate::persist_ssz;
use crate::serve_cache_seed::slot_from_block_ssz;
use crate::{Error, Result};
use ethean_primitives::Hash32;
use std::fs;
use tracing::info;

/// Slots retained below the finalized checkpoint before pruning.
pub const KEEP_BELOW_FINALIZED: u64 = 256;

/// Floor advance that triggers the next prune pass. Each pass decodes every
/// stored block, so it runs in batches instead of on every flush.
pub const PRUNE_FLOOR_STEP: u64 = 32;

/// True once `floor` has moved at least [`PRUNE_FLOOR_STEP`] past the last pass.
pub fn prune_due(last_floor: u64, floor: u64) -> bool {
    floor >= last_floor.saturating_add(PRUNE_FLOOR_STEP)
}

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
    let mut known = Vec::new();
    let below = collect_block_slots(paths, floor_slot, &mut known)?;
    report.files_removed = remove_block_files(paths, &below)?;
    report.redb_removed = prune_redb_blocks(paths, floor_slot, &known)?;
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

/// Roots of block files whose slot is strictly below `floor_slot`.
///
/// The slot is read from the first bytes of each file, so nothing is decoded.
/// `block_root_slot` receives each root with its slot so a caller can remember
/// them (the redb slot index is filled from this pass).
pub fn collect_block_slots(
    paths: &PersistPaths,
    floor_slot: u64,
    block_root_slot: &mut Vec<(Hash32, u64)>,
) -> Result<Vec<Hash32>> {
    let dir = paths.blocks_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let entries = fs::read_dir(&dir)
        .map_err(|e| Error::Config(format!("read blocks dir {}: {e}", dir.display())))?;
    let mut below = Vec::new();
    for entry in entries {
        let entry = entry
            .map_err(|e| Error::Config(format!("read blocks entry {}: {e}", dir.display())))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("ssz") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some(root) = root_from_hex(stem) else {
            continue;
        };
        let Some(slot) = slot_of_file_head(&path) else {
            continue;
        };
        block_root_slot.push((root, slot));
        if slot < floor_slot {
            below.push(root);
        }
    }
    Ok(below)
}

fn root_from_hex(hex: &str) -> Option<Hash32> {
    if hex.len() != 64 {
        return None;
    }
    let mut root = [0u8; 32];
    for (i, byte) in root.iter_mut().enumerate() {
        *byte = u8::from_str_radix(hex.get(i * 2..i * 2 + 2)?, 16).ok()?;
    }
    Some(root)
}

/// Slot from the first 16 bytes of a block file.
fn slot_of_file_head(path: &std::path::Path) -> Option<u64> {
    use std::io::Read;
    let mut file = fs::File::open(path).ok()?;
    let mut head = [0u8; 16];
    file.read_exact(&mut head).ok()?;
    slot_from_block_ssz(&head)
}

fn remove_block_files(paths: &PersistPaths, roots: &[Hash32]) -> Result<u32> {
    let mut removed = 0u32;
    for root in roots {
        let path = paths.block_ssz(&persist_ssz::hex32(root));
        match fs::remove_file(&path) {
            Ok(()) => removed = removed.saturating_add(1),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(Error::Config(format!("remove {}: {e}", path.display()))),
        }
    }
    Ok(removed)
}

fn prune_redb_blocks(
    paths: &PersistPaths,
    floor_slot: u64,
    known: &[(Hash32, u64)],
) -> Result<u32> {
    let doomed = crate::chain_redb::block_roots_below(paths, floor_slot, known)?;
    crate::chain_redb::remove_blocks(paths, &doomed)
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn prune_runs_in_floor_batches() {
        assert!(!prune_due(0, 0));
        assert!(!prune_due(0, PRUNE_FLOOR_STEP - 1));
        assert!(prune_due(0, PRUNE_FLOOR_STEP));
        assert!(!prune_due(100, 100 + PRUNE_FLOOR_STEP - 1));
        assert!(prune_due(100, 100 + PRUNE_FLOOR_STEP));
    }

    #[test]
    fn resolve_prefers_cli_over_default() {
        assert_eq!(resolve_prune_keep_slots(Some(64)), 64);
        // Without CLI and without a valid env override, default keep applies.
        std::env::remove_var("ETHEAN_PRUNE_KEEP_SLOTS");
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

    #[test]
    fn pruning_redb_uses_the_slot_index_and_keeps_it_current() {
        let dir = tempfile::tempdir().unwrap();
        let paths = PersistPaths::new(dir.path());
        paths.ensure_dir().unwrap();
        let (old_root, old_enc) = signed_at(1);
        let (new_root, new_enc) = signed_at(100);
        // Rows saved before the slot index existed carry no slot.
        crate::chain_redb::save_head(&paths, &[0u8; 32], &signed_state(), None, &[]).unwrap();
        {
            let db = redb::Database::open(paths.redb()).unwrap();
            let txn = db.begin_write().unwrap();
            {
                let mut table = txn
                    .open_table(redb::TableDefinition::<&[u8], &[u8]>::new("ethean_blocks"))
                    .unwrap();
                table
                    .insert(old_root.as_slice(), old_enc.as_slice())
                    .unwrap();
                table
                    .insert(new_root.as_slice(), new_enc.as_slice())
                    .unwrap();
            }
            txn.commit().unwrap();
        }
        persist_ssz::save_block_ssz(&paths, &old_root, &old_enc).unwrap();
        persist_ssz::save_block_ssz(&paths, &new_root, &new_enc).unwrap();

        let report = prune_below_floor(&paths, 50).unwrap();
        assert_eq!(report.files_removed, 1);
        assert_eq!(report.redb_removed, 1);
        let left = crate::chain_redb::load_all_blocks(&paths).unwrap();
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].0, new_root);

        // The kept row now has a slot entry, so a later pass needs no files.
        std::fs::remove_dir_all(paths.blocks_dir()).unwrap();
        assert_eq!(prune_below_floor(&paths, 50).unwrap().redb_removed, 0);
        assert_eq!(prune_below_floor(&paths, 200).unwrap().redb_removed, 1);
        assert!(crate::chain_redb::load_all_blocks(&paths)
            .unwrap()
            .is_empty());
    }

    fn signed_state() -> ethean_types::State {
        use ethean_primitives::{Bytes52, Slot, ValidatorIndex};
        use ethean_types::{BlockHeader, Checkpoint, GenesisConfig, State, Validator};
        State {
            config: GenesisConfig::new(1),
            slot: Slot::ZERO,
            latest_block_header: BlockHeader::default(),
            latest_justified: Checkpoint::genesis(),
            latest_finalized: Checkpoint::genesis(),
            historical_block_hashes: Vec::new(),
            justified_slots: Vec::new(),
            validators: vec![
                Validator::new(Bytes52::ZERO, Bytes52::ZERO, ValidatorIndex::new(0)).unwrap(),
            ],
            justifications_roots: Vec::new(),
            justifications_validators: Vec::new(),
        }
    }
}
