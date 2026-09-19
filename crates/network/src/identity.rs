//! Local node identity handle (PeerId binding deferred to QUIC transport wiring).

use ethean_primitives::Hash32;
use sha2::{Digest, Sha256};

/// Opaque local identity fingerprint derived from a seed (not a libp2p key yet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeIdentity {
    /// 32-byte fingerprint used until QUIC/TLS identity lands.
    pub fingerprint: Hash32,
}

impl NodeIdentity {
    /// Derive a stable fingerprint from seed bytes (must be persisted by the node).
    pub fn from_seed(seed: &[u8]) -> Self {
        let fingerprint = Sha256::digest(seed).into();
        Self { fingerprint }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_from_seed() {
        let a = NodeIdentity::from_seed(b"ethean-node-1");
        let b = NodeIdentity::from_seed(b"ethean-node-1");
        assert_eq!(a, b);
    }
}
