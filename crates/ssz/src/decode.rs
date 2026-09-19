//! Strict SSZ decoding with bounds checks.

use crate::error::SszError;

/// Decode a little-endian `u8` and advance `cursor`.
pub fn decode_u8(input: &[u8], cursor: &mut usize) -> Result<u8, SszError> {
    need(input, *cursor, 1)?;
    let v = input[*cursor];
    *cursor += 1;
    Ok(v)
}

/// Decode a little-endian `u16`.
pub fn decode_u16(input: &[u8], cursor: &mut usize) -> Result<u16, SszError> {
    need(input, *cursor, 2)?;
    let mut buf = [0u8; 2];
    buf.copy_from_slice(&input[*cursor..*cursor + 2]);
    *cursor += 2;
    Ok(u16::from_le_bytes(buf))
}

/// Decode a little-endian `u32`.
pub fn decode_u32(input: &[u8], cursor: &mut usize) -> Result<u32, SszError> {
    need(input, *cursor, 4)?;
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&input[*cursor..*cursor + 4]);
    *cursor += 4;
    Ok(u32::from_le_bytes(buf))
}

/// Decode a little-endian `u64`.
pub fn decode_u64(input: &[u8], cursor: &mut usize) -> Result<u64, SszError> {
    need(input, *cursor, 8)?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&input[*cursor..*cursor + 8]);
    *cursor += 8;
    Ok(u64::from_le_bytes(buf))
}

/// Decode a boolean (`0x00` / `0x01` only).
pub fn decode_bool(input: &[u8], cursor: &mut usize) -> Result<bool, SszError> {
    let b = decode_u8(input, cursor)?;
    match b {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(SszError::InvalidBool(other)),
    }
}

/// Decode exactly `len` fixed bytes into `out`.
pub fn decode_fixed_bytes(input: &[u8], cursor: &mut usize, out: &mut [u8]) -> Result<(), SszError> {
    need(input, *cursor, out.len())?;
    out.copy_from_slice(&input[*cursor..*cursor + out.len()]);
    *cursor += out.len();
    Ok(())
}

/// Decode an offset list of variable elements; each slice is a contiguous payload.
pub fn decode_offset_list<'a>(
    input: &'a [u8],
    limit: usize,
) -> Result<Vec<&'a [u8]>, SszError> {
    if input.is_empty() {
        return Ok(Vec::new());
    }
    if input.len() < 4 {
        return Err(SszError::BufferTooShort {
            need: 4,
            have: input.len(),
        });
    }
    let first = u32::from_le_bytes(input[0..4].try_into().unwrap()) as usize;
    if first % 4 != 0 || first == 0 {
        return Err(SszError::OffsetBeforeFixed {
            offset: first,
            fixed_end: 4,
        });
    }
    let count = first / 4;
    if count > limit {
        return Err(SszError::ListTooLong {
            got: count,
            limit,
        });
    }
    let mut offsets = Vec::with_capacity(count);
    for i in 0..count {
        let start = i * 4;
        need(input, start, 4)?;
        let off = u32::from_le_bytes(input[start..start + 4].try_into().unwrap()) as usize;
        offsets.push(off);
    }
    if offsets[0] != first {
        return Err(SszError::OffsetBeforeFixed {
            offset: offsets[0],
            fixed_end: first,
        });
    }
    for w in offsets.windows(2) {
        if w[1] < w[0] {
            return Err(SszError::OffsetsNotIncreasing);
        }
        if w[1] > input.len() {
            return Err(SszError::OffsetOutOfRange {
                offset: w[1],
                payload_len: input.len(),
            });
        }
    }
    if *offsets.last().unwrap() > input.len() {
        return Err(SszError::OffsetOutOfRange {
            offset: *offsets.last().unwrap(),
            payload_len: input.len(),
        });
    }
    let mut parts = Vec::with_capacity(count);
    for i in 0..count {
        let start = offsets[i];
        let end = if i + 1 < count {
            offsets[i + 1]
        } else {
            input.len()
        };
        if start > end || end > input.len() {
            return Err(SszError::OffsetOutOfRange {
                offset: start,
                payload_len: input.len(),
            });
        }
        parts.push(&input[start..end]);
    }
    Ok(parts)
}

/// Decode a bitlist; `limit` is the maximum number of data bits (excluding delimiter).
pub fn decode_bitlist(input: &[u8], limit: usize) -> Result<Vec<bool>, SszError> {
    if input.is_empty() {
        return Err(SszError::InvalidBitlist);
    }
    let last = *input.last().unwrap();
    if last == 0 {
        return Err(SszError::InvalidBitlist);
    }
    // Highest set bit in the last byte is the delimiter (inclusive length).
    let msb = 7 - last.leading_zeros() as usize;
    let total_bits = (input.len() - 1) * 8 + msb + 1;
    let data_bits = total_bits - 1;
    if data_bits > limit {
        return Err(SszError::ListTooLong {
            got: data_bits,
            limit,
        });
    }
    let mut bits = Vec::with_capacity(data_bits);
    for i in 0..data_bits {
        bits.push((input[i / 8] >> (i % 8)) & 1 == 1);
    }
    Ok(bits)
}

/// Require `need` bytes available from `cursor`.
pub fn need(input: &[u8], cursor: usize, need: usize) -> Result<(), SszError> {
    let end = cursor.checked_add(need).ok_or(SszError::BufferTooShort {
        need,
        have: input.len().saturating_sub(cursor),
    })?;
    if end > input.len() {
        return Err(SszError::BufferTooShort {
            need,
            have: input.len().saturating_sub(cursor),
        });
    }
    Ok(())
}

/// Fail if `cursor` has not consumed the whole buffer.
pub fn expect_exhausted(input: &[u8], cursor: usize) -> Result<(), SszError> {
    if cursor != input.len() {
        return Err(SszError::TrailingBytes(input.len() - cursor));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::{encode_bitlist, encode_offset_list, encode_u64};

    #[test]
    fn roundtrip_u64() {
        let mut buf = Vec::new();
        encode_u64(&mut buf, 42);
        let mut c = 0;
        assert_eq!(decode_u64(&buf, &mut c).unwrap(), 42);
        expect_exhausted(&buf, c).unwrap();
    }

    #[test]
    fn roundtrip_offset_list() {
        let enc = encode_offset_list(&[vec![9], vec![8, 7]]).unwrap();
        let parts = decode_offset_list(&enc, 8).unwrap();
        assert_eq!(parts, vec![&[9][..], &[8, 7][..]]);
    }

    #[test]
    fn roundtrip_bitlist() {
        for bits in [
            vec![],
            vec![true],
            vec![true, true],
            vec![true, false, true],
            vec![false, true, true, false, true],
        ] {
            let enc = encode_bitlist(&bits);
            assert_eq!(decode_bitlist(&enc, 64).unwrap(), bits);
        }
    }

    #[test]
    fn rejects_oversized_bitlist() {
        let enc = encode_bitlist(&[true; 5]);
        assert!(matches!(
            decode_bitlist(&enc, 4),
            Err(SszError::ListTooLong { .. })
        ));
    }
}
