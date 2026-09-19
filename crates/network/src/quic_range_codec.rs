//! Framed-Snappy blocks-by-range codec for libp2p request_response.

#![cfg(feature = "libp2p-quic")]

use crate::quic_framed::{read_framed, write_framed};
use async_trait::async_trait;
use ethean_network_wire::rpc_blocks_by_range;
use futures::prelude::*;
use libp2p::request_response;
use libp2p::swarm::StreamProtocol;
use std::io;

/// Max framed blocks-by-range request/response size (compressed).
const MAX_RANGE_FRAME: u64 = 8 * 1024 * 1024;

/// Lean blocks-by-range codec over `/leanconsensus/req/blocks_by_range/1/ssz_snappy`.
#[derive(Debug, Clone, Default)]
pub struct BlocksByRangeCodec;

/// Build the blocks-by-range request_response behaviour.
pub fn blocks_by_range_behaviour() -> request_response::Behaviour<BlocksByRangeCodec> {
    let proto = StreamProtocol::new(rpc_blocks_by_range());
    request_response::Behaviour::new(
        [(proto, request_response::ProtocolSupport::Full)],
        request_response::Config::default(),
    )
}

#[async_trait]
impl request_response::Codec for BlocksByRangeCodec {
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
        read_framed(io, MAX_RANGE_FRAME).await
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        read_framed(io, MAX_RANGE_FRAME).await
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
        write_framed(io, &req).await
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
        write_framed(io, &res).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn behaviour_advertises_lean_blocks_by_range() {
        let _ = blocks_by_range_behaviour();
        assert!(rpc_blocks_by_range().contains("/leanconsensus/req/blocks_by_range/"));
    }
}
