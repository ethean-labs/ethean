//! Status handshake payload (SSZ-free fixed layout for Phase 10 scaffolding).

use ethean_primitives::Hash32;

use crate::error::{Result, WireError};

/// Compact status exchanged at connection (genesis + head + finalized).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    /// Genesis root.
    pub genesis_root: Hash32,
    /// Fork segment string bytes (UTF-8), max 16.
    pub fork_segment: String,
    /// Claimed head slot.
    pub head_slot: u64,
    /// Claimed head root.
    pub head_root: Hash32,
    /// Claimed finalized slot.
    pub finalized_slot: u64,
    /// Claimed finalized root.
    pub finalized_root: Hash32,
}

impl Status {
    /// Encode as: genesis(32) || fork_len(u8) || fork || head_slot(u64 LE) || head(32)
    /// || finalized_slot(u64 LE) || finalized(32).
    pub fn encode(&self) -> Result<Vec<u8>> {
        if self.fork_segment.len() > 16 || self.fork_segment.is_empty() {
            return Err(WireError::InvalidStatus("fork segment length".into()));
        }
        let mut out = Vec::with_capacity(32 + 1 + self.fork_segment.len() + 8 + 32 + 8 + 32);
        out.extend_from_slice(&self.genesis_root);
        out.push(self.fork_segment.len() as u8);
        out.extend_from_slice(self.fork_segment.as_bytes());
        out.extend_from_slice(&self.head_slot.to_le_bytes());
        out.extend_from_slice(&self.head_root);
        out.extend_from_slice(&self.finalized_slot.to_le_bytes());
        out.extend_from_slice(&self.finalized_root);
        Ok(out)
    }

    /// Decode Status; rejects trailing bytes.
    pub fn decode(input: &[u8]) -> Result<Self> {
        if input.len() < 32 + 1 {
            return Err(WireError::InvalidStatus("too short".into()));
        }
        let mut c = 0;
        let mut genesis_root = [0u8; 32];
        genesis_root.copy_from_slice(&input[c..c + 32]);
        c += 32;
        let flen = input[c] as usize;
        c += 1;
        if flen == 0 || flen > 16 || c + flen > input.len() {
            return Err(WireError::InvalidStatus("fork length".into()));
        }
        let fork_segment = std::str::from_utf8(&input[c..c + flen])
            .map_err(|_| WireError::InvalidStatus("fork utf8".into()))?
            .to_string();
        c += flen;
        if input.len() - c != 8 + 32 + 8 + 32 {
            return Err(WireError::TrailingBytes);
        }
        let head_slot = u64::from_le_bytes(input[c..c + 8].try_into().unwrap());
        c += 8;
        let mut head_root = [0u8; 32];
        head_root.copy_from_slice(&input[c..c + 32]);
        c += 32;
        let finalized_slot = u64::from_le_bytes(input[c..c + 8].try_into().unwrap());
        c += 8;
        let mut finalized_root = [0u8; 32];
        finalized_root.copy_from_slice(&input[c..c + 32]);
        Ok(Self {
            genesis_root,
            fork_segment,
            head_slot,
            head_root,
            finalized_slot,
            finalized_root,
        })
    }

    /// Reject mismatched genesis or fork before trusting peer head claims.
    pub fn compatible_with(&self, local: &Status) -> Result<()> {
        if self.genesis_root != local.genesis_root {
            return Err(WireError::InvalidStatus("genesis mismatch".into()));
        }
        if self.fork_segment != local.fork_segment {
            return Err(WireError::InvalidStatus("fork mismatch".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_compat() {
        let s = Status {
            genesis_root: [1u8; 32],
            fork_segment: "abcd1234".into(),
            head_slot: 9,
            head_root: [2u8; 32],
            finalized_slot: 3,
            finalized_root: [3u8; 32],
        };
        let enc = s.encode().unwrap();
        let dec = Status::decode(&enc).unwrap();
        assert_eq!(dec, s);
        assert!(s.compatible_with(&dec).is_ok());
    }
}
