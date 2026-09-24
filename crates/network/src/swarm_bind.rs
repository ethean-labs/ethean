//! QUIC swarm bind variants for [`SwarmFacade`] (`libp2p-quic` feature).

#![cfg(feature = "libp2p-quic")]

use crate::error::Result;
use crate::quic_swarm::QuicSwarm;
use crate::swarm::SwarmFacade;
use crate::transport::{ListenIdentity, TransportConfig};

impl SwarmFacade {
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
        self.bind_quic_swarm_for_fork_segment_subnets(
            cfg,
            fork_segment,
            crate::SMOKE_ATTESTATION_SUBNETS,
        )
        .await
    }

    /// Bind QUIC with an explicit attestation subnet subscription count.
    #[cfg(feature = "libp2p-quic")]
    pub async fn bind_quic_swarm_for_fork_segment_subnets(
        &mut self,
        cfg: &TransportConfig,
        fork_segment: &str,
        attestation_subnets: u16,
    ) -> Result<()> {
        let swarm = QuicSwarm::bind_for_fork_segment_subnets(
            cfg,
            fork_segment,
            attestation_subnets,
        )
        .await?;
        self.quic = Some(swarm);
        self.note_progress();
        Ok(())
    }

    /// Bind QUIC on `who.listen_ip` with the node key from `who` (Hive / quickstart identity).
    #[cfg(feature = "libp2p-quic")]
    pub async fn bind_quic_swarm_with_identity(
        &mut self,
        cfg: &TransportConfig,
        fork_segment: &str,
        attestation_subnets: u16,
        who: &ListenIdentity,
    ) -> Result<()> {
        let swarm = QuicSwarm::bind_with_identity(cfg, fork_segment, attestation_subnets, who).await?;
        self.quic = Some(swarm);
        self.note_progress();
        Ok(())
    }
}
