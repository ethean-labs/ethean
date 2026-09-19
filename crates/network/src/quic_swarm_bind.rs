//! QUIC listen helpers and bind smoke tests for QuicSwarm.

#![cfg(feature = "libp2p-quic")]

use crate::error::NetworkError;
use crate::gossip::LeanGossipTopics;
use crate::quic_swarm::{LeanBehaviour, QuicSwarm};
use crate::transport::TransportConfig;
use libp2p::futures::StreamExt;
use libp2p::gossipsub::{self, IdentTopic, MessageAuthenticity, ValidationMode};
use libp2p::identity;
use libp2p::swarm::SwarmEvent;
use libp2p::Multiaddr;

type NetResult<T> = std::result::Result<T, NetworkError>;

pub(crate) fn build_gossipsub(keypair: &identity::Keypair) -> NetResult<gossipsub::Behaviour> {
    let config = gossipsub::ConfigBuilder::default()
        .validation_mode(ValidationMode::Permissive)
        .build()
        .map_err(|e| NetworkError::Handshake(format!("gossipsub config: {e}")))?;
    gossipsub::Behaviour::new(MessageAuthenticity::Signed(keypair.clone()), config)
        .map_err(|e| NetworkError::Handshake(format!("gossipsub behaviour: {e}")))
}

pub(crate) fn subscribe_all(
    swarm: &mut libp2p::Swarm<LeanBehaviour>,
    topics: &LeanGossipTopics,
) -> NetResult<()> {
    for topic in topics.as_slice() {
        LeanGossipTopics::reject_if_eth2(topic)?;
        let t = IdentTopic::new(topic);
        swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&t)
            .map_err(|e| NetworkError::Handshake(format!("subscribe {topic}: {e}")))?;
    }
    Ok(())
}

pub(crate) async fn wait_quic_listen(
    swarm: &mut libp2p::Swarm<LeanBehaviour>,
) -> NetResult<Multiaddr> {
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                if address.to_string().contains("quic") {
                    return Ok(address);
                }
            }
            SwarmEvent::ListenerError { error, .. } => {
                return Err(NetworkError::Handshake(format!("listener: {error}")));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn binds_ephemeral_quic() {
        let mut swarm = QuicSwarm::bind(&TransportConfig {
            listen_port: 0,
            idle_timeout_ms: 1_000,
        })
        .await
        .expect("quic bind");
        assert!(swarm.listen_addr.to_string().contains("quic"));
        assert!(swarm.dial("/ip4/127.0.0.1/tcp/1").is_err());
    }

    #[tokio::test]
    async fn binds_and_subscribes_lean_topics() {
        let mut swarm = QuicSwarm::bind_for_fork(
            &TransportConfig {
                listen_port: 0,
                idle_timeout_ms: 1_000,
            },
            "lstar",
        )
        .await
        .expect("quic+gossip");
        let topics = swarm.topics.as_ref().expect("topics");
        assert!(topics.block.contains("/leanconsensus/"));
        assert!(swarm
            .publish_gossip("/eth2/beacon_block/ssz_snappy", b"x")
            .is_err());
    }
}
