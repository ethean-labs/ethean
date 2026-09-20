//! Apply [`ChainCommand`] to the chain owner and shutdown state.

use crate::chain_owner::ChainOwner;
use crate::commands::ChainCommand;
use crate::events::ChainEvent;
use crate::gossip_decode::{content_root_for, try_decode_block};
use crate::gossip_pool::ingest_into_pool;
use crate::gossip_stf::import_decoded_block;
use crate::shutdown::ShutdownState;

/// Dispatch one command; returns the observer event.
pub fn apply_command(
    owner: &mut ChainOwner,
    shutdown: &mut ShutdownState,
    cmd: ChainCommand,
) -> ChainEvent {
    match cmd {
        ChainCommand::Tick(tick) => {
            if !shutdown.accepts_new_duties() {
                return ChainEvent::ShutdownComplete;
            }
            if owner.on_tick(tick) {
                ChainEvent::TickAccepted(tick)
            } else {
                ChainEvent::TickDuplicate(tick)
            }
        }
        ChainCommand::ImportBlock { root, parent } => import_block(owner, shutdown, root, parent),
        ChainCommand::IngestGossip {
            topic,
            payload,
            peer: _,
        } => ingest_gossip(owner, shutdown, topic, payload),
        ChainCommand::SetSyncing(syncing) => {
            owner.syncing = syncing;
            ChainEvent::SyncingUpdated(syncing)
        }
        ChainCommand::Shutdown => {
            shutdown.begin_drain();
            shutdown.finish();
            ChainEvent::ShutdownComplete
        }
    }
}

fn import_block(
    owner: &mut ChainOwner,
    shutdown: &ShutdownState,
    root: ethean_primitives::Hash32,
    parent: ethean_primitives::Hash32,
) -> ChainEvent {
    if shutdown.phase() == crate::shutdown::ShutdownPhase::Stopped {
        return ChainEvent::ShutdownComplete;
    }
    if parent != owner.head_root {
        return ChainEvent::HeadUpdated {
            root: owner.head_root,
            slot: owner.last_tick.map(|t| t.slot.get()).unwrap_or(0),
        };
    }
    owner.head_root = root;
    ChainEvent::HeadUpdated {
        root,
        slot: owner.last_tick.map(|t| t.slot.get()).unwrap_or(0),
    }
}

