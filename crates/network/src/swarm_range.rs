//! Blocks-by-range outbox and serve-cache helpers on SwarmFacade.

#[cfg(feature = "libp2p-quic")]
use crate::error::{NetworkError, Result};
use crate::reqresp::OutboundBlocksByRangeRequest;
use crate::swarm::SwarmFacade;
#[cfg(feature = "libp2p-quic")]
use ethean_primitives::Hash32;

impl SwarmFacade {
    /// Index a block body by slot for inbound blocks-by-range replies.
    #[cfg(feature = "libp2p-quic")]
    pub fn put_block_at_slot(
        &mut self,
        slot: u64,
        root: Hash32,
        bytes: Vec<u8>,
    ) -> Result<()> {
        let Some(swarm) = self.quic.as_mut() else {
            return Err(NetworkError::TransportPending(
                "bind_quic_swarm before put_block_at_slot",
            ));
        };
        swarm.put_block_at_slot(slot, root, bytes);
        self.note_progress();
        Ok(())
    }

    /// Stage encoded blocks-by-range requests for later stream send.
    pub fn enqueue_blocks_range_outbounds(&mut self, reqs: Vec<OutboundBlocksByRangeRequest>) {
        self.blocks_range_outbox.extend(reqs);
        self.note_progress();
    }

    /// Drain staged blocks-by-range outbox payloads.
    pub fn take_blocks_range_outbox(&mut self) -> Vec<OutboundBlocksByRangeRequest> {
        std::mem::take(&mut self.blocks_range_outbox)
    }

    /// Flush staged blocks-by-range outbox payloads over Lean req/resp streams.
    #[cfg(feature = "libp2p-quic")]
    pub fn flush_blocks_range_outbox(&mut self) -> Result<usize> {
        let Some(swarm) = self.quic.as_mut() else {
            return Err(NetworkError::TransportPending(
                "bind_quic_swarm before flush_blocks_range_outbox",
            ));
        };
        let pending = std::mem::take(&mut self.blocks_range_outbox);
        let mut sent = 0;
        for req in pending {
            swarm.send_blocks_by_range_request(req.peer, req.payload)?;
            sent += 1;
        }
        if sent > 0 {
            self.note_progress();
        }
        Ok(sent)
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

    #[test]
    fn range_outbox_roundtrip() {
        let mut s = SwarmFacade::default();
        s.enqueue_blocks_range_outbounds(vec![OutboundBlocksByRangeRequest {
            peer: [1u8; 32],
            request_id: crate::reqresp::RequestId(1),
            protocol_id: "test",
            payload: vec![1, 2, 3],
        }]);
        assert_eq!(s.take_blocks_range_outbox().len(), 1);
        assert!(s.take_blocks_range_outbox().is_empty());
    }
}
