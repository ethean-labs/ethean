//! QUIC transport facade (not yet bound to libp2p).

use crate::error::{NetworkError, Result};
use crate::identity::NodeIdentity;

/// Desired listen configuration for QUIC-v1 / UDP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportConfig {
    /// UDP listen port.
    pub listen_port: u16,
    /// Idle timeout milliseconds.
    pub idle_timeout_ms: u64,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            listen_port: 9000,
            idle_timeout_ms: 30_000,
        }
    }
}

/// Build marker proving identity + config are present; real dial is Phase open gate.
pub fn prepare_transport(_identity: &NodeIdentity, _cfg: &TransportConfig) -> Result<()> {
    Err(NetworkError::TransportPending(
        "libp2p QUIC-v1 swarm not wired; refuse simulated TCP/WS fallback",
    ))
}
