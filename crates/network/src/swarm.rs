//! Swarm event-loop facade (UDP bind + optional libp2p QUIC swarm).

use crate::error::{NetworkError, Result};
use crate::peer_manager::PeerManager;
use crate::reqresp::{
    OutboundBlocksByRangeRequest, OutboundBlocksByRootRequest, OutboundStatusRequest, RequestTracker,
};
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
    /// Status request payloads staged for Lean req/resp streams (wire send TBD).
    pub status_outbox: Vec<OutboundStatusRequest>,
    /// Blocks-by-root payloads staged after a Status head gap (wire send TBD).
    pub blocks_outbox: Vec<OutboundBlocksByRootRequest>,
    /// Blocks-by-range payloads staged for deep slot catch-up.
    pub blocks_range_outbox: Vec<OutboundBlocksByRangeRequest>,
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

    /// Bind QUIC using a resolved fork segment (operator digest).
    #[cfg(feature = "libp2p-quic")]
    pub async fn bind_quic_swarm_for_fork_segment(
        &mut self,
        cfg: &TransportConfig,
        fork_segment: &str,
    ) -> Result<()> {
        let swarm = QuicSwarm::bind_for_fork_segment(cfg, fork_segment).await?;
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

    /// Pump one libp2p event when a QuicSwarm is bound; apply peer score deltas.
    #[cfg(feature = "libp2p-quic")]
    pub async fn pump_quic_once(&mut self) -> Result<crate::gossip::PumpEvent> {
        let Some(swarm) = self.quic.as_mut() else {
            return Err(NetworkError::TransportPending(
                "bind_quic_swarm before pump_quic_once",
            ));
        };
        let event = swarm.pump_once().await;
        if let Some(g) = event.gossip() {
            let delta = crate::gossip::delta_for(g.action);
            if delta != 0 {
                if let Some(peer) = g.peer {
                    self.peers.ensure_and_feedback(peer, delta);
                }
            }
        }
        self.note_progress();
        Ok(event)
    }

    /// Cache local Status SSZ for inbound Status replies.
    #[cfg(feature = "libp2p-quic")]
    pub fn set_local_status_bytes(&mut self, bytes: Vec<u8>) -> Result<()> {
        let Some(swarm) = self.quic.as_mut() else {
            return Err(NetworkError::TransportPending(
                "bind_quic_swarm before set_local_status_bytes",
            ));
        };
        swarm.set_local_status_bytes(bytes);
        self.note_progress();
        Ok(())
    }

    /// Flush staged Status outbox payloads over Lean Status request streams.
    #[cfg(feature = "libp2p-quic")]
    pub fn flush_status_outbox(&mut self) -> Result<usize> {
        let Some(swarm) = self.quic.as_mut() else {
            return Err(NetworkError::TransportPending(
                "bind_quic_swarm before flush_status_outbox",
            ));
        };
        let pending = std::mem::take(&mut self.status_outbox);
        let mut sent = 0;
        for req in pending {
            swarm.send_status_request(req.peer, req.payload)?;
            sent += 1;
        }
        if sent > 0 {
            self.note_progress();
        }
        Ok(sent)
    }

    /// Flush staged blocks-by-root outbox payloads over Lean req/resp streams.
    #[cfg(feature = "libp2p-quic")]
    pub fn flush_blocks_outbox(&mut self) -> Result<usize> {
        let Some(swarm) = self.quic.as_mut() else {
            return Err(NetworkError::TransportPending(
                "bind_quic_swarm before flush_blocks_outbox",
            ));
        };
        let pending = std::mem::take(&mut self.blocks_outbox);
        let mut sent = 0;
        for req in pending {
            swarm.send_blocks_by_root_request(req.peer, req.payload)?;
            sent += 1;
        }
        if sent > 0 {
            self.note_progress();
        }
        Ok(sent)
    }

    /// Cache a block body for inbound blocks-by-root replies.
    #[cfg(feature = "libp2p-quic")]
    pub fn put_block_bytes(&mut self, root: ethean_primitives::Hash32, bytes: Vec<u8>) -> Result<()> {
        let Some(swarm) = self.quic.as_mut() else {
            return Err(NetworkError::TransportPending(
                "bind_quic_swarm before put_block_bytes",
            ));
        };
        swarm.put_block_bytes(root, bytes);
        self.note_progress();
        Ok(())
    }

    /// Stage encoded Status requests for later stream send.
    pub fn enqueue_status_outbounds(&mut self, reqs: Vec<OutboundStatusRequest>) {
        self.status_outbox.extend(reqs);
        self.note_progress();
    }

    /// Drain staged Status outbox payloads.
    pub fn take_status_outbox(&mut self) -> Vec<OutboundStatusRequest> {
        std::mem::take(&mut self.status_outbox)
    }

    /// Stage encoded blocks-by-root requests for later stream send.
    pub fn enqueue_blocks_outbounds(&mut self, reqs: Vec<OutboundBlocksByRootRequest>) {
        self.blocks_outbox.extend(reqs);
        self.note_progress();
    }

    /// Drain staged blocks-by-root outbox payloads.
    pub fn take_blocks_outbox(&mut self) -> Vec<OutboundBlocksByRootRequest> {
        std::mem::take(&mut self.blocks_outbox)
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
