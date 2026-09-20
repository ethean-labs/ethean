//! Operator vs interim fork-digest policy for mesh joins.

use crate::network_target::{NetworkId, NetworkTarget};

/// How the gossip fork segment was chosen for this start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForkDigestSource {
    /// `--fork-digest`, `ETHEAN_FORK_DIGEST`, or `*.forkdigest` file.
    OperatorOverride,
    /// Interim SHA-256(fork_name) prefix (local smoke / missing pin).
    InterimNameHash,
}

impl NetworkTarget {
    /// Where the gossip fork segment will come from after resolve.
    pub fn fork_digest_source(&self) -> ForkDigestSource {
        if self.fork_digest.as_ref().is_some_and(|s| !s.trim().is_empty()) {
            ForkDigestSource::OperatorOverride
        } else {
            ForkDigestSource::InterimNameHash
        }
    }

    /// True when a pq-devnet label is using the interim hash (isolation risk on mesh).
    pub fn mesh_isolation_risk(&self) -> bool {
        matches!(self.id, NetworkId::PqDevnet4 | NetworkId::PqDevnet5)
            && self.fork_digest_source() == ForkDigestSource::InterimNameHash
            && self.has_bootnodes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_override_clears_isolation_risk() {
        let mut t = NetworkTarget::pq_devnet_5();
        t.bootnodes.push("/ip4/127.0.0.1/udp/9/quic-v1".into());
        assert!(t.mesh_isolation_risk());
        t.fork_digest = Some("aabbccdd".into());
        assert!(!t.mesh_isolation_risk());
        assert_eq!(t.fork_digest_source(), ForkDigestSource::OperatorOverride);
    }

    #[test]
    fn local_smoke_has_no_mesh_isolation_flag() {
        let t = NetworkTarget {
            id: NetworkId::Local,
            bootnodes: vec!["/ip4/127.0.0.1/udp/9/quic-v1".into()],
            fork_digest: None,
        };
        assert!(!t.mesh_isolation_risk());
    }
}
