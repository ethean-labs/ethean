//! Pending Status handshake sessions keyed by peer fingerprint.

use crate::error::{NetworkError, Result};
use crate::reqresp::handler::{handle_status, StatusExchange};
use ethean_network_wire::{rpc_status, Status};
use ethean_primitives::Hash32;
use std::collections::HashMap;

/// Book of in-flight Status exchanges (stream send still pending libp2p req/resp).
#[derive(Debug, Default)]
pub struct StatusSessionBook {
    /// Local Status awaiting a remote reply, keyed by peer fingerprint.
    pending: HashMap<Hash32, Status>,
    /// Completed compatible exchanges.
    completed: Vec<StatusExchange>,
}

impl StatusSessionBook {
    /// Lean Status protocol id.
    pub fn protocol_id() -> &'static str {
        rpc_status()
    }

    /// Queue a Status exchange when a peer connects.
    pub fn on_peer_connected(&mut self, peer: Hash32, local: Status) {
        self.pending.insert(peer, local);
    }

    /// Drop pending state when a peer disconnects.
    pub fn on_peer_disconnected(&mut self, peer: &Hash32) {
        self.pending.remove(peer);
    }

    /// Encode the local Status payload for an outbound Status request.
    pub fn encode_local_for(&self, peer: &Hash32) -> Result<Vec<u8>> {
        let local = self
            .pending
            .get(peer)
            .ok_or_else(|| NetworkError::Handshake("no pending status for peer".into()))?;
        local
            .encode()
            .map_err(|e| NetworkError::Handshake(e.to_string()))
    }

    /// Ingest remote Status bytes for a pending peer; fail closed on mismatch.
    pub fn ingest_remote(&mut self, peer: Hash32, remote_bytes: &[u8]) -> Result<StatusExchange> {
        let local = self
            .pending
            .remove(&peer)
            .ok_or_else(|| NetworkError::Handshake("no pending status for peer".into()))?;
        let remote = Status::decode(remote_bytes)
            .map_err(|e| NetworkError::Handshake(e.to_string()))?;
        let exchange = handle_status(&local, remote)?;
        self.completed.push(exchange.clone());
        Ok(exchange)
    }

    /// Pending peer count.
    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    /// Drain completed exchanges.
    pub fn take_completed(&mut self) -> Vec<StatusExchange> {
        std::mem::take(&mut self.completed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(fork: &str) -> Status {
        Status {
            genesis_root: [1u8; 32],
            fork_segment: fork.into(),
            head_slot: 4,
            head_root: [2u8; 32],
            finalized_slot: 0,
            finalized_root: [0u8; 32],
        }
    }

    #[test]
    fn roundtrip_compatible_peer() {
        let peer = [9u8; 32];
        let mut book = StatusSessionBook::default();
        let local = sample("aabbccdd");
        book.on_peer_connected(peer, local.clone());
        let enc = book.encode_local_for(&peer).unwrap();
        let mut remote = local.clone();
        remote.head_slot = 20;
        remote.head_root = [7u8; 32];
        let rem_bytes = remote.encode().unwrap();
        let ex = book.ingest_remote(peer, &rem_bytes).unwrap();
        assert_eq!(ex.remote.head_slot, 20);
        assert_eq!(book.take_completed().len(), 1);
        assert_eq!(book.pending_len(), 0);
        let _ = enc;
    }

    #[test]
    fn rejects_fork_mismatch() {
        let peer = [3u8; 32];
        let mut book = StatusSessionBook::default();
        book.on_peer_connected(peer, sample("aabbccdd"));
        let bad = sample("ffffffff").encode().unwrap();
        assert!(book.ingest_remote(peer, &bad).is_err());
    }

    #[test]
    fn protocol_id_is_lean() {
        assert!(StatusSessionBook::protocol_id().starts_with("/leanconsensus/req/status/"));
    }
}
