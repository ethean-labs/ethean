//! Little-endian SSZ encoding for basic types and offset lists.

use crate::error::SszError;

/// Append a little-endian `u8`.
pub fn encode_u8(out: &mut Vec<u8>, value: u8) {
    out.push(value);
}

/// Append a little-endian `u16`.
pub fn encode_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// Append a little-endian `u32`.
pub fn encode_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// Append a little-endian `u64`.
pub fn encode_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// Append a boolean as `0x00` / `0x01`.
pub fn encode_bool(out: &mut Vec<u8>, value: bool) {
    out.push(u8::from(value));
}

/// Append a fixed-length byte slice as-is.
pub fn encode_fixed_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(bytes);
}

/// Encode a list of already-serialized variable-size elements with SSZ offsets.
///
/// Layout: `offset_0 .. offset_n` (u32 LE each), then concatenated payloads.
/// Empty lists encode to empty bytes.
pub fn encode_offset_list(elements: &[Vec<u8>]) -> Result<Vec<u8>, SszError> {
    if elements.is_empty() {
        return Ok(Vec::new());
    }
    let fixed_end = elements
        .len()
        .checked_mul(4)
        .ok_or(SszError::ListTooLong {
            got: elements.len(),
            limit: usize::MAX / 4,
        })?;
    let mut out = Vec::with_capacity(fixed_end + elements.iter().map(Vec::len).sum::<usize>());
    let mut cursor = fixed_end as u32;
    for el in elements {
        encode_u32(&mut out, cursor);
        cursor = cursor
            .checked_add(el.len() as u32)
            .ok_or(SszError::BytesTooLong {
                got: el.len(),
                limit: u32::MAX as usize,
            })?;
    }
    for el in elements {
        out.extend_from_slice(el);
    }
    Ok(out)
}

/// Encode a bitlist: packed bits plus a delimiting `1` bit after the last data bit.
pub fn encode_bitlist(bits: &[bool]) -> Vec<u8> {
    let bit_len = bits.len() + 1; // delimiter
    let byte_len = (bit_len + 7) / 8;
    let mut out = vec![0u8; byte_len];
    for (i, &bit) in bits.iter().enumerate() {
        if bit {
            out[i / 8] |= 1 << (i % 8);
        }
    }
    let delim = bits.len();
    out[delim / 8] |= 1 << (delim % 8);
    out
}

/// Encode a byte list as raw bytes (caller supplies limit checks).
pub fn encode_byte_list(bytes: &[u8]) -> Vec<u8> {
    bytes.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_u64_le() {
        let mut out = Vec::new();
        encode_u64(&mut out, 0x0102_0304_0506_0708);
        assert_eq!(out, [0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
    }

    #[test]
    fn encodes_empty_offset_list() {
        assert!(encode_offset_list(&[]).unwrap().is_empty());
    }

    #[test]
    fn encodes_offset_list_two() {
        let encoded = encode_offset_list(&[vec![1, 2], vec![3]]).unwrap();
        // offsets at 8 and 10
        assert_eq!(&encoded[..4], &8u32.to_le_bytes());
        assert_eq!(&encoded[4..8], &10u32.to_le_bytes());
        assert_eq!(&encoded[8..], &[1, 2, 3]);
    }

    #[test]
    fn encodes_bitlist_empty() {
        // only delimiter bit → [0x01]
        assert_eq!(encode_bitlist(&[]), vec![0x01]);
    }
}
