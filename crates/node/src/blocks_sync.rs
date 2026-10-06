//! Import SignedBlock blobs from blocks-by-root responses (with parent catch-up).

use crate::chain_owner::ChainOwner;
use crate::events::ChainEvent;
use crate::gossip_decode::try_decode_block;
use crate::gossip_stf::{import_decoded_block, GossipStfResult};
use crate::network::{decode_blocks_by_root_response, SwarmFacade};
use crate::shutdown::ShutdownState;
use crate::sync_orphan::SyncOrphan;
use ethean_primitives::Hash32;
use tracing::info;

/// Synthetic topic so [`crate::gossip_decode::try_decode_block`] accepts sync payloads.
pub const SYNC_BLOCK_TOPIC: &str = "/leanconsensus/sync/block/ssz_snappy";

/// Result of ingesting one blocks-by-root response.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BlocksSyncOutcome {
    /// Observer events (gossip ingested / head moved via import).
    pub events: Vec<ChainEvent>,
    /// Missing parent roots to request next from the same peer.
    pub fetch_roots: Vec<Hash32>,
}

/// Decode a blocks-by-root response and ingest each SignedBlock / Block blob.
///
/// Orphans (parent ≠ head) are buffered on the chain owner; callers should request
/// [`BlocksSyncOutcome::fetch_roots`] via blocks-by-root.
pub fn ingest_blocks_by_root_response(
    owner: &mut ChainOwner,
    shutdown: &mut ShutdownState,
    mut swarm: Option<&mut SwarmFacade>,
    peer: Hash32,
    payload: &[u8],
) -> BlocksSyncOutcome {
    let mut out = BlocksSyncOutcome::default();
    let blobs = match decode_blocks_by_root_response(payload) {
        Ok(b) => b,
        Err(e) => {
            info!(
                peer0 = peer[0],
                error = %e,
                "blocks-by-root response decode failed"
            );
            return out;
        }
    };
    if blobs.is_empty() {
        info!(peer0 = peer[0], "blocks-by-root response empty");
        return out;
    }

    for blob in blobs {
        let Some(decoded) = try_decode_block(SYNC_BLOCK_TOPIC, &blob) else {
            info!(
                peer0 = peer[0],
                bytes = blob.len(),
                "blocks-by-root blob not SignedBlock/Block SSZ"
            );
            continue;
        };
        if let Some(facade) = swarm.as_deref_mut() {
            #[cfg(feature = "libp2p-quic")]
            {
                let _ = facade.put_block_bytes(decoded.root, blob.clone());
                let _ =
                    facade.put_block_at_slot(decoded.block.slot.get(), decoded.root, blob.clone());
            }
            #[cfg(not(feature = "libp2p-quic"))]
            {
                let _ = facade;
            }
        }
        owner.last_gossip_root = Some(decoded.root);
        out.events.push(ChainEvent::GossipIngested {
            topic: SYNC_BLOCK_TOPIC.to_string(),
            content_root: decoded.root,
        });

        let stf = import_decoded_block(owner, shutdown, &decoded);
        match stf {
            GossipStfResult::Skipped
                if !owner.can_import_parent(decoded.parent)
                    && decoded.block.slot.get() > owner.finalized_slot() =>
            {
                info!(
                    peer0 = peer[0],
                    root0 = decoded.root[0],
                    parent0 = decoded.parent[0],
                    "sync block orphaned; will fetch parent"
                );
                owner.sync_orphans.insert(
                    decoded.root,
                    SyncOrphan {
                        parent: decoded.parent,
                        blob,
                    },
                );
            }
            GossipStfResult::Applied { .. } => {
                owner.remember_durable_block(decoded.root, blob.clone());
                info!(
                    peer0 = peer[0],
                    root0 = decoded.root[0],
                    ?stf,
                    "sync block imported"
                );
                drain_orphans(owner, shutdown, &mut out.events);
            }
            GossipStfResult::Skipped | GossipStfResult::Rejected { .. } => {
                info!(
                    peer0 = peer[0],
                    root0 = decoded.root[0],
                    ?stf,
                    "sync block not applied"
                );
            }
        }
    }

    out.fetch_roots = owner.sync_orphans.missing_parents();
    // Do not re-request roots we already have as head.
    out.fetch_roots.retain(|r| *r != owner.head_root);
    out
}

/// Buffer gossip blocks whose parent is unknown and return, per source peer,
/// the parent roots to request by root. Without this a node that joined late
/// (checkpoint anchor) or missed a block never links the live chain unless a
/// Status handshake happens to start range catch-up.
pub fn buffer_gossip_orphans(
    owner: &mut ChainOwner,
    accepted: &[crate::network::GossipIngress],
) -> Vec<(Hash32, Vec<Hash32>)> {
    let mut requests: Vec<(Hash32, Vec<Hash32>)> = Vec::new();
    for g in accepted {
        let (Some(peer), Some(plain)) = (g.peer, g.plain.as_deref()) else {
            continue;
        };
        let Some(decoded) = try_decode_block(&g.topic, plain) else {
            continue;
        };
        let known = |root: &Hash32| {
            owner.head_root == *root
                || owner
                    .fc
                    .as_ref()
                    .is_some_and(|fc| fc.blocks.contains_key(root))
        };
        if known(&decoded.root)
            || owner.can_import_parent(decoded.parent)
            || decoded.block.slot.get() <= owner.finalized_slot()
        {
            continue;
        }
        owner.sync_orphans.insert(
            decoded.root,
            SyncOrphan {
                parent: decoded.parent,
                blob: plain.to_vec(),
            },
        );
        match requests.iter_mut().find(|(p, _)| *p == peer) {
            Some((_, roots)) if !roots.contains(&decoded.parent) => roots.push(decoded.parent),
            Some(_) => {}
            None => requests.push((peer, vec![decoded.parent])),
        }
    }
    requests
}

