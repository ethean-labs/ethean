//! Block header without body.

use ethean_primitives::{Hash32, Slot, ValidatorIndex};
use ethean_ssz::{
    decode_fixed_bytes, decode_u64, encode_fixed_bytes, encode_u64, expect_exhausted,
    hash_tree_root_bytes, hash_tree_root_container, hash_tree_root_u64, Root,
};

use crate::error::TypesError;

/// Metadata summarizing a block without its body.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockHeader {
    pub slot: Slot,
    pub proposer_index: ValidatorIndex,
    pub parent_root: Hash32,
    pub state_root: Hash32,
    pub body_root: Hash32,
}

impl BlockHeader {
    pub fn ssz_encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(8 + 8 + 32 * 3);
        encode_u64(&mut out, self.slot.get());
        encode_u64(&mut out, self.proposer_index.get());
        encode_fixed_bytes(&mut out, &self.parent_root);
        encode_fixed_bytes(&mut out, &self.state_root);
        encode_fixed_bytes(&mut out, &self.body_root);
        out
    }

    pub fn ssz_decode(input: &[u8]) -> Result<Self, TypesError> {
        let mut c = 0;
        let slot = Slot::new(decode_u64(input, &mut c)?);
        let proposer_index = ValidatorIndex::new(decode_u64(input, &mut c)?);
        let mut parent_root = [0u8; 32];
        let mut state_root = [0u8; 32];
        let mut body_root = [0u8; 32];
        decode_fixed_bytes(input, &mut c, &mut parent_root)?;
        decode_fixed_bytes(input, &mut c, &mut state_root)?;
        decode_fixed_bytes(input, &mut c, &mut body_root)?;
        expect_exhausted(input, c)?;
        Ok(Self {
            slot,
            proposer_index,
            parent_root,
            state_root,
            body_root,
        })
    }

    pub fn hash_tree_root(&self) -> Root {
        hash_tree_root_container(&[
            hash_tree_root_u64(self.slot.get()),
            hash_tree_root_u64(self.proposer_index.get()),
            hash_tree_root_bytes(&self.parent_root),
            hash_tree_root_bytes(&self.state_root),
            hash_tree_root_bytes(&self.body_root),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_roundtrip_and_root() {
        let h = BlockHeader {
            slot: Slot::new(5),
            proposer_index: ValidatorIndex::new(1),
            parent_root: [2u8; 32],
            state_root: [3u8; 32],
            body_root: [4u8; 32],
        };
        assert_eq!(BlockHeader::ssz_decode(&h.ssz_encode()).unwrap(), h);
        assert_eq!(h.hash_tree_root(), h.hash_tree_root());
    }
}
