//! Outbound Lean req/resp sends on a bound [`QuicSwarm`].

#![cfg(feature = "libp2p-quic")]

use crate::error::NetworkError;
use crate::quic_swarm::QuicSwarm;
use ethean_primitives::Hash32;

type NetResult<T> = std::result::Result<T, NetworkError>;

impl QuicSwarm {
    /// Send a Status request to a connected peer fingerprint.
    pub fn send_status_request(&mut self, peer: Hash32, payload: Vec<u8>) -> NetResult<()> {
        let Some(peer_id) = self.peers.get(&peer).copied() else {
            return Err(NetworkError::Handshake(
                "peer not connected for Status request".into(),
            ));
        };
        let _ = self
            .swarm
            .behaviour_mut()
            .status
            .send_request(&peer_id, payload);
        Ok(())
    }

    /// Send a blocks-by-root request to a connected peer fingerprint.
    pub fn send_blocks_by_root_request(
        &mut self,
        peer: Hash32,
        payload: Vec<u8>,
    ) -> NetResult<()> {
        let Some(peer_id) = self.peers.get(&peer).copied() else {
            return Err(NetworkError::Handshake(
                "peer not connected for blocks-by-root request".into(),
            ));
        };
        let _ = self
            .swarm
            .behaviour_mut()
            .blocks_by_root
            .send_request(&peer_id, payload);
        Ok(())
    }

    /// Send a blocks-by-range request to a connected peer fingerprint.
    pub fn send_blocks_by_range_request(
        &mut self,
        peer: Hash32,
        payload: Vec<u8>,
    ) -> NetResult<()> {
        let Some(peer_id) = self.peers.get(&peer).copied() else {
            return Err(NetworkError::Handshake(
                "peer not connected for blocks-by-range request".into(),
            ));
        };
        let _ = self
            .swarm
            .behaviour_mut()
            .blocks_by_range
            .send_request(&peer_id, payload);
        Ok(())
    }
}
