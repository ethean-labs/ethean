//! Raw Snappy (gossip) and framed Snappy (req/resp) codecs.

use std::io::{Cursor, Read, Write};

use crate::error::{Result, WireError};
use crate::limits::{MAX_COMPRESSED_GOSSIP_BYTES, MAX_DECOMPRESSED_BYTES};

/// Compress with raw Snappy (gossip path).
pub fn compress_raw(plain: &[u8]) -> Result<Vec<u8>> {
    if plain.len() > MAX_DECOMPRESSED_BYTES {
        return Err(WireError::PayloadTooLarge {
            got: plain.len(),
            max: MAX_DECOMPRESSED_BYTES,
        });
    }
    snap::raw::Encoder::new()
        .compress_vec(plain)
        .map_err(|e| WireError::Snappy(e.to_string()))
}

/// Decompress raw Snappy with size and expansion checks.
pub fn decompress_raw(compressed: &[u8]) -> Result<Vec<u8>> {
    if compressed.len() > MAX_COMPRESSED_GOSSIP_BYTES {
        return Err(WireError::PayloadTooLarge {
            got: compressed.len(),
            max: MAX_COMPRESSED_GOSSIP_BYTES,
        });
    }
    let plain = snap::raw::Decoder::new()
        .decompress_vec(compressed)
        .map_err(|e| WireError::Snappy(e.to_string()))?;
    if plain.len() > MAX_DECOMPRESSED_BYTES {
        return Err(WireError::PayloadTooLarge {
            got: plain.len(),
            max: MAX_DECOMPRESSED_BYTES,
        });
    }
    Ok(plain)
}

/// Compress with the Snappy framing format (stream identifier + checksummed chunks).
pub fn compress_frame(plain: &[u8]) -> Result<Vec<u8>> {
    if plain.len() > MAX_DECOMPRESSED_BYTES {
        return Err(WireError::PayloadTooLarge {
            got: plain.len(),
            max: MAX_DECOMPRESSED_BYTES,
        });
    }
    if plain.is_empty() {
        // snap writes nothing for empty input; the spec stream is the bare identifier.
        return Ok(STREAM_IDENTIFIER.to_vec());
    }
    let mut encoder = snap::write::FrameEncoder::new(Vec::new());
    encoder
        .write_all(plain)
        .and_then(|_| encoder.flush())
        .map_err(|e| WireError::Snappy(e.to_string()))?;
    encoder
        .into_inner()
        .map_err(|e| WireError::Snappy(e.error().to_string()))
}

/// Snappy stream identifier (`0xff 06 00 00 sNaPpY`).
pub const STREAM_IDENTIFIER: &[u8] = b"\xff\x06\x00\x00sNaPpY";

/// Decompress the snappy framed stream at the start of `data` that expands to
/// exactly `declared` bytes; returns `(plain, bytes_consumed)`.
///
/// The declared uncompressed length (varint prefix on the wire) bounds the stream,
/// so a following response chunk is never misread as a snappy chunk header.
pub fn decompress_frame_prefix(data: &[u8], declared: usize) -> Result<(Vec<u8>, usize)> {
    if data.len() < STREAM_IDENTIFIER.len() {
        return Err(WireError::Snappy("framed payload too short".into()));
    }
    if !data.starts_with(STREAM_IDENTIFIER) {
        return Err(WireError::Snappy("invalid stream identifier".into()));
    }
    if declared > MAX_DECOMPRESSED_BYTES {
        return Err(WireError::PayloadTooLarge {
            got: declared,
            max: MAX_DECOMPRESSED_BYTES,
        });
    }
    if declared == 0 {
        return Ok((Vec::new(), STREAM_IDENTIFIER.len()));
    }
    let mut cursor = Cursor::new(data);
    let mut decoder = snap::read::FrameDecoder::new(&mut cursor);
    let mut plain = vec![0u8; declared];
    decoder
        .read_exact(&mut plain)
        .map_err(|e| WireError::Snappy(format!("truncated chunk body: {e}")))?;
    drop(decoder);
    Ok((plain, cursor.position() as usize))
}

