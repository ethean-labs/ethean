//! List / container variable-section SSZ decode helpers.

use crate::decode::need;
use crate::error::SszError;

/// Decode packed `Hash32` / `[u8; 32]` list (no per-element offsets).
pub fn decode_hash32_list(input: &[u8], limit: usize) -> Result<Vec<[u8; 32]>, SszError> {
    if input.len() % 32 != 0 {
        return Err(SszError::InvalidFixedVector {
            got: input.len(),
            element: 32,
        });
    }
    let count = input.len() / 32;
    if count > limit {
        return Err(SszError::ListTooLong { got: count, limit });
    }
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let start = i * 32;
        let mut root = [0u8; 32];
        root.copy_from_slice(&input[start..start + 32]);
        out.push(root);
    }
    Ok(out)
}

/// Decode `field_count` variable-size fields after a fixed section of a container.
///
/// Offsets are absolute from the start of `input` (SSZ container layout).
pub fn decode_container_offsets<'a>(
    input: &'a [u8],
    fixed_end: usize,
    field_count: usize,
) -> Result<Vec<&'a [u8]>, SszError> {
    if field_count == 0 {
        return Ok(Vec::new());
    }
    let offsets_len = field_count
        .checked_mul(4)
        .ok_or(SszError::ListTooLong {
            got: field_count,
            limit: usize::MAX / 4,
        })?;
    need(input, fixed_end, offsets_len)?;
    let expected_first = fixed_end + offsets_len;
    let mut offsets = Vec::with_capacity(field_count);
    for i in 0..field_count {
        let at = fixed_end + i * 4;
        let off = u32::from_le_bytes(input[at..at + 4].try_into().unwrap()) as usize;
        offsets.push(off);
    }
    if offsets[0] != expected_first {
        return Err(SszError::OffsetBeforeFixed {
            offset: offsets[0],
            fixed_end: expected_first,
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
    let mut parts = Vec::with_capacity(field_count);
    for i in 0..field_count {
        let start = offsets[i];
        let end = if i + 1 < field_count {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::encode_u32;

    #[test]
    fn roundtrip_hash32_list() {
        let a = [1u8; 32];
        let b = [2u8; 32];
        let mut raw = Vec::new();
        raw.extend_from_slice(&a);
        raw.extend_from_slice(&b);
        assert_eq!(decode_hash32_list(&raw, 8).unwrap(), vec![a, b]);
        assert!(decode_hash32_list(&raw[..31], 8).is_err());
    }

    #[test]
    fn container_offsets_allow_empty_parts() {
        let mut buf = vec![0u8; 4];
        let fixed_end = 4;
        let first = (fixed_end + 8) as u32;
        encode_u32(&mut buf, first);
        encode_u32(&mut buf, first);
        let parts = decode_container_offsets(&buf, fixed_end, 2).unwrap();
        assert_eq!(parts.len(), 2);
        assert!(parts[0].is_empty());
        assert!(parts[1].is_empty());
    }
}