fn ingest_gossip(
    owner: &mut ChainOwner,
    shutdown: &ShutdownState,
    topic: String,
    payload: Vec<u8>,
) -> ChainEvent {
    if shutdown.phase() == crate::shutdown::ShutdownPhase::Stopped {
        return ChainEvent::ShutdownComplete;
    }
    let content_root = content_root_for(&topic, &payload);
    owner.last_gossip_root = Some(content_root);

    let _ = ingest_into_pool(&mut owner.aggregates, &topic, &payload);

    if let Some(decoded) = try_decode_block(&topic, &payload) {
        match import_decoded_block(owner, shutdown, &decoded) {
            crate::gossip_stf::GossipStfResult::Applied { .. }
            | crate::gossip_stf::GossipStfResult::AppliedVerified { .. }
            | crate::gossip_stf::GossipStfResult::RootOnly { .. } => {
                owner.remember_durable_block(decoded.root, payload.clone());
            }
            _ => {}
        }
    }

    ChainEvent::GossipIngested {
        topic,
        content_root,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::Slot;
    use ethean_validator::DutyTick;

    #[test]
    fn tick_then_shutdown() {
        let mut owner = ChainOwner::new(2);
        let mut shutdown = ShutdownState::default();
        let tick = DutyTick {
            slot: Slot::new(1),
            interval: 0,
            generation: 1,
        };
        assert!(matches!(
            apply_command(&mut owner, &mut shutdown, ChainCommand::Tick(tick)),
            ChainEvent::TickAccepted(_)
        ));
        assert_eq!(
            apply_command(&mut owner, &mut shutdown, ChainCommand::Shutdown),
            ChainEvent::ShutdownComplete
        );
        assert!(!shutdown.accepts_new_duties());
    }

    #[test]
    fn import_requires_parent() {
        let mut owner = ChainOwner::new(2);
        owner.head_root = [1u8; 32];
        let mut shutdown = ShutdownState::default();
        let ev = apply_command(
            &mut owner,
            &mut shutdown,
            ChainCommand::ImportBlock {
                root: [2u8; 32],
                parent: [9u8; 32],
            },
        );
        assert_eq!(owner.head_root, [1u8; 32]);
        assert!(matches!(ev, ChainEvent::HeadUpdated { root, .. } if root == [1u8; 32]));
        let ev = apply_command(
            &mut owner,
            &mut shutdown,
            ChainCommand::ImportBlock {
                root: [2u8; 32],
                parent: [1u8; 32],
            },
        );
        assert_eq!(owner.head_root, [2u8; 32]);
        assert!(matches!(ev, ChainEvent::HeadUpdated { root, .. } if root == [2u8; 32]));
    }

    #[test]
    fn ingest_gossip_records_content_root() {
        let mut owner = ChainOwner::new(2);
        let mut shutdown = ShutdownState::default();
        let ev = apply_command(
            &mut owner,
            &mut shutdown,
            ChainCommand::IngestGossip {
                topic: "/leanconsensus/x/aggregation/ssz_snappy".into(),
                payload: b"block-bytes".to_vec(),
                peer: None,
            },
        );
        let root = owner.last_gossip_root.expect("root");
        assert!(matches!(
            ev,
            ChainEvent::GossipIngested { content_root, .. } if content_root == root
        ));
        assert_eq!(owner.head_root, [0u8; 32]);
        assert_eq!(owner.aggregates.len(), 1);
    }

    #[test]
    fn ingest_block_advances_head_when_parent_matches() {
        use ethean_primitives::{Slot, ValidatorIndex};
        use ethean_types::{Block, BlockBody};

        let mut owner = ChainOwner::new(2);
        owner.head_root = [1u8; 32];
        let mut shutdown = ShutdownState::default();
        let block = Block {
            slot: Slot::new(4),
            proposer_index: ValidatorIndex::new(0),
            parent_root: [1u8; 32],
            state_root: [7u8; 32],
            body: BlockBody::default(),
        };
        let enc = block.ssz_encode().unwrap();
        let root = block.hash_tree_root().unwrap();
        let ev = apply_command(
            &mut owner,
            &mut shutdown,
            ChainCommand::IngestGossip {
                topic: "/leanconsensus/abcd/block/ssz_snappy".into(),
                payload: enc,
                peer: None,
            },
        );
        assert_eq!(owner.head_root, root);
        assert_eq!(owner.last_gossip_root, Some(root));
        assert!(matches!(
            ev,
            ChainEvent::GossipIngested { content_root, .. } if content_root == root
        ));
    }

    #[test]
    fn ingest_attestation_fills_aggregate_pool() {
        use ethean_types::{AggregatedAttestation, AggregationBits, AttestationData, Checkpoint};

        let mut owner = ChainOwner::new(2);
        let mut shutdown = ShutdownState::default();
        let agg = AggregatedAttestation {
            aggregation_bits: AggregationBits {
                bits: vec![true, true, false],
            },
            data: AttestationData {
                slot: Slot::new(3),
                head: Checkpoint::genesis(),
                target: Checkpoint::genesis(),
                source: Checkpoint::genesis(),
            },
        };
        let enc = agg.ssz_encode();
        let _ = apply_command(
            &mut owner,
            &mut shutdown,
            ChainCommand::IngestGossip {
                topic: "/leanconsensus/abcd/attestation_0/ssz_snappy".into(),
                payload: enc,
                peer: None,
            },
        );
        assert_eq!(owner.aggregates.len(), 1);
        assert_eq!(owner.head_root, [0u8; 32]);
    }
}
