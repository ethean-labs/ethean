//! Hex helpers for leanSpec JSON (`0x…` roots and keys).

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum HexError {
    #[error("expected 0x-prefixed hex, got {0}")]
    BadPrefix(String),
    #[error("invalid hex digit in {0}")]
    BadDigit(String),
    #[error("expected {expected} bytes, got {got}")]
    BadLength { expected: usize, got: usize },
}

/// Decode `0x`-prefixed hex into a fixed-size array.
pub fn decode_hex_fixed<const N: usize>(s: &str) -> Result<[u8; N], HexError> {
    let raw = s.strip_prefix("0x").unwrap_or(s);
    if raw.len() != N * 2 {
        return Err(HexError::BadLength {
            expected: N,
            got: raw.len() / 2,
        });
    }
    let mut out = [0u8; N];
    for (i, chunk) in raw.as_bytes().chunks(2).enumerate() {
        let hi = from_digit(chunk[0], s)?;
        let lo = from_digit(chunk[1], s)?;
        out[i] = (hi << 4) | lo;
    }
    Ok(out)
}

fn from_digit(b: u8, ctx: &str) -> Result<u8, HexError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(HexError::BadDigit(ctx.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_32_byte_root() {
        let z = "0x".to_string() + &"00".repeat(32);
        assert_eq!(decode_hex_fixed::<32>(&z).unwrap(), [0u8; 32]);
    }
}
