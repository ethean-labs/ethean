//! Turn accepted gossip ingress into chain-owner ingest commands.

use crate::chain_owner::ChainOwner;
use crate::commands::ChainCommand;
use crate::dispatch::apply_command;
use crate::events::ChainEvent;
use crate::network::GossipIngress;
use crate::shutdown::ShutdownState;

/// Dispatch [`ChainCommand::IngestGossip`] for each ACCEPT payload.
pub fn ingest_accepted(
    owner: &mut ChainOwner,
    shutdown: &mut ShutdownState,
    accepted: &[GossipIngress],
) -> Vec<ChainEvent> {
    let mut events = Vec::with_capacity(accepted.len());
    for g in accepted {
        let Some(payload) = g.plain.as_ref() else {
            continue;
        };
        events.push(apply_command(
            owner,
            shutdown,
            ChainCommand::IngestGossip {
                topic: g.topic.clone(),
                payload: payload.clone(),
                peer: g.peer,
            },
        ));
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::GossipAction;

    #[test]
    fn ingest_sets_last_gossip_root() {
        let mut owner = ChainOwner::new(2);
        let mut shutdown = ShutdownState::default();
        let accepted = vec![GossipIngress {
            action: GossipAction::Accept,
            topic: "/leanconsensus/x/block/ssz_snappy".into(),
            peer: Some([9u8; 32]),
            plain: Some(b"hello".to_vec()),
        }];
        let ev = ingest_accepted(&mut owner, &mut shutdown, &accepted);
        assert_eq!(ev.len(), 1);
        assert!(owner.last_gossip_root.is_some());
    }
}
