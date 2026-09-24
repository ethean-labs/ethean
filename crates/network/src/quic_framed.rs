//! Spec req/resp framed I/O: varint length + snappy frame (feature `libp2p-quic`).

#![cfg(feature = "libp2p-quic")]

use ethean_network_wire::{
    decode_request, decode_response_stream, encode_request, encode_response, ResponseCode,
    MAX_PAYLOAD_SIZE,
};
use futures::prelude::*;
use std::io;

/// Read one request: varint + snappy framed SSZ, up to `max_frame` compressed bytes.
pub async fn read_request<T>(io: &mut T, max_frame: u64) -> io::Result<Vec<u8>>
where
    T: AsyncRead + Unpin + Send,
{
    let mut buf = Vec::new();
    io.take(max_frame).read_to_end(&mut buf).await?;
    decode_request(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

/// Write one request (varint + snappy frame) and close the stream.
pub async fn write_request<T>(io: &mut T, plain: &[u8]) -> io::Result<()>
where
    T: AsyncWrite + Unpin + Send,
{
    if plain.len() > MAX_PAYLOAD_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "payload exceeds MAX_PAYLOAD_SIZE",
        ));
    }
    let framed = encode_request(plain)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    io.write_all(&framed).await?;
    io.close().await?;
    Ok(())
}

/// Read a concatenated response stream (one or more code+varint+frame chunks) to EOF.
pub async fn read_response_stream<T>(io: &mut T, max_frame: u64) -> io::Result<Vec<u8>>
where
    T: AsyncRead + Unpin + Send,
{
    let mut buf = Vec::new();
    io.take(max_frame).read_to_end(&mut buf).await?;
    Ok(buf)
}

/// Write a SUCCESS response chunk (single payload) and close.
pub async fn write_success<T>(io: &mut T, plain: &[u8]) -> io::Result<()>
where
    T: AsyncWrite + Unpin + Send,
{
    let framed = encode_response(ResponseCode::Success, plain)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    io.write_all(&framed).await?;
    io.close().await?;
    Ok(())
}

/// Write already-encoded response-stream bytes (one or more chunks) and close.
pub async fn write_response_bytes<T>(io: &mut T, framed: &[u8]) -> io::Result<()>
where
    T: AsyncWrite + Unpin + Send,
{
    io.write_all(framed).await?;
    io.close().await?;
    Ok(())
}

/// Back-compat aliases used by Status (single SUCCESS chunk).
pub async fn read_framed<T>(io: &mut T, max_frame: u64) -> io::Result<Vec<u8>>
where
    T: AsyncRead + Unpin + Send,
{
    let buf = read_response_stream(io, max_frame).await?;
    let chunks = decode_response_stream(&buf)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    match chunks.into_iter().next() {
        Some(c) if c.code == ResponseCode::Success => Ok(c.payload),
        Some(_) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "non-success status response",
        )),
        None => Ok(Vec::new()),
    }
}

/// Write one SUCCESS framed payload (Status).
pub async fn write_framed<T>(io: &mut T, plain: &[u8]) -> io::Result<()>
where
    T: AsyncWrite + Unpin + Send,
{
    write_success(io, plain).await
}

/// Read a request using the request (no code byte) framing.
pub async fn read_framed_request<T>(io: &mut T, max_frame: u64) -> io::Result<Vec<u8>>
where
    T: AsyncRead + Unpin + Send,
{
    read_request(io, max_frame).await
}

/// Write a request using the request (no code byte) framing.
pub async fn write_framed_request<T>(io: &mut T, plain: &[u8]) -> io::Result<()>
where
    T: AsyncWrite + Unpin + Send,
{
    write_request(io, plain).await
}
