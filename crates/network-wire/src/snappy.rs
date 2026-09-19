//! Raw Snappy (gossip) and framed Snappy (req/resp) codecs with bomb guards.

use crate::error::{Result, WireError};
use crate::limits::{
    MAX_COMPRESSED_GOSSIP_BYTES, MAX_DECOMPRESSED_BYTES, MAX_SNAPPY_EXPANSION_RATIO,
};

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
    check_expansion(compressed.len(), plain.len())?;
    if plain.len() > MAX_DECOMPRESSED_BYTES {
        return Err(WireError::PayloadTooLarge {
            got: plain.len(),
            max: MAX_DECOMPRESSED_BYTES,
        });
    }
    Ok(plain)
}

/// Framed Snappy: u32 LE length of compressed frame || raw-snappy bytes.
pub fn compress_framed(plain: &[u8]) -> Result<Vec<u8>> {
    let body = compress_raw(plain)?;
    let mut out = Vec::with_capacity(4 + body.len());
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(&body);
    Ok(out)
}

/// Decompress one framed Snappy message; rejects trailing bytes.
pub fn decompress_framed(input: &[u8]) -> Result<Vec<u8>> {
    if input.len() < 4 {
        return Err(WireError::Snappy("framed header too short".into()));
    }
    let len = u32::from_le_bytes(input[0..4].try_into().unwrap()) as usize;
    if input.len() != 4 + len {
        return Err(WireError::TrailingBytes);
    }
    decompress_raw(&input[4..])
}

fn check_expansion(compressed: usize, plain: usize) -> Result<()> {
    if compressed == 0 {
        return Err(WireError::Snappy("empty compressed payload".into()));
    }
    if plain / compressed > MAX_SNAPPY_EXPANSION_RATIO {
        return Err(WireError::Snappy(format!(
            "expansion ratio {} exceeds {}",
            plain / compressed,
            MAX_SNAPPY_EXPANSION_RATIO
        )));
    }
    Ok(())
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
    fn framed_rejects_trailing() {
        let mut f = compress_framed(b"abc").unwrap();
        f.push(0xff);
        assert_eq!(decompress_framed(&f).unwrap_err(), WireError::TrailingBytes);
    }

    #[test]
    fn raw_vs_framed_not_interchangeable() {
        let raw = compress_raw(b"hello").unwrap();
        assert!(decompress_framed(&raw).is_err());
    }
}