/// Decompress a complete Snappy framed stream.
pub fn decompress_frame(framed: &[u8]) -> Result<Vec<u8>> {
    if !framed.starts_with(STREAM_IDENTIFIER) {
        return Err(WireError::Snappy("invalid stream identifier".into()));
    }
    if framed.len() > MAX_COMPRESSED_GOSSIP_BYTES {
        return Err(WireError::PayloadTooLarge {
            got: framed.len(),
            max: MAX_COMPRESSED_GOSSIP_BYTES,
        });
    }
    let mut decoder = snap::read::FrameDecoder::new(Cursor::new(framed));
    let mut plain = Vec::new();
    decoder
        .read_to_end(&mut plain)
        .map_err(|e| WireError::Snappy(e.to_string()))?;
    if plain.len() > MAX_DECOMPRESSED_BYTES {
        return Err(WireError::PayloadTooLarge {
            got: plain.len(),
            max: MAX_DECOMPRESSED_BYTES,
        });
    }
    Ok(plain)
}

/// Framed Snappy used on req/resp: alias of [`compress_frame`].
pub fn compress_framed(plain: &[u8]) -> Result<Vec<u8>> {
    compress_frame(plain)
}

/// Decompress one Snappy framed stream; rejects empty input.
pub fn decompress_framed(input: &[u8]) -> Result<Vec<u8>> {
    if input.is_empty() {
        return Err(WireError::Snappy("empty framed payload".into()));
    }
    decompress_frame(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_roundtrip() {
        let msg = b"lean-gossip-payload";
        let c = compress_raw(msg).unwrap();
        assert_eq!(decompress_raw(&c).unwrap(), msg);
    }

    #[test]
    fn frame_roundtrip_and_identifier() {
        let msg = b"Hello, Ethereum!";
        let framed = compress_frame(msg).unwrap();
        assert!(framed.starts_with(b"\xff\x06\x00\x00sNaPpY"));
        assert_eq!(decompress_frame(&framed).unwrap(), msg);
    }

    #[test]
    fn frame_empty_and_repeated() {
        let empty = compress_frame(b"").unwrap();
        assert_eq!(decompress_frame(&empty).unwrap(), b"");
        let repeated = vec![0u8; 1024];
        assert_eq!(decompress_frame(&compress_frame(&repeated).unwrap()).unwrap(), repeated);
    }

    #[test]
    fn frame_rejects_bad_identifier() {
        assert!(decompress_frame(&[0u8; 10]).is_err());
        assert!(decompress_frame(&[]).is_err());
    }

    #[test]
    fn prefix_stops_at_declared_length() {
        let mut stream = compress_frame(b"first").unwrap();
        let first_len = stream.len();
        stream.extend_from_slice(&[0x00, 0x05]);
        stream.extend_from_slice(&compress_frame(b"second").unwrap());
        let (got, used) = decompress_frame_prefix(&stream, 5).unwrap();
        assert_eq!(got, b"first");
        assert_eq!(used, first_len);
        assert!(decompress_frame_prefix(&stream, 6).is_err());
    }

    #[test]
    fn raw_vs_framed_not_interchangeable() {
        let raw = compress_raw(b"hello").unwrap();
        assert!(decompress_framed(&raw).is_err());
    }

    #[test]
    fn leanspec_snappy_block_roundtrips() {
        roundtrip_raw(&[]);
        roundtrip_raw(&[0x42]);
        roundtrip_raw(b"Hello, Ethereum!");
        roundtrip_raw(&vec![0x41; 1000]);
        roundtrip_raw(&b"\xab\xcd".repeat(500));
        roundtrip_raw(&(0u8..=255).collect::<Vec<_>>());
    }

    #[test]
    fn leanspec_snappy_frame_roundtrips() {
        roundtrip_frame(&[]);
        roundtrip_frame(b"Ethereum Snappy!");
        roundtrip_frame(&vec![0u8; 2048]);
        roundtrip_frame(&(0u8..=255).collect::<Vec<_>>());
        let mut ssz_like = (0u8..64).collect::<Vec<_>>();
        ssz_like.extend_from_slice(&[0u8; 448]);
        roundtrip_frame(&ssz_like);
    }

    fn roundtrip_raw(plain: &[u8]) {
        assert_eq!(decompress_raw(&compress_raw(plain).unwrap()).unwrap(), plain);
    }

    fn roundtrip_frame(plain: &[u8]) {
        let framed = compress_frame(plain).unwrap();
        assert!(framed.starts_with(STREAM_IDENTIFIER));
        assert_eq!(decompress_frame(&framed).unwrap(), plain);
        let (got, used) = decompress_frame_prefix(&framed, plain.len()).unwrap();
        assert_eq!(got, plain);
        assert_eq!(used, framed.len());
    }
}