fn drain_orphans(owner: &mut ChainOwner, shutdown: &ShutdownState, events: &mut Vec<ChainEvent>) {
    loop {
        let parents: Vec<Hash32> = {
            let mut p = vec![owner.head_root];
            if let Some(fc) = owner.fc.as_ref() {
                p.extend(fc.blocks.keys().copied());
            }
            p.sort();
            p.dedup();
            p
        };
        let mut progressed = false;
        for parent in parents {
            let Some((root, orphan)) = owner.sync_orphans.take_child_of(parent) else {
                continue;
            };
            let Some(decoded) = try_decode_block(SYNC_BLOCK_TOPIC, &orphan.blob) else {
                progressed = true;
                break;
            };
            let stf = import_decoded_block(owner, shutdown, &decoded);
            match stf {
                GossipStfResult::Applied { .. } => {
                    owner.remember_durable_block(root, orphan.blob.clone());
                    events.push(ChainEvent::GossipIngested {
                        topic: SYNC_BLOCK_TOPIC.to_string(),
                        content_root: root,
                    });
                    progressed = true;
                }
                _ => {
                    // Parent matched but import failed — drop (do not infinite-loop).
                }
            }
            break;
        }
        if !progressed {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::encode_blocks_by_root_response;
    use ethean_primitives::{Slot, ValidatorIndex};
    use ethean_types::{Block, BlockBody, MultiMessageAggregate, SignedBlock};

    fn signed_child(parent: [u8; 32], slot: u64, tag: u8) -> ([u8; 32], Vec<u8>) {
        let block = Block {
            slot: Slot::new(slot),
            proposer_index: ValidatorIndex::new(0),
            parent_root: parent,
            state_root: [tag; 32],
            body: BlockBody::default(),
        };
        let root = block.hash_tree_root().unwrap();
        let signed = SignedBlock::new(block, MultiMessageAggregate::default());
        (root, signed.ssz_encode().unwrap())
    }

    #[test]
    fn never_advances_head_without_a_verified_block() {
        let (_root, enc) = signed_child([0u8; 32], 1, 2);
        let payload = encode_blocks_by_root_response(&[enc]).unwrap();
        let mut owner = ChainOwner::new(2);
        owner.head_root = [0u8; 32];
        let mut shutdown = ShutdownState::default();
        let out =
            ingest_blocks_by_root_response(&mut owner, &mut shutdown, None, [9u8; 32], &payload);
        assert_eq!(out.events.len(), 1, "ingest is observed");
        assert_eq!(
            owner.head_root, [0u8; 32],
            "no state and no proof: head unchanged"
        );
    }

    #[test]
    fn orphaned_tip_requests_its_parent() {
        let genesis = [0u8; 32];
        let (mid_root, _mid_enc) = signed_child(genesis, 1, 3);
        let (_tip_root, tip_enc) = signed_child(mid_root, 2, 4);
        let mut owner = ChainOwner::new(2);
        owner.head_root = genesis;
        let mut shutdown = ShutdownState::default();
        let tip_payload = encode_blocks_by_root_response(&[tip_enc]).unwrap();
        let out = ingest_blocks_by_root_response(
            &mut owner,
            &mut shutdown,
            None,
            [9u8; 32],
            &tip_payload,
        );
        assert_eq!(owner.head_root, genesis);
        assert_eq!(out.fetch_roots, vec![mid_root]);
        assert_eq!(owner.sync_orphans.len(), 1);
    }

    fn block_gossip(peer: Option<[u8; 32]>, plain: Vec<u8>) -> crate::network::GossipIngress {
        crate::network::GossipIngress {
            action: crate::network::GossipAction::Accept,
            topic: "/leanconsensus/x/block/ssz_snappy".into(),
            peer,
            plain: Some(plain),
        }
    }

    #[test]
    fn gossip_orphan_asks_its_sender_for_the_parent() {
        let genesis = [0u8; 32];
        let (mid_root, _) = signed_child(genesis, 1, 3);
        let (_, tip_enc) = signed_child(mid_root, 2, 4);
        let (_, tip2_enc) = signed_child(mid_root, 2, 5);
        let (_, on_head_enc) = signed_child(genesis, 1, 6);
        let mut owner = ChainOwner::new(2);
        owner.head_root = genesis;
        let (a, b) = ([7u8; 32], [8u8; 32]);
        let accepted = vec![
            block_gossip(Some(a), tip_enc.clone()),
            block_gossip(Some(a), tip2_enc),
            block_gossip(Some(b), tip_enc),
            block_gossip(Some(b), on_head_enc),
            block_gossip(None, signed_child(mid_root, 3, 9).1),
        ];
        let requests = buffer_gossip_orphans(&mut owner, &accepted);
        assert_eq!(requests, vec![(a, vec![mid_root]), (b, vec![mid_root])]);
        assert_eq!(owner.sync_orphans.len(), 2);
        assert_eq!(owner.sync_orphans.missing_parents(), vec![mid_root]);
    }
}
