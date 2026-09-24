//! QUIC listen helpers and bind smoke tests for QuicSwarm.

#![cfg(feature = "libp2p-quic")]

use crate::error::NetworkError;
use crate::gossip::LeanGossipTopics;
use crate::quic_swarm::LeanBehaviour;
use ethean_network_wire::gossip::{
    FANOUT_TTL, GOSSIP_LAZY, HEARTBEAT, HISTORY_GOSSIP, HISTORY_LENGTH, MAX_MESSAGES_PER_RPC,
    MESH_N, MESH_N_HIGH, MESH_N_LOW, SEEN_TTL,
};
use ethean_network_wire::{
    compute_message_id, decompress_raw, max_message_size, MESSAGE_DOMAIN_INVALID_SNAPPY,
    MESSAGE_DOMAIN_VALID_SNAPPY,
};
use libp2p::futures::StreamExt;
use libp2p::gossipsub::{self, IdentTopic, MessageAuthenticity, MessageId, ValidationMode};
use libp2p::identity;
use libp2p::swarm::SwarmEvent;
use libp2p::Multiaddr;

type NetResult<T> = std::result::Result<T, NetworkError>;

pub(crate) fn build_gossipsub(_keypair: &identity::Keypair) -> NetResult<gossipsub::Behaviour> {
    let config = gossipsub::ConfigBuilder::default()
        .max_transmit_size(max_message_size())
        .heartbeat_interval(HEARTBEAT)
        .fanout_ttl(FANOUT_TTL)
        .mesh_n(MESH_N)
        .mesh_n_low(MESH_N_LOW)
        .mesh_n_high(MESH_N_HIGH)
        .gossip_lazy(GOSSIP_LAZY)
        .history_length(HISTORY_LENGTH)
        .history_gossip(HISTORY_GOSSIP)
        .max_messages_per_rpc(Some(MAX_MESSAGES_PER_RPC))
        .duplicate_cache_time(SEEN_TTL)
        .validation_mode(ValidationMode::Anonymous)
        .allow_self_origin(true)
        .flood_publish(false)
        .message_id_fn(|message| {
            let topic = message.topic.as_str().as_bytes();
            let (data, domain) = match decompress_raw(&message.data) {
                Ok(plain) => (plain, MESSAGE_DOMAIN_VALID_SNAPPY),
                Err(_) => (message.data.clone(), MESSAGE_DOMAIN_INVALID_SNAPPY),
            };
            MessageId::from(&compute_message_id(topic, &data, domain)[..])
        })
        .build()
        .map_err(|e| NetworkError::Handshake(format!("gossipsub config: {e}")))?;
    gossipsub::Behaviour::new(MessageAuthenticity::Anonymous, config)
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
    use crate::quic_swarm::QuicSwarm;
    use crate::transport::TransportConfig;

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
        assert!(topics.block.contains("12345678"));
        assert!(swarm
            .publish_gossip("/eth2/beacon_block/ssz_snappy", b"x")
            .is_err());
    }
}
