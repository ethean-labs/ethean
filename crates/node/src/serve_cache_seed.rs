//! Seed QuicSwarm blocks-by-range serve cache from durable `--data-dir`.

use crate::chain_redb;
use crate::persist_paths::PersistPaths;
use crate::persist_ssz;
use ethean_primitives::Hash32;
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

/// Slot of a block SSZ blob, read from the leading bytes.
///
/// A `SignedBlock` starts with two 4-byte offsets and then the `Block`, whose
/// first field is the slot. A bare `Block` starts with the slot directly.
pub fn slot_from_block_ssz(payload: &[u8]) -> Option<u64> {
    if payload.len() >= 16 {
        let block_at = u32::from_le_bytes(payload[..4].try_into().ok()?) as usize;
        if block_at == 8 && payload.len() >= block_at + 8 {
            return Some(u64::from_le_bytes(
                payload[block_at..block_at + 8].try_into().ok()?,
            ));
        }
    }
    if payload.len() >= 8 {
        return Some(u64::from_le_bytes(payload[..8].try_into().ok()?));
    }
    None
}

/// Full decode, kept to cross-check [`slot_from_block_ssz`] in tests.
#[cfg(test)]
fn slot_from_full_decode(payload: &[u8]) -> Option<u64> {
    use ethean_types::{Block, SignedBlock};
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

/// Read `blocks/<root>.ssz` for blocks that left the in-memory serve cache.
pub fn block_file_loader(paths: &PersistPaths) -> ethean_network::serve_cache::BlockLoader {
    let paths = paths.clone();
    std::sync::Arc::new(move |root: &Hash32| {
        std::fs::read(paths.block_ssz(&persist_ssz::hex32(root)))
            .ok()
            .filter(|b| !b.is_empty())
    })
}

/// Index persisted blocks into `put_block_at_slot` for inbound range replies.
#[cfg(feature = "libp2p-quic")]
pub fn seed_facade_from_data_dir(
    facade: &mut crate::network::SwarmFacade,
    paths: &PersistPaths,
) -> ServeCacheSeed {
    let _ = facade.set_block_loader(block_file_loader(paths));
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

/// Gossip blocks the chain imported, as `(slot, root, payload)` for the serve
/// cache. Without them a node only serves its own proposals, and a peer
/// catching up gets empty blocks-by-root replies for everything else.
pub fn imported_gossip_blocks(
    owner: &crate::chain_owner::ChainOwner,
    accepted: &[crate::network::GossipIngress],
) -> Vec<(u64, Hash32, Vec<u8>)> {
    accepted
        .iter()
        .filter_map(|g| {
            let plain = g.plain.as_deref()?;
            let decoded = crate::gossip_decode::try_decode_block(&g.topic, plain)?;
            let known = owner.head_root == decoded.root
                || owner
                    .fc
                    .as_ref()
                    .is_some_and(|fc| fc.blocks.contains_key(&decoded.root));
            known.then(|| (decoded.block.slot.get(), decoded.root, plain.to_vec()))
        })
        .collect()
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
        let bare = block.ssz_encode().unwrap();
        let signed = SignedBlock::new(block, MultiMessageAggregate::default());
        let enc = signed.ssz_encode().unwrap();
        assert_eq!(slot_from_block_ssz(&enc), Some(9));
        assert_eq!(slot_from_block_ssz(&enc), slot_from_full_decode(&enc));
        assert_eq!(slot_from_block_ssz(&bare), Some(9));
        assert_eq!(slot_from_block_ssz(&bare), slot_from_full_decode(&bare));
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

    #[test]
    fn file_loader_reads_persisted_blocks_only() {
        let dir = tempfile::tempdir().unwrap();
        let paths = PersistPaths::new(dir.path());
        paths.ensure_dir().unwrap();
        let root = [7u8; 32];
        persist_ssz::save_block_ssz(&paths, &root, &[1, 2, 3]).unwrap();
        let loader = block_file_loader(&paths);
        assert_eq!(loader(&root), Some(vec![1, 2, 3]));
        assert_eq!(loader(&[8u8; 32]), None);
    }

    fn block_gossip(slot: u64) -> (Hash32, crate::network::GossipIngress) {
        let block = Block {
            slot: Slot::new(slot),
            proposer_index: ValidatorIndex::new(0),
            parent_root: [1u8; 32],
            state_root: [2u8; 32],
            body: BlockBody::default(),
        };
        let root = block.hash_tree_root().unwrap();
        let signed = SignedBlock::new(block, MultiMessageAggregate::default());
        let ingress = crate::network::GossipIngress {
            action: crate::network::GossipAction::Accept,
            topic: "/leanconsensus/x/block/ssz_snappy".into(),
            peer: None,
            plain: Some(signed.ssz_encode().unwrap()),
        };
        (root, ingress)
    }

    #[test]
    fn imported_gossip_blocks_keeps_only_known_roots() {
        let (known_root, known) = block_gossip(5);
        let (_, unknown) = block_gossip(6);
        let vote = crate::network::GossipIngress {
            action: crate::network::GossipAction::Accept,
            topic: "/leanconsensus/x/attestation_0/ssz_snappy".into(),
            peer: None,
            plain: Some(vec![1, 2, 3]),
        };
        let mut owner = crate::chain_owner::ChainOwner::new(0);
        owner.head_root = known_root;
        let out = imported_gossip_blocks(&owner, &[known.clone(), unknown, vote]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].0, 5);
        assert_eq!(out[0].1, known_root);
        assert_eq!(out[0].2, known.plain.unwrap());
    }
}
