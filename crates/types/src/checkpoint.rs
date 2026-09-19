//! Casper-FFG checkpoints and attestation vote content (Lean: slot, not epoch).

use ethean_primitives::{Hash32, Slot, HASH32_ZERO};
use ethean_ssz::{
    decode_fixed_bytes, decode_u64, encode_fixed_bytes, encode_u64, expect_exhausted,
    hash_tree_root_bytes, hash_tree_root_container, hash_tree_root_u64, Root,
};

use crate::error::TypesError;

/// A `(block root, slot)` pair that can be justified and finalized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Checkpoint {
    pub root: Hash32,
    pub slot: Slot,
}

impl Checkpoint {
    pub fn new(root: Hash32, slot: Slot) -> Self {
        Self { root, slot }
    }

    pub fn genesis() -> Self {
        Self {
            root: HASH32_ZERO,
            slot: Slot::ZERO,
        }
    }

    /// Later of two checkpoints by slot; keep `self` on a tie.
    pub fn advance_to(self, candidate: Checkpoint) -> Checkpoint {
        if candidate.slot > self.slot {
            candidate
        } else {
            self
        }
    }

    pub fn ssz_encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(40);
        encode_fixed_bytes(&mut out, &self.root);
        encode_u64(&mut out, self.slot.get());
        out
    }

    pub fn ssz_decode(input: &[u8]) -> Result<Self, TypesError> {
        let mut c = 0;
        let mut root = [0u8; 32];
        decode_fixed_bytes(input, &mut c, &mut root)?;
        let slot = Slot::new(decode_u64(input, &mut c)?);
        expect_exhausted(input, c)?;
        Ok(Self { root, slot })
    }

    pub fn hash_tree_root(&self) -> Root {
        hash_tree_root_container(&[
            hash_tree_root_bytes(&self.root),
            hash_tree_root_u64(self.slot.get()),
        ])
    }
}

/// Attestation content: head / target / source checkpoints for a slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AttestationData {
    pub slot: Slot,
    pub head: Checkpoint,
    pub target: Checkpoint,
    pub source: Checkpoint,
}

impl AttestationData {
    pub fn ssz_encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(8 + 40 * 3);
        encode_u64(&mut out, self.slot.get());
        out.extend_from_slice(&self.head.ssz_encode());
        out.extend_from_slice(&self.target.ssz_encode());
        out.extend_from_slice(&self.source.ssz_encode());
        out
    }

    pub fn ssz_decode(input: &[u8]) -> Result<Self, TypesError> {
        let mut c = 0;
        let slot = Slot::new(decode_u64(input, &mut c)?);
        let head = decode_checkpoint_at(input, &mut c)?;
        let target = decode_checkpoint_at(input, &mut c)?;
        let source = decode_checkpoint_at(input, &mut c)?;
        expect_exhausted(input, c)?;
        Ok(Self {
            slot,
            head,
            target,
            source,
        })
    }

    pub fn hash_tree_root(&self) -> Root {
        hash_tree_root_container(&[
            hash_tree_root_u64(self.slot.get()),
            self.head.hash_tree_root(),
            self.target.hash_tree_root(),
            self.source.hash_tree_root(),
        ])
    }
}

fn decode_checkpoint_at(input: &[u8], c: &mut usize) -> Result<Checkpoint, TypesError> {
    let mut root = [0u8; 32];
    decode_fixed_bytes(input, c, &mut root)?;
    let slot = Slot::new(decode_u64(input, c)?);
    Ok(Checkpoint { root, slot })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_roundtrip() {
        let cp = Checkpoint::new([9u8; 32], Slot::new(7));
        let enc = cp.ssz_encode();
        assert_eq!(Checkpoint::ssz_decode(&enc).unwrap(), cp);
    }

    #[test]
    fn checkpoint_root_stable() {
        let cp = Checkpoint::new([1u8; 32], Slot::new(0));
        assert_eq!(cp.hash_tree_root(), cp.hash_tree_root());
    }

    #[test]
    fn attestation_data_roundtrip() {
        let data = AttestationData {
            slot: Slot::new(3),
            head: Checkpoint::new([1u8; 32], Slot::new(3)),
            target: Checkpoint::new([2u8; 32], Slot::new(2)),
            source: Checkpoint::new([3u8; 32], Slot::new(1)),
        };
        let enc = data.ssz_encode();
        assert_eq!(AttestationData::ssz_decode(&enc).unwrap(), data);
        assert_eq!(data.hash_tree_root(), data.hash_tree_root());
    }

    #[test]
    fn advance_to_prefers_higher_slot() {
        let a = Checkpoint::new([0u8; 32], Slot::new(1));
        let b = Checkpoint::new([1u8; 32], Slot::new(2));
        assert_eq!(a.advance_to(b).slot, Slot::new(2));
        assert_eq!(b.advance_to(a).slot, Slot::new(2));
    }
}
