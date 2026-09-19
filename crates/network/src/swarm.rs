//! Swarm event-loop facade (UDP bind + optional libp2p QUIC swarm).

use crate::error::{NetworkError, Result};
use crate::peer_manager::PeerManager;
use crate::reqresp::RequestTracker;
use crate::transport::{dial_quic, BoundTransport};

#[cfg(feature = "libp2p-quic")]
use crate::quic_swarm::QuicSwarm;
#[cfg(feature = "libp2p-quic")]
use crate::transport::TransportConfig;

/// High-level swarm state; may hold a bound UDP listen socket and/or QUIC swarm.
#[derive(Debug, Default)]
pub struct SwarmFacade {
    pub peers: PeerManager,
    pub requests: RequestTracker,
    pub events_processed: u64,
    /// Present after [`Self::attach_transport`].
    pub transport: Option<BoundTransport>,
    /// Present after [`Self::bind_quic_swarm`] (`libp2p-quic` feature).
    #[cfg(feature = "libp2p-quic")]
    pub quic: Option<QuicSwarm>,
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

    /// True when a libp2p QUIC swarm is bound.
    #[cfg(feature = "libp2p-quic")]
    pub fn has_quic_swarm(&self) -> bool {
        self.quic.is_some()
    }

    /// Bind a real libp2p QUIC-v1 swarm (replaces UDP-only facade for dial).
    #[cfg(feature = "libp2p-quic")]
    pub async fn bind_quic_swarm(&mut self, cfg: &TransportConfig) -> Result<()> {
        let swarm = QuicSwarm::bind(cfg).await?;
        self.quic = Some(swarm);
        self.note_progress();
        Ok(())
    }

    /// Bind QUIC and subscribe to Lean gossip topics for `fork_name`.
    #[cfg(feature = "libp2p-quic")]
    pub async fn bind_quic_swarm_for_fork(
        &mut self,
        cfg: &TransportConfig,
        fork_name: &str,
    ) -> Result<()> {
        let swarm = QuicSwarm::bind_for_fork(cfg, fork_name).await?;
        self.quic = Some(swarm);
        self.note_progress();
        Ok(())
    }

    /// Publish compressed gossip on a Lean topic via the bound swarm.
    #[cfg(feature = "libp2p-quic")]
    pub fn publish_gossip(&mut self, topic: &str, compressed: &[u8]) -> Result<()> {
        let Some(swarm) = self.quic.as_mut() else {
            return Err(NetworkError::TransportPending(
                "bind_quic_swarm before publish_gossip",
            ));
        };
        swarm.publish_gossip(topic, compressed)?;
        self.note_progress();
        Ok(())
    }

    /// Dial via the bound libp2p QUIC swarm.
    #[cfg(feature = "libp2p-quic")]
    pub fn dial_quic_peer(&mut self, multiaddr: &str) -> Result<()> {
        let Some(swarm) = self.quic.as_mut() else {
            return Err(NetworkError::TransportPending(
                "bind_quic_swarm before dial_quic_peer",
            ));
        };
        swarm.dial(multiaddr)?;
        self.note_progress();
        Ok(())
    }

    /// Pump one libp2p event when a QuicSwarm is bound.
    #[cfg(feature = "libp2p-quic")]
    pub async fn pump_quic_once(&mut self) -> Result<&'static str> {
        let Some(swarm) = self.quic.as_mut() else {
            return Err(NetworkError::TransportPending(
                "bind_quic_swarm before pump_quic_once",
            ));
        };
        let kind = swarm.pump_once().await;
        self.note_progress();
        Ok(kind)
    }

    /// Record a tick of the (future) event loop for health.
    pub fn note_progress(&mut self) {
        self.events_processed = self.events_processed.saturating_add(1);
    }

    /// Healthy if the facade has observed progress since start.
    pub fn is_progressing(&self) -> bool {
        self.events_processed > 0
    }

    /// Dial without libp2p swarm: UDP bind required, then pending or feature path.
    pub fn dial_static_peer(&self, multiaddr: &str) -> Result<()> {
        let Some(bound) = self.transport.as_ref() else {
            return Err(NetworkError::TransportPending(
                "attach UDP listen bind before dial",
            ));
        };
        dial_quic(bound, multiaddr)
    }

    /// UDP Status path probe (not a QUIC crypto handshake).
    pub fn probe_peer_status(
        &self,
        multiaddr: &str,
        local: &ethean_network_wire::Status,
        timeout: std::time::Duration,
    ) -> Result<crate::dial::UdpDialProbe> {
        let Some(bound) = self.transport.as_ref() else {
            return Err(NetworkError::TransportPending(
                "attach UDP listen bind before status probe",
            ));
        };
        crate::probe_udp_status(bound, multiaddr, local, timeout)
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

    #[cfg(feature = "libp2p-quic")]
    #[tokio::test]
    async fn bind_quic_enables_swarm_dial() {
        let mut s = SwarmFacade::default();
        s.bind_quic_swarm(&TransportConfig {
            listen_port: 0,
            idle_timeout_ms: 1_000,
        })
        .await
        .expect("quic swarm");
        assert!(s.has_quic_swarm());
        assert!(s.dial_quic_peer("/ip4/127.0.0.1/tcp/9").is_err());
        assert!(s.dial_quic_peer("/ip4/127.0.0.1/udp/9/quic-v1").is_ok());
    }
}
