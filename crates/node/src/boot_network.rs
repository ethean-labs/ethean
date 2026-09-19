//! UDP listen + optional QuicSwarm bind used during client boot.

use crate::network::{NodeIdentity, SwarmFacade, TransportConfig};
use crate::Result;
use tracing::info;

/// Bind the UDP facade; with `libp2p-quic`, also subscribe Lean gossip topics.
pub async fn prepare_boot_network(
    fork_name: &str,
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
    info!(port, "UDP listen bind for QUIC facade");

    #[cfg(feature = "libp2p-quic")]
    {
        let facade = bind_quic_facade(bound, fork_name).await?;
        return Ok((port, Some(facade)));
    }
    #[cfg(not(feature = "libp2p-quic"))]
    {
        let _ = bound;
        let _ = fork_name;
        Ok((port, None))
    }
}

#[cfg(feature = "libp2p-quic")]
async fn bind_quic_facade(
    bound: crate::network::BoundTransport,
    fork_name: &str,
) -> Result<SwarmFacade> {
    let mut facade = SwarmFacade::default();
    facade.attach_transport(bound);
    facade
        .bind_quic_swarm_for_fork(
            &TransportConfig {
                listen_port: 0,
                idle_timeout_ms: 30_000,
            },
            fork_name,
        )
        .await?;
    let topic_block = facade
        .quic
        .as_ref()
        .and_then(|q| q.topics.as_ref())
        .map(|t| t.block.clone())
        .unwrap_or_default();
    info!(
        peer = %facade.quic.as_ref().map(|q| q.peer_id.to_string()).unwrap_or_default(),
        listen = %facade.quic.as_ref().map(|q| q.listen_addr.to_string()).unwrap_or_default(),
        topic = %topic_block,
        "libp2p QuicSwarm bound with Lean gossip topics"
    );
    Ok(facade)
}
