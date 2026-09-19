//! QuicSwarm gossip and Status event handlers (feature `libp2p-quic`).

#![cfg(feature = "libp2p-quic")]

use crate::gossip::{GossipAction, GossipIngress, PumpEvent};
use crate::quic_swarm::QuicSwarm;
use ethean_primitives::Hash32;
use libp2p::gossipsub;
use libp2p::identity::PeerId;
use libp2p::request_response::{self, ResponseChannel};
use sha2::{Digest, Sha256};

impl QuicSwarm {
    pub(crate) fn handle_status_event(
        &mut self,
        ev: request_response::Event<Vec<u8>, Vec<u8>>,
    ) -> PumpEvent {
        match ev {
            request_response::Event::Message { peer, message } => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    self.reply_status(channel);
                    PumpEvent::StatusRequest {
                        peer: peer_fingerprint(&peer),
                        payload: request,
                    }
                }
                request_response::Message::Response { response, .. } => PumpEvent::StatusResponse {
                    peer: peer_fingerprint(&peer),
                    payload: response,
                },
            },
            _ => PumpEvent::Behaviour,
        }
    }

    pub(crate) fn reply_status(&mut self, channel: ResponseChannel<Vec<u8>>) {
        if let Some(bytes) = self.local_status.clone() {
            let _ = self
                .swarm
                .behaviour_mut()
                .status
                .send_response(channel, bytes);
        }
    }

    pub(crate) fn handle_gossip_event(&mut self, ev: gossipsub::Event) -> PumpEvent {
        match ev {
            gossipsub::Event::Message {
                propagation_source,
                message,
                ..
            } => {
                let topic = message.topic.to_string();
                let peer = Some(peer_fingerprint(&propagation_source));
                let (action, plain) = crate::gossip::validate_gossip_payload(
                    &topic,
                    &message.data,
                    &mut self.seen_ids,
                );
                let plain = match action {
                    GossipAction::Accept => plain,
                    _ => None,
                };
                PumpEvent::Gossip(GossipIngress {
                    action,
                    topic,
                    peer,
                    plain,
                })
            }
            gossipsub::Event::Subscribed { .. } => PumpEvent::GossipSubscribed,
            gossipsub::Event::Unsubscribed { .. } => PumpEvent::GossipUnsubscribed,
            _ => PumpEvent::Other,
        }
    }

    pub(crate) fn remember_peer(&mut self, peer_id: PeerId) -> Hash32 {
        let peer = peer_fingerprint(&peer_id);
        self.peers.insert(peer, peer_id);
        peer
    }

    pub(crate) fn forget_peer(&mut self, peer_id: &PeerId) -> Hash32 {
        let peer = peer_fingerprint(peer_id);
        self.peers.remove(&peer);
        peer
    }
}

pub(crate) fn peer_fingerprint(peer: &PeerId) -> Hash32 {
    let mut hasher = Sha256::new();
    hasher.update(peer.to_bytes());
    let dig = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&dig);
    out
}
