//! Signed block envelope with multi-message aggregate proof.

use ethean_ssz::{decode_offset_list, encode_u32, hash_tree_root_container, Root};

use crate::aggregate::MultiMessageAggregate;
use crate::error::TypesError;

use super::body::Block;

/// Block plus Type-2 multi-message proof (attestations + proposer).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SignedBlock {
    pub block: Block,
    pub proof: MultiMessageAggregate,
}

impl SignedBlock {
    pub fn new(block: Block, proof: MultiMessageAggregate) -> Self {
        Self { block, proof }
    }

    pub fn ssz_encode(&self) -> Result<Vec<u8>, TypesError> {
        let block = self.block.ssz_encode()?;
        let proof = self.proof.ssz_encode();
        // block variable, proof variable → two offsets
        let fixed_end = 8;
        let mut out = Vec::with_capacity(fixed_end + block.len() + proof.len());
        encode_u32(&mut out, fixed_end as u32);
        encode_u32(&mut out, (fixed_end + block.len()) as u32);
        out.extend_from_slice(&block);
        out.extend_from_slice(&proof);
        Ok(out)
    }

    pub fn ssz_decode(input: &[u8]) -> Result<Self, TypesError> {
        let parts = decode_offset_list(input, 2)?;
        if parts.len() != 2 {
            return Err(TypesError::InvalidByteLength {
                got: parts.len(),
                expected: 2,
            });
        }
        Ok(Self {
            block: Block::ssz_decode(parts[0])?,
            proof: MultiMessageAggregate::ssz_decode(parts[1])?,
        })
    }

    pub fn hash_tree_root(&self) -> Result<Root, TypesError> {
        Ok(hash_tree_root_container(&[
            self.block.hash_tree_root()?,
            self.proof.hash_tree_root()?,
        ]))
    }
}
