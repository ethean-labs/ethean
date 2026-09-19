//! Shared framed-Snappy I/O for Lean req/resp codecs (feature `libp2p-quic`).

#![cfg(feature = "libp2p-quic")]

use ethean_network_wire::{compress_framed, decompress_framed};
use futures::prelude::*;
use std::io;

/// Read one framed-Snappy payload up to `max_frame` compressed bytes.
pub async fn read_framed<T>(io: &mut T, max_frame: u64) -> io::Result<Vec<u8>>
where
    T: AsyncRead + Unpin + Send,
{
    let mut buf = Vec::new();
    io.take(max_frame).read_to_end(&mut buf).await?;
    decompress_framed(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

/// Write one framed-Snappy payload and close the stream.
pub async fn write_framed<T>(io: &mut T, plain: &[u8]) -> io::Result<()>
where
    T: AsyncWrite + Unpin + Send,
{
    let framed = compress_framed(plain)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    io.write_all(&framed).await?;
    io.close().await?;
    Ok(())
}
