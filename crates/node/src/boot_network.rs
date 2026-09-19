//! UDP listen + optional QuicSwarm bind used during client boot.

use crate::network::{NodeIdentity, SwarmFacade, TransportConfig};
use crate::network_target::NetworkTarget;
use crate::Result;
use tracing::{info, warn};

/// Bind the UDP facade; with `libp2p-quic`, subscribe using a resolved fork segment.
pub async fn prepare_boot_network(
    fork_segment: &str,
) -> Result<(u16, Option<SwarmFacade>)> {
    let identity = NodeIdentity::from_seed(b"ethean-local");
    let bound = crate::network::prepare_transport(
        &identity,
        &TransportConfig {
            listen_port: 0,
            idle_timeout_ms: 30_000,
        },
    )?;
    let port = bound.listen_port;
    info!(port, fork_segment, "UDP listen bind for QUIC facade");

    #[cfg(feature = "libp2p-quic")]
    {
        let facade = bind_quic_facade(bound, fork_segment).await?;
        return Ok((port, Some(facade)));
    }
    #[cfg(not(feature = "libp2p-quic"))]
    {
        let _ = bound;
        let _ = fork_segment;
        Ok((port, None))
    }
}

/// Dial configured bootnodes on a live QuicSwarm (no-op when list empty).
pub fn dial_bootnodes(target: &NetworkTarget, swarm: Option<&mut SwarmFacade>) {
    if !target.has_bootnodes() {
        match target.id {
            crate::network_target::NetworkId::PqDevnet4 => {
                warn!(
                    network = target.id.as_str(),
                    "no bootnodes configured; running pq-devnet-4 offline (set --bootnodes, ETHEAN_BOOTNODES, or config/networks/pq-devnet-4.bootnodes)"
                );
            }
            crate::network_target::NetworkId::PqDevnet5 => {
                warn!(
                    network = target.id.as_str(),
                    "no bootnodes configured; running pq-devnet-5 label offline (set --bootnodes, ETHEAN_BOOTNODES, or config/networks/pq-devnet-5.bootnodes)"
                );
            }
            crate::network_target::NetworkId::Local => {
                info!(network = target.id.as_str(), "local smoke; skipping mesh dial");
            }
        }
        return;
    }

    #[cfg(feature = "libp2p-quic")]
    {
        let Some(facade) = swarm else {
            warn!(
                n = target.bootnodes.len(),
                "bootnodes set but QuicSwarm missing (build with ethean-node/libp2p-quic)"
            );
            return;
        };
        for addr in &target.bootnodes {
            match facade.dial_quic_peer(addr) {
                Ok(()) => info!(%addr, network = target.id.as_str(), "dialed bootnode"),
                Err(e) => warn!(%addr, error = %e, "bootnode dial failed"),
            }
        }
    }
    #[cfg(not(feature = "libp2p-quic"))]
    {
        let _ = swarm;
        warn!(
            n = target.bootnodes.len(),
            "bootnodes set but libp2p-quic feature is off; rebuild with --features libp2p-quic"
        );
    }
}

#[cfg(feature = "libp2p-quic")]
async fn bind_quic_facade(
    bound: crate::network::BoundTransport,
    fork_segment: &str,
) -> Result<SwarmFacade> {
    let mut facade = SwarmFacade::default();
    facade.attach_transport(bound);
    facade
        .bind_quic_swarm_for_fork_segment(
            &TransportConfig {
                listen_port: 0,
                idle_timeout_ms: 30_000,
            },
            fork_segment,
        )
        .await?;
    let topic_block = facade
        .quic
        .as_ref()
        .and_then(|q| q.topics.as_ref())
        .map(|t| t.block.clone())
        .unwrap_or_default();
    let peer = facade
        .quic
        .as_ref()
        .map(|q| q.peer_id.to_string())
        .unwrap_or_default();
    let listen = facade
        .quic
        .as_ref()
        .map(|q| q.listen_addr.to_string())
        .unwrap_or_default();
    let dialable = if peer.is_empty() || listen.is_empty() {
        String::new()
    } else {
        format!("{listen}/p2p/{peer}")
    };
    info!(
        %peer,
        %listen,
        %dialable,
        topic = %topic_block,
        fork_segment,
        "libp2p QuicSwarm bound with Lean gossip topics"
    );
    Ok(facade)
}
