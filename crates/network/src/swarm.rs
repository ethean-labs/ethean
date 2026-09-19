//! Swarm event-loop facade (UDP bind + pending QUIC dial).

use crate::error::{NetworkError, Result};
use crate::peer_manager::PeerManager;
use crate::reqresp::RequestTracker;
use crate::transport::{dial_quic, BoundTransport};

/// High-level swarm state; may hold a bound UDP listen socket.
#[derive(Debug, Default)]
pub struct SwarmFacade {
    pub peers: PeerManager,
    pub requests: RequestTracker,
    pub events_processed: u64,
    /// Present after [`Self::attach_transport`].
    pub transport: Option<BoundTransport>,
}

impl SwarmFacade {
    /// Attach a bound UDP/QUIC listen socket from [`crate::prepare_transport`].
    pub fn attach_transport(&mut self, bound: BoundTransport) {
        self.transport = Some(bound);
        self.note_progress();
    }

    /// True when a listen socket is held.
    pub fn has_listen_bind(&self) -> bool {
        self.transport.is_some()
    }

    /// Record a tick of the (future) event loop for health.
    pub fn note_progress(&mut self) {
        self.events_processed = self.events_processed.saturating_add(1);
    }

    /// Healthy if the facade has observed progress since start.
    pub fn is_progressing(&self) -> bool {
        self.events_processed > 0
    }

    /// Dial requires an attached bind; libp2p QUIC swarm still pending.
    pub fn dial_static_peer(&self, multiaddr: &str) -> Result<()> {
        let Some(bound) = self.transport.as_ref() else {
            return Err(NetworkError::TransportPending(
                "attach UDP listen bind before dial",
            ));
        };
        dial_quic(bound, multiaddr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::NodeIdentity;
    use crate::transport::{prepare_transport, TransportConfig};

    #[test]
    fn progress_flag() {
        let mut s = SwarmFacade::default();
        assert!(!s.is_progressing());
        s.note_progress();
        assert!(s.is_progressing());
    }

    #[test]
    fn attach_bind_enables_dial_path() {
        let mut s = SwarmFacade::default();
        let id = NodeIdentity::from_seed(b"swarm");
        let bound = prepare_transport(
            &id,
            &TransportConfig {
                listen_port: 0,
                idle_timeout_ms: 1_000,
            },
        )
        .unwrap();
        s.attach_transport(bound);
        assert!(s.has_listen_bind());
        assert!(s.dial_static_peer("/ip4/127.0.0.1/udp/9/quic-v1").is_err());
    }
}
