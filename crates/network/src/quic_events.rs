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
        let mut blocks = Vec::new();
        for root in roots {
            if let Some(block) = self.blocks_by_root.get(&root) {
                blocks.push(block.clone());
            }
        }
        let payload = crate::reqresp::encode_blocks_by_root_response(&blocks).unwrap_or_else(|_| {
            let mut empty = Vec::with_capacity(4);
            empty.extend_from_slice(&0u32.to_le_bytes());
            empty
        });
        let _ = self
            .swarm
            .behaviour_mut()
            .blocks_by_root
            .send_response(channel, payload);
    }

    pub(crate) fn handle_blocks_by_range_event(
        &mut self,
        ev: request_response::Event<Vec<u8>, Vec<u8>>,
    ) -> PumpEvent {
        match ev {
            request_response::Event::Message { peer, message } => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    self.reply_blocks_by_range(&request, channel);
                    PumpEvent::BlocksByRangeRequest {
                        peer: peer_fingerprint(&peer),
                        payload: request,
                    }
                }
                request_response::Message::Response { response, .. } => {
                    PumpEvent::BlocksByRangeResponse {
                        peer: peer_fingerprint(&peer),
                        payload: response,
                    }
                }
            },
            _ => PumpEvent::Behaviour,
        }
    }

    pub(crate) fn reply_blocks_by_range(
        &mut self,
        request: &[u8],
        channel: ResponseChannel<Vec<u8>>,
    ) {
        let blocks = match crate::reqresp::decode_blocks_by_range(request) {
            Ok(req) => {
                let collected = collect_slot_range(
                    &self.blocks_by_slot,
                    req.start_slot,
                    req.count,
                    req.step,
                );
                if collected.missing > 0 || (req.count > 0 && collected.found == 0) {
                    tracing::warn!(
                        start_slot = req.start_slot,
                        count = req.count,
                        step = req.step.max(1),
                        found = collected.found,
                        missing = collected.missing,
                        cache_slots = self.blocks_by_slot.len(),
                        "blocks-by-range serve incomplete (cold cache or gaps)"
                    );
                } else {
                    tracing::debug!(
                        start_slot = req.start_slot,
                        found = collected.found,
                        "blocks-by-range serve ok"
                    );
                }
                collected.blocks
            }
            Err(e) => {
                tracing::warn!(error = %e, "blocks-by-range request decode failed");
                Vec::new()
            }
        };
        let payload = crate::reqresp::encode_blocks_by_root_response(&blocks).unwrap_or_else(|_| {
            let mut empty = Vec::with_capacity(4);
            empty.extend_from_slice(&0u32.to_le_bytes());
            empty
        });
        let _ = self
            .swarm
            .behaviour_mut()
            .blocks_by_range
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

/// Result of walking a slot range against the in-memory serve cache.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RangeCollect {
    blocks: Vec<Vec<u8>>,
    found: u64,
    missing: u64,
}

/// Collect cached block bodies for start..start+count*step (missing slots counted).
fn collect_slot_range(
    by_slot: &std::collections::HashMap<u64, Vec<u8>>,
    start: u64,
    count: u64,
    step: u64,
) -> RangeCollect {
    let step = step.max(1);
    let mut blocks = Vec::new();
    let mut found = 0u64;
    let mut missing = 0u64;
    let mut slot = start;
    for _ in 0..count {
        if let Some(bytes) = by_slot.get(&slot) {
            blocks.push(bytes.clone());
            found = found.saturating_add(1);
        } else {
            missing = missing.saturating_add(1);
        }
        slot = slot.saturating_add(step);
    }
    RangeCollect {
        blocks,
        found,
        missing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn collect_counts_gaps() {
        let mut map = HashMap::new();
        map.insert(10, vec![1]);
        map.insert(12, vec![2]);
        let c = collect_slot_range(&map, 10, 3, 1);
        assert_eq!(c.found, 2);
        assert_eq!(c.missing, 1);
        assert_eq!(c.blocks.len(), 2);
    }

    #[test]
    fn collect_respects_step() {
        let mut map = HashMap::new();
        map.insert(0, vec![9]);
        map.insert(4, vec![8]);
        let c = collect_slot_range(&map, 0, 2, 4);
        assert_eq!(c.found, 2);
        assert_eq!(c.missing, 0);
    }
}
