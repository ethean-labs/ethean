//! Turn accepted gossip ingress into chain-owner ingest commands.

use crate::chain_owner::ChainOwner;
use crate::dispatch::ingest_gossip;
use crate::events::ChainEvent;
use crate::gossip_attestation::preverify_votes;
use crate::network::GossipIngress;
use crate::shutdown::ShutdownState;

/// Ingest each ACCEPT payload in arrival order. Vote signatures of the whole
/// window are verified up front in parallel; admission (pool, fork choice)
/// still happens one payload at a time, so a vote that follows its head
/// block in the window still sees that block.
pub fn ingest_accepted(
    owner: &mut ChainOwner,
    shutdown: &mut ShutdownState,
    accepted: &[GossipIngress],
) -> Vec<ChainEvent> {
    let plain: Vec<(&str, &[u8])> = accepted
        .iter()
        .filter_map(|g| Some((g.topic.as_str(), g.plain.as_deref()?)))
        .collect();
    let verdicts = preverify_votes(owner, &plain);
    plain
        .into_iter()
        .zip(verdicts)
        .map(|((topic, payload), vote)| {
            ingest_gossip(owner, shutdown, topic.to_string(), payload.to_vec(), vote)
        })
        .collect()
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

    #[test]
    fn a_window_of_forged_votes_never_reaches_the_pool() {
        use ethean_genesis::GenesisBuilder;
        use ethean_primitives::{Bytes52, Slot, ValidatorIndex};
        use ethean_types::{AttestationData, Checkpoint, SignedAttestation};

        let genesis = GenesisBuilder::new(1_700_000_000)
            .with_validator_keys(vec![(Bytes52::ZERO, Bytes52::ZERO); 4])
            .build()
            .unwrap();
        let mut owner = ChainOwner::new(2);
        owner.head_state = Some(genesis.state);
        owner.is_aggregator = true;
        let mut shutdown = ShutdownState::default();
        let data = AttestationData {
            slot: Slot::new(0),
            head: Checkpoint::genesis(),
            target: Checkpoint::genesis(),
            source: Checkpoint::genesis(),
        };
        let accepted: Vec<GossipIngress> = (0..4u64)
            .map(|i| GossipIngress {
                action: GossipAction::Accept,
                topic: "/leanconsensus/x/attestation_0/ssz_snappy".into(),
                peer: None,
                plain: Some(
                    SignedAttestation::new(
                        ValidatorIndex::new(i),
                        data,
                        vec![0u8; ethean_crypto::SIGNATURE_BYTES],
                    )
                    .unwrap()
                    .ssz_encode(),
                ),
            })
            .collect();
        let ev = ingest_accepted(&mut owner, &mut shutdown, &accepted);
        assert_eq!(ev.len(), 4);
        assert!(owner.signatures.is_empty());
    }
}
