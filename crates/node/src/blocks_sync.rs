//! Import SignedBlock blobs from blocks-by-root responses.

use crate::chain_owner::ChainOwner;
use crate::commands::ChainCommand;
use crate::dispatch::apply_command;
use crate::events::ChainEvent;
use crate::gossip_decode::try_decode_block;
use crate::network::{decode_blocks_by_root_response, SwarmFacade};
use crate::shutdown::ShutdownState;
use ethean_primitives::Hash32;
use tracing::info;

/// Synthetic topic so [`crate::gossip_decode::try_decode_block`] accepts sync payloads.
pub const SYNC_BLOCK_TOPIC: &str = "/leanconsensus/sync/block/ssz_snappy";

/// Decode a blocks-by-root response and ingest each SignedBlock / Block blob.
///
/// Also caches successfully decoded SSZ into the QuicSwarm serve map when present.
pub fn ingest_blocks_by_root_response(
    owner: &mut ChainOwner,
    shutdown: &mut ShutdownState,
    mut swarm: Option<&mut SwarmFacade>,
    peer: Hash32,
    payload: &[u8],
) -> Vec<ChainEvent> {
    let blobs = match decode_blocks_by_root_response(payload) {
        Ok(b) => b,
        Err(e) => {
            info!(
                peer0 = peer[0],
                error = %e,
                "blocks-by-root response decode failed"
            );
            return Vec::new();
        }
    };
    if blobs.is_empty() {
        info!(peer0 = peer[0], "blocks-by-root response empty");
        return Vec::new();
    }

    let mut events = Vec::with_capacity(blobs.len());
    for blob in blobs {
        if let Some(decoded) = try_decode_block(SYNC_BLOCK_TOPIC, &blob) {
            if let Some(facade) = swarm.as_deref_mut() {
                let _ = facade.put_block_bytes(decoded.root, blob.clone());
            }
            info!(
                peer0 = peer[0],
                root0 = decoded.root[0],
                bytes = blob.len(),
                "blocks-by-root blob decoded; ingesting"
            );
        } else {
            info!(
                peer0 = peer[0],
                bytes = blob.len(),
                "blocks-by-root blob not SignedBlock/Block SSZ"
            );
            continue;
        }
        events.push(apply_command(
            owner,
            shutdown,
            ChainCommand::IngestGossip {
                topic: SYNC_BLOCK_TOPIC.to_string(),
                payload: blob,
                peer: Some(peer),
            },
        ));
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::encode_blocks_by_root_response;
    use ethean_primitives::{Slot, ValidatorIndex};
    use ethean_types::{Block, BlockBody, MultiMessageAggregate, SignedBlock};

    #[test]
    fn ingests_signed_block_from_response() {
        let block = Block {
            slot: Slot::new(1),
            proposer_index: ValidatorIndex::new(0),
            parent_root: [0u8; 32],
            state_root: [2u8; 32],
            body: BlockBody::default(),
        };
        let signed = SignedBlock::new(block.clone(), MultiMessageAggregate::default());
        let enc = signed.ssz_encode().unwrap();
        let payload = encode_blocks_by_root_response(&[enc]).unwrap();
        let mut owner = ChainOwner::new(2);
        owner.head_root = [0u8; 32];
        let mut shutdown = ShutdownState::default();
        let events =
            ingest_blocks_by_root_response(&mut owner, &mut shutdown, None, [9u8; 32], &payload);
        assert_eq!(events.len(), 1);
        assert_eq!(owner.head_root, block.hash_tree_root().unwrap());
    }
}
