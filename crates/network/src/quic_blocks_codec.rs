//! Framed-Snappy blocks-by-root codec for libp2p request_response.

#![cfg(feature = "libp2p-quic")]

use crate::quic_framed::{
    read_framed_request, read_response_stream, write_framed_request, write_response_bytes,
};
use async_trait::async_trait;
use ethean_network_wire::rpc_blocks_by_root;
use futures::prelude::*;
use libp2p::request_response;
use libp2p::swarm::StreamProtocol;
use std::io;

/// Max framed blocks-by-root request/response size (compressed).
const MAX_BLOCKS_FRAME: u64 = ethean_network_wire::MAX_MESSAGE_SIZE as u64;

/// Lean blocks-by-root codec over `/leanconsensus/req/blocks_by_root/1/ssz_snappy`.
#[derive(Debug, Clone, Default)]
pub struct BlocksByRootCodec;

/// Build the blocks-by-root request_response behaviour.
pub fn blocks_by_root_behaviour() -> request_response::Behaviour<BlocksByRootCodec> {
    let proto = StreamProtocol::new(rpc_blocks_by_root());
    request_response::Behaviour::new(
        [(proto, request_response::ProtocolSupport::Full)],
        request_response::Config::default(),
    )
}

#[async_trait]
impl request_response::Codec for BlocksByRootCodec {
    type Protocol = StreamProtocol;
    type Request = Vec<u8>;
    type Response = Vec<u8>;

    async fn read_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        read_framed_request(io, MAX_BLOCKS_FRAME).await
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        read_response_stream(io, MAX_BLOCKS_FRAME).await
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_framed_request(io, &req).await
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_response_bytes(io, &res).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn behaviour_advertises_lean_blocks_by_root() {
        let _ = blocks_by_root_behaviour();
        assert!(rpc_blocks_by_root().contains("/leanconsensus/req/blocks_by_root/"));
    }
}
