//! Decode fixed-width hex (optional `0x` prefix).

use crate::error::GenesisError;

/// Parse a hex string into exactly `N` bytes.
pub fn decode_hex_fixed<const N: usize>(s: &str) -> Result<[u8; N], GenesisError> {
    let t = s.trim();
    let hex = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")).unwrap_or(t);
    if hex.len() != N * 2 {
        return Err(GenesisError::LeanConfig(format!(
            "hex length {} != {} (need {} bytes)",
            hex.len(),
            N * 2,
            N
        )));
    }
    let mut out = [0u8; N];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let hi = from_nibble(chunk[0])?;
        let lo = from_nibble(chunk[1])?;
        out[i] = (hi << 4) | lo;
    }
    Ok(out)
}

fn from_nibble(b: u8) -> Result<u8, GenesisError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(GenesisError::LeanConfig(format!(
            "invalid hex digit {}",
            b as char
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_with_and_without_prefix() {
        let a = decode_hex_fixed::<2>("0xabcd").unwrap();
        let b = decode_hex_fixed::<2>("ABCD").unwrap();
        assert_eq!(a, [0xab, 0xcd]);
        assert_eq!(b, [0xab, 0xcd]);
    }
}
