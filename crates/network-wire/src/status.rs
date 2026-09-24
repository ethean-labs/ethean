//! SSZ Status handshake: `Status{finalized: Checkpoint, head: Checkpoint}`.

use ethean_primitives::Hash32;

use crate::error::{Result, WireError};

/// Checkpoint = root(32) ‖ slot(u64 LE). 40 bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Checkpoint {
    /// Block root.
    pub root: Hash32,
    /// Slot of that root.
    pub slot: u64,
}

impl Checkpoint {
    /// Encode 40-byte SSZ checkpoint.
    pub fn encode(&self) -> [u8; 40] {
        let mut out = [0u8; 40];
        out[..32].copy_from_slice(&self.root);
        out[32..].copy_from_slice(&self.slot.to_le_bytes());
        out
    }

    /// Decode 40-byte SSZ checkpoint.
    pub fn decode(input: &[u8]) -> Result<Self> {
        if input.len() != 40 {
            return Err(WireError::InvalidStatus(format!(
                "checkpoint length {} != 40",
                input.len()
            )));
        }
        let mut root = [0u8; 32];
        root.copy_from_slice(&input[..32]);
        let slot = u64::from_le_bytes(input[32..40].try_into().unwrap());
        Ok(Self { root, slot })
    }
}

/// Status exchanged at connection (80-byte SSZ container).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Status {
    /// Latest finalized checkpoint.
    pub finalized: Checkpoint,
    /// Current head checkpoint.
    pub head: Checkpoint,
}

impl Status {
    /// Convenience accessors used by sync / handshake callers.
    pub fn head_slot(&self) -> u64 {
        self.head.slot
    }

    /// Head root.
    pub fn head_root(&self) -> Hash32 {
        self.head.root
    }

    /// Finalized slot.
    pub fn finalized_slot(&self) -> u64 {
        self.finalized.slot
    }

    /// Finalized root.
    pub fn finalized_root(&self) -> Hash32 {
        self.finalized.root
    }

    /// Encode as 80-byte SSZ: finalized then head.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut out = Vec::with_capacity(80);
        out.extend_from_slice(&self.finalized.encode());
        out.extend_from_slice(&self.head.encode());
        Ok(out)
    }

    /// Decode Status; rejects trailing bytes and wrong length.
    pub fn decode(input: &[u8]) -> Result<Self> {
        if input.len() != 80 {
            return Err(WireError::InvalidStatus(format!(
                "status length {} != 80",
                input.len()
            )));
        }
        Ok(Self {
            finalized: Checkpoint::decode(&input[..40])?,
            head: Checkpoint::decode(&input[40..])?,
        })
    }

    /// Status has no genesis/fork fields; compatibility is a no-op.
    pub fn compatible_with(&self, _local: &Status) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// leanSpec `STATUS_SSZ`: finalized=Checkpoint(0x01.., 100), head=Checkpoint(0x02.., 150).
    const STATUS_SSZ: &str = concat!(
        "0101010101010101010101010101010101010101010101010101010101010101",
        "6400000000000000",
        "0202020202020202020202020202020202020202020202020202020202020202",
        "9600000000000000"
    );

    fn unhex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn leanspec_status_ssz_vector() {
        let s = Status {
            finalized: Checkpoint {
                root: [0x01; 32],
                slot: 100,
            },
            head: Checkpoint {
                root: [0x02; 32],
                slot: 150,
            },
        };
        let enc = s.encode().unwrap();
        assert_eq!(enc.len(), 80);
        assert_eq!(enc, unhex(STATUS_SSZ));
        assert_eq!(Status::decode(&enc).unwrap(), s);
        assert_eq!(s.head_slot(), 150);
        assert_eq!(s.finalized_slot(), 100);
    }

    #[test]
    fn rejects_wrong_length() {
        assert!(Status::decode(&[0u8; 79]).is_err());
        assert!(Status::decode(&[0u8; 81]).is_err());
    }
}
