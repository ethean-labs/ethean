//! Local node identity fingerprint (SHA-256 of the libp2p PeerId when a node key exists).

use crate::node_key::NodeKey;
use ethean_primitives::Hash32;
use sha2::{Digest, Sha256};

/// Local identity fingerprint; matches the peer fingerprint remote nodes compute for us.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeIdentity {
    /// 32-byte fingerprint (SHA-256 of PeerId bytes, or of a seed for tests).
    pub fingerprint: Hash32,
}

impl NodeIdentity {
    /// Derive a stable fingerprint from seed bytes (tests / keyless smoke).
    pub fn from_seed(seed: &[u8]) -> Self {
        let fingerprint = Sha256::digest(seed).into();
        Self { fingerprint }
    }

    /// Fingerprint of the PeerId derived from a secp256k1 node key.
    ///
    /// Without `libp2p-quic` the PeerId cannot be derived, so the fingerprint
    /// falls back to a domain-separated hash of the secret.
    pub fn from_node_key(key: &NodeKey) -> Self {
        #[cfg(feature = "libp2p-quic")]
        {
            if let Ok(peer_id) = key.peer_id() {
                let fingerprint = Sha256::digest(peer_id.to_bytes()).into();
                return Self { fingerprint };
            }
        }
        let mut hasher = Sha256::new();
        hasher.update(b"ethean-node-key-fingerprint");
        hasher.update(key.secret_bytes());
        Self {
            fingerprint: hasher.finalize().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_node_key_is_stable_and_non_zero() {
        let key = NodeKey::from_hex(&"11".repeat(32)).unwrap();
        let a = NodeIdentity::from_node_key(&key);
        assert_eq!(a, NodeIdentity::from_node_key(&key));
        assert_ne!(a.fingerprint, [0u8; 32]);
    }

    #[test]
    fn stable_from_seed() {
        let a = NodeIdentity::from_seed(b"ethean-node-1");
        let b = NodeIdentity::from_seed(b"ethean-node-1");
        assert_eq!(a, b);
    }
}
