//! Unsigned LEB128 varint (libp2p / protobuf / leanSpec req/resp).

use crate::error::{Result, WireError};

/// Maximum encoded length of a 64-bit unsigned varint.
pub const MAX_VARINT_BYTES: usize = 10;

/// Encode `value` as unsigned LEB128.
pub fn encode_varint(mut value: u64) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(MAX_VARINT_BYTES);
    while value >= 0x80 {
        encoded.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    encoded.push(value as u8);
    encoded
}

/// Decode an unsigned LEB128 varint from `data[offset..]`.
///
/// Returns `(value, bytes_consumed)`.
pub fn decode_varint(data: &[u8], offset: usize) -> Result<(u64, usize)> {
    decode_varint_capped(data, offset, MAX_VARINT_BYTES)
}

/// Decode with an explicit byte cap (5 for 32-bit, 10 for 64-bit).
pub fn decode_varint_capped(data: &[u8], offset: usize, max_bytes: usize) -> Result<(u64, usize)> {
    let mut decoded: u64 = 0;
    let mut shift = 0;
    let mut pos = offset;
    loop {
        if pos >= data.len() {
            return Err(WireError::Varint("truncated varint".into()));
        }
        let byte = data[pos];
        pos += 1;
        decoded |= u64::from(byte & 0x7f) << shift;
        shift += 7;
        if byte & 0x80 == 0 {
            break;
        }
        if pos - offset >= max_bytes {
            return Err(WireError::Varint(format!(
                "varint exceeds {max_bytes} bytes"
            )));
        }
        if shift >= 64 {
            return Err(WireError::Varint("varint overflow".into()));
        }
    }
    Ok((decoded, pos - offset))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    fn roundtrip(value: u64, expected_hex: &str) {
        let enc = encode_varint(value);
        assert_eq!(hex(&enc), expected_hex, "encode {value}");
        let (dec, n) = decode_varint(&enc, 0).unwrap();
        assert_eq!(dec, value);
        assert_eq!(n, enc.len());
    }

    #[test]
    fn leanspec_varint_vectors() {
        roundtrip(0, "00");
        roundtrip(1, "01");
        roundtrip(127, "7f");
        roundtrip(128, "8001");
        roundtrip(150, "9601");
        roundtrip(255, "ff01");
        roundtrip(256, "8002");
        roundtrip(300, "ac02");
        roundtrip(16383, "ff7f");
        roundtrip(16384, "808001");
        roundtrip(2_097_151, "ffff7f");
        roundtrip(268_435_455, "ffffff7f");
        roundtrip(u32::MAX as u64, "ffffffff0f");
        roundtrip(u64::MAX, "ffffffffffffffffff01");
    }

    #[test]
    fn truncated_is_error() {
        assert!(decode_varint(&[0x80], 0).is_err());
        assert!(decode_varint(&[], 0).is_err());
    }
}
