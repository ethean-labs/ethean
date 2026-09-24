//! Blocks-by-root / range request shapes and response codes.

use ethean_primitives::Hash32;

use crate::error::{Result, WireError};
use crate::limits::MAX_BLOCKS_PER_REQUEST;

/// Blocks-by-root request: SSZ `List[Bytes32, 1024]` (4-byte offset then roots).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlocksByRootRequest {
    /// Requested block roots in preference order.
    pub roots: Vec<Hash32>,
}

impl BlocksByRootRequest {
    /// Construct after validating count (empty list is valid SSZ).
    pub fn new(roots: Vec<Hash32>) -> Result<Self> {
        if roots.len() as u64 > MAX_BLOCKS_PER_REQUEST {
            return Err(WireError::InvalidReqResp(format!(
                "roots {} exceeds {}",
                roots.len(),
                MAX_BLOCKS_PER_REQUEST
            )));
        }
        Ok(Self { roots })
    }

    /// SSZ encode: offset `4` then packed 32-byte roots.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(4 + self.roots.len() * 32);
        out.extend_from_slice(&4u32.to_le_bytes());
        for root in &self.roots {
            out.extend_from_slice(root);
        }
        out
    }

    /// SSZ decode a `List[Bytes32, 1024]` container.
    pub fn decode(input: &[u8]) -> Result<Self> {
        if input.len() < 4 {
            return Err(WireError::InvalidReqResp(
                "blocks-by-root request too short".into(),
            ));
        }
        let offset = u32::from_le_bytes(input[0..4].try_into().unwrap()) as usize;
        if offset != 4 || offset > input.len() {
            return Err(WireError::InvalidReqResp(
                "blocks-by-root offset".into(),
            ));
        }
        let rest = &input[offset..];
        if rest.len() % 32 != 0 {
            return Err(WireError::InvalidReqResp(
                "blocks-by-root roots not multiple of 32".into(),
            ));
        }
        let mut roots = Vec::with_capacity(rest.len() / 32);
        for chunk in rest.chunks_exact(32) {
            let mut root = [0u8; 32];
            root.copy_from_slice(chunk);
            roots.push(root);
        }
        Self::new(roots)
    }
}

/// Blocks-by-range request: start_slot + count (16 bytes, no step).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlocksByRangeRequest {
    /// First slot (inclusive).
    pub start_slot: u64,
    /// Number of slots to cover (step is always 1).
    pub count: u64,
}

impl BlocksByRangeRequest {
    /// Validate count against the pinned maximum. Zero is invalid.
    pub fn new(start_slot: u64, count: u64) -> Result<Self> {
        if count == 0 || count > MAX_BLOCKS_PER_REQUEST {
            return Err(WireError::InvalidReqResp(format!(
                "count {count} invalid (max {MAX_BLOCKS_PER_REQUEST})"
            )));
        }
        Ok(Self { start_slot, count })
    }

    /// Encode 16-byte SSZ container.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(16);
        out.extend_from_slice(&self.start_slot.to_le_bytes());
        out.extend_from_slice(&self.count.to_le_bytes());
        out
    }

    /// Decode 16-byte SSZ container (no step field).
    pub fn decode(input: &[u8]) -> Result<Self> {
        if input.len() != 16 {
            return Err(WireError::InvalidReqResp(format!(
                "blocks-by-range length {} != 16",
                input.len()
            )));
        }
        let start = u64::from_le_bytes(input[0..8].try_into().unwrap());
        let count = u64::from_le_bytes(input[8..16].try_into().unwrap());
        Self::new(start, count)
    }

    /// Enumerate requested slots (step 1).
    pub fn slots(&self) -> Vec<u64> {
        (0..self.count)
            .map(|i| self.start_slot.saturating_add(i))
            .collect()
    }
}

/// Response code for a single req/resp chunk (leanSpec / Ethereum P2P).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ResponseCode {
    /// Success chunk follows.
    Success = 0,
    /// Invalid request.
    InvalidRequest = 1,
    /// Server error.
    ServerError = 2,
    /// Resource unavailable / missing.
    ResourceUnavailable = 3,
}

impl ResponseCode {
    /// Parse a raw code byte, mapping unknown 4-127 to ServerError and 128-255 to InvalidRequest.
    pub fn from_wire(byte: u8) -> Self {
        match byte {
            0 => Self::Success,
            1 => Self::InvalidRequest,
            2 => Self::ServerError,
            3 => Self::ResourceUnavailable,
            4..=127 => Self::ServerError,
            _ => Self::InvalidRequest,
        }
    }

    /// Wire byte.
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unhex(s: &str) -> Vec<u8> {
        let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn blocks_by_root_ssz_vectors() {
        let empty = BlocksByRootRequest::new(vec![]).unwrap();
        assert_eq!(empty.encode(), unhex("04000000"));
        assert_eq!(BlocksByRootRequest::decode(&empty.encode()).unwrap(), empty);

        let two = BlocksByRootRequest::new(vec![[0xaa; 32], [0xbb; 32]]).unwrap();
        let expected = unhex(concat!(
            "04000000",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        ));
        assert_eq!(two.encode(), expected);
        assert_eq!(BlocksByRootRequest::decode(&expected).unwrap(), two);
    }

    #[test]
    fn range_is_16_bytes_no_step() {
        let r = BlocksByRangeRequest::new(10, 3).unwrap();
        assert_eq!(r.slots(), vec![10, 11, 12]);
        let enc = r.encode();
        assert_eq!(enc.len(), 16);
        assert_eq!(BlocksByRangeRequest::decode(&enc).unwrap(), r);
        assert!(BlocksByRangeRequest::decode(&[0u8; 24]).is_err());
    }

    #[test]
    fn response_codes_match_spec() {
        assert_eq!(ResponseCode::Success.as_u8(), 0);
        assert_eq!(ResponseCode::InvalidRequest.as_u8(), 1);
        assert_eq!(ResponseCode::ServerError.as_u8(), 2);
        assert_eq!(ResponseCode::ResourceUnavailable.as_u8(), 3);
        assert_eq!(ResponseCode::from_wire(4), ResponseCode::ServerError);
        assert_eq!(ResponseCode::from_wire(200), ResponseCode::InvalidRequest);
    }

    #[test]
    fn rejects_too_many_roots() {
        let roots = vec![[0u8; 32]; 1025];
        assert!(BlocksByRootRequest::new(roots).is_err());
    }
}
