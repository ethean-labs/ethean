//! Seed QuicSwarm blocks-by-range serve cache from durable `--data-dir`.

use crate::chain_redb;
use crate::persist_paths::PersistPaths;
use crate::persist_ssz;
use ethean_primitives::Hash32;
use ethean_types::{Block, SignedBlock};
use std::collections::HashSet;
#[cfg(feature = "libp2p-quic")]
use tracing::info;

/// How many slot entries were indexed into the serve cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ServeCacheSeed {
    /// Blobs considered from redb + `blocks/*.ssz`.
    pub candidates: u32,
    /// Successfully indexed by slot.
    pub indexed: u32,
}

/// Decode slot from a SignedBlock or bare Block SSZ blob.
pub fn slot_from_block_ssz(payload: &[u8]) -> Option<u64> {
    if let Ok(signed) = SignedBlock::ssz_decode(payload) {
        return Some(signed.block.slot.get());
    }
    if let Ok(block) = Block::ssz_decode(payload) {
        return Some(block.slot.get());
    }
    None
}

/// Collect unique (root, payload) pairs from redb then filesystem (redb wins).
pub fn load_persisted_block_blobs(paths: &PersistPaths) -> Vec<(Hash32, Vec<u8>)> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    if let Ok(redb) = chain_redb::load_all_blocks(paths) {
        for (root, bytes) in redb {
            if seen.insert(root) {
                out.push((root, bytes));
            }
        }
    }
    if let Ok(files) = persist_ssz::load_block_ssz_dir(paths) {
        for (root, bytes) in files {
            if seen.insert(root) {
                out.push((root, bytes));
            }
        }
    }
    out
}

/// Index persisted blocks into `put_block_at_slot` for inbound range replies.
#[cfg(feature = "libp2p-quic")]
pub fn seed_facade_from_data_dir(
    facade: &mut crate::network::SwarmFacade,
    paths: &PersistPaths,
) -> ServeCacheSeed {
    let blobs = load_persisted_block_blobs(paths);
    let mut seed = ServeCacheSeed {
        candidates: blobs.len() as u32,
        indexed: 0,
    };
    for (root, bytes) in blobs {
        let Some(slot) = slot_from_block_ssz(&bytes) else {
            continue;
        };
        if facade.put_block_at_slot(slot, root, bytes).is_ok() {
            seed.indexed = seed.indexed.saturating_add(1);
        }
    }
    if seed.candidates > 0 || seed.indexed > 0 {
        info!(
            candidates = seed.candidates,
            indexed = seed.indexed,
            path = %paths.root.display(),
            "seeded blocks-by-range serve cache from data-dir"
        );
    }
    seed
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Slot, ValidatorIndex};
    use ethean_types::{Block, BlockBody, MultiMessageAggregate, SignedBlock};

    #[test]
    fn slot_from_signed_block() {
        let block = Block {
            slot: Slot::new(9),
            proposer_index: ValidatorIndex::new(0),
            parent_root: [1u8; 32],
            state_root: [2u8; 32],
            body: BlockBody::default(),
        };
        let signed = SignedBlock::new(block, MultiMessageAggregate::default());
        let enc = signed.ssz_encode().unwrap();
        assert_eq!(slot_from_block_ssz(&enc), Some(9));
    }

    #[test]
    fn load_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let paths = PersistPaths::new(dir.path());
        assert!(load_persisted_block_blobs(&paths).is_empty());
    }

    #[test]
    fn loads_block_ssz_file() {
        let dir = tempfile::tempdir().unwrap();
        let paths = PersistPaths::new(dir.path());
        paths.ensure_dir().unwrap();
        let block = Block {
            slot: Slot::new(3),
            proposer_index: ValidatorIndex::new(0),
            parent_root: [1u8; 32],
            state_root: [2u8; 32],
            body: BlockBody::default(),
        };
        let signed = SignedBlock::new(block, MultiMessageAggregate::default());
        let root = [9u8; 32];
        let enc = signed.ssz_encode().unwrap();
        persist_ssz::save_block_ssz(&paths, &root, &enc).unwrap();
        let blobs = load_persisted_block_blobs(&paths);
        assert_eq!(blobs.len(), 1);
        assert_eq!(blobs[0].0, root);
        assert_eq!(slot_from_block_ssz(&blobs[0].1), Some(3));
    }
}
