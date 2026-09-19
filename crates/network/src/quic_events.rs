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

    pub(crate) fn handle_blocks_by_root_event(
        &mut self,
        ev: request_response::Event<Vec<u8>, Vec<u8>>,
    ) -> PumpEvent {
        match ev {
            request_response::Event::Message { peer, message } => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    self.reply_blocks_by_root(&request, channel);
                    PumpEvent::BlocksByRootRequest {
                        peer: peer_fingerprint(&peer),
                        payload: request,
                    }
                }
                request_response::Message::Response { response, .. } => {
                    PumpEvent::BlocksByRootResponse {
                        peer: peer_fingerprint(&peer),
                        payload: response,
                    }
                }
            },
            _ => PumpEvent::Behaviour,
        }
    }

    pub(crate) fn reply_blocks_by_root(
        &mut self,
        request: &[u8],
        channel: ResponseChannel<Vec<u8>>,
    ) {
        let roots = decode_root_list(request);
        let mut out = Vec::new();
        let mut n: u32 = 0;
        for root in roots {
            if let Some(block) = self.blocks_by_root.get(&root) {
                out.extend_from_slice(block);
                n = n.saturating_add(1);
            }
        }
        let mut payload = Vec::with_capacity(4 + out.len());
        payload.extend_from_slice(&n.to_le_bytes());
        payload.extend_from_slice(&out);
        let _ = self
            .swarm
            .behaviour_mut()
            .blocks_by_root
            .send_response(channel, payload);
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

/// Decode the scaffold blocks-by-root request (u32 LE count + 32-byte roots).
fn decode_root_list(input: &[u8]) -> Vec<Hash32> {
    if input.len() < 4 {
        return Vec::new();
    }
    let n = u32::from_le_bytes(input[0..4].try_into().unwrap_or([0; 4])) as usize;
    let mut roots = Vec::with_capacity(n.min(128));
    let mut off = 4;
    for _ in 0..n {
        if off + 32 > input.len() {
            break;
        }
        let mut root = [0u8; 32];
        root.copy_from_slice(&input[off..off + 32]);
        roots.push(root);
        off += 32;
    }
    roots
}
