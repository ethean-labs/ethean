//! Framed-Snappy Status codec for libp2p request_response (feature `libp2p-quic`).

#![cfg(feature = "libp2p-quic")]

use async_trait::async_trait;
use ethean_network_wire::{compress_framed, decompress_framed, rpc_status};
use futures::prelude::*;
use libp2p::request_response;
use libp2p::swarm::StreamProtocol;
use std::io;

/// Max framed Status request/response size (compressed).
const MAX_STATUS_FRAME: u64 = 256 * 1024;

/// Lean Status bytes codec: framed Snappy over `/leanconsensus/req/status/1/ssz_snappy`.
#[derive(Debug, Clone, Default)]
pub struct StatusCodec;

/// Build the Status request_response behaviour.
pub fn status_behaviour() -> request_response::Behaviour<StatusCodec> {
    let proto = StreamProtocol::new(rpc_status());
    request_response::Behaviour::new(
        [(proto, request_response::ProtocolSupport::Full)],
        request_response::Config::default(),
    )
}

#[async_trait]
impl request_response::Codec for StatusCodec {
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
        read_framed(io).await
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        read_framed(io).await
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

async fn read_framed<T>(io: &mut T) -> io::Result<Vec<u8>>
where
    T: AsyncRead + Unpin + Send,
{
    let mut buf = Vec::new();
    io.take(MAX_STATUS_FRAME).read_to_end(&mut buf).await?;
    decompress_framed(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

async fn write_framed<T>(io: &mut T, plain: &[u8]) -> io::Result<()>
where
    T: AsyncWrite + Unpin + Send,
{
    let framed = compress_framed(plain)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    io.write_all(&framed).await?;
    io.close().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn behaviour_advertises_lean_status() {
        let _ = status_behaviour();
        assert!(rpc_status().contains("/leanconsensus/req/status/"));
    }
}
