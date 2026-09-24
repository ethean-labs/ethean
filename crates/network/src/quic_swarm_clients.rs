//! Per-client peer accounting for leanMetrics (`client` label).

use std::collections::HashMap;

use libp2p::gossipsub::TopicHash;
use libp2p::PeerId;

use crate::quic_swarm::QuicSwarm;

/// leanMetrics client family of an identify agent version (`ream/0.4.1` -> `ream`).
pub fn client_family(agent_version: &str) -> String {
    let head = agent_version
        .split(|c: char| c == '/' || c == ' ' || c == '-')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if head.is_empty() {
        "unknown".into()
    } else {
        head
    }
}

impl QuicSwarm {
    fn family_of(&self, peer: &PeerId) -> String {
        self.agents
            .get(peer)
            .map(|a| client_family(a))
            .unwrap_or_else(|| "unknown".into())
    }

    /// Connected peers grouped by client family.
    pub fn peer_clients(&self) -> Vec<(String, u64)> {
        let mut counts: HashMap<String, u64> = HashMap::new();
        for peer in self.swarm.connected_peers() {
            *counts.entry(self.family_of(peer)).or_insert(0) += 1;
        }
        let mut out: Vec<_> = counts.into_iter().collect();
        out.sort();
        out
    }

    /// Peers in any subscribed gossipsub mesh, grouped by client family.
    pub fn mesh_peer_clients(&self) -> Vec<(String, u64)> {
        let Some(topics) = self.topics.as_ref() else {
            return Vec::new();
        };
        let mut seen: HashMap<PeerId, ()> = HashMap::new();
        let names = std::iter::once(&topics.block)
            .chain(std::iter::once(&topics.aggregation))
            .chain(topics.attestations.iter());
        for name in names {
            let hash = TopicHash::from_raw(name.clone());
            for peer in self.swarm.behaviour().gossipsub.mesh_peers(&hash) {
                seen.insert(*peer, ());
            }
        }
        let mut counts: HashMap<String, u64> = HashMap::new();
        for peer in seen.keys() {
            *counts.entry(self.family_of(peer)).or_insert(0) += 1;
        }
        let mut out: Vec<_> = counts.into_iter().collect();
        out.sort();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::client_family;

    #[test]
    fn families_from_agent_strings() {
        assert_eq!(client_family("ream/0.4.1"), "ream");
        assert_eq!(client_family("Zeam v0.5.12"), "zeam");
        assert_eq!(client_family("ethlambda-0.3"), "ethlambda");
        assert_eq!(client_family(""), "unknown");
    }
}
