//! Block body and full block containers.

use ethean_primitives::{Hash32, Slot, ValidatorIndex};
use ethean_ssz::{
    decode_fixed_bytes, decode_offset_list, decode_u32, decode_u64, encode_fixed_bytes,
    encode_offset_list, encode_u32, encode_u64, hash_tree_root_bytes, hash_tree_root_container,
    hash_tree_root_list, hash_tree_root_u64, Root,
};

use crate::error::TypesError;
use crate::limits::{AGGREGATED_ATTESTATIONS_LIMIT, MAX_ATTESTATIONS_DATA};
use crate::operation::AggregatedAttestation;

use super::header::BlockHeader;

/// Block payload carrying aggregated attestations.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockBody {
    pub attestations: Vec<AggregatedAttestation>,
}

impl BlockBody {
    pub fn new(attestations: Vec<AggregatedAttestation>) -> Result<Self, TypesError> {
        if attestations.len() > AGGREGATED_ATTESTATIONS_LIMIT {
            return Err(TypesError::ListTooLong {
                got: attestations.len(),
                limit: AGGREGATED_ATTESTATIONS_LIMIT,
            });
        }
        Ok(Self { attestations })
    }

    /// Consensus cap on distinct attestation data (not the SSZ list limit).
    pub fn exceeds_max_attestations_data(&self) -> bool {
        self.attestations.len() > MAX_ATTESTATIONS_DATA
    }

    pub fn ssz_encode(&self) -> Result<Vec<u8>, TypesError> {
        let mut elements = Vec::with_capacity(self.attestations.len());
        for a in &self.attestations {
            elements.push(a.ssz_encode());
        }
        Ok(encode_offset_list(&elements)?)
    }

    pub fn ssz_decode(input: &[u8]) -> Result<Self, TypesError> {
        let parts = decode_offset_list(input, AGGREGATED_ATTESTATIONS_LIMIT)?;
        let mut attestations = Vec::with_capacity(parts.len());
        for p in parts {
            attestations.push(AggregatedAttestation::ssz_decode(p)?);
        }
        Self::new(attestations)
    }

    pub fn hash_tree_root(&self) -> Result<Root, TypesError> {
        let mut roots = Vec::with_capacity(self.attestations.len());
        for a in &self.attestations {
            roots.push(a.hash_tree_root()?);
        }
        Ok(hash_tree_root_list(
            &roots,
            AGGREGATED_ATTESTATIONS_LIMIT,
        )?)
    }
}

/// Complete block including header fields and body.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Block {
    pub slot: Slot,
    pub proposer_index: ValidatorIndex,
    pub parent_root: Hash32,
    pub state_root: Hash32,
    pub body: BlockBody,
}

impl Block {
    pub fn header(&self) -> Result<BlockHeader, TypesError> {
        Ok(BlockHeader {
            slot: self.slot,
            proposer_index: self.proposer_index,
            parent_root: self.parent_root,
            state_root: self.state_root,
            body_root: self.body.hash_tree_root()?,
        })
    }

    pub fn ssz_encode(&self) -> Result<Vec<u8>, TypesError> {
        let body = self.body.ssz_encode()?;
        let mut out = Vec::with_capacity(8 + 8 + 32 + 32 + 4 + body.len());
        encode_u64(&mut out, self.slot.get());
        encode_u64(&mut out, self.proposer_index.get());
        encode_fixed_bytes(&mut out, &self.parent_root);
        encode_fixed_bytes(&mut out, &self.state_root);
        let offset = (out.len() + 4) as u32;
        encode_u32(&mut out, offset);
        out.extend_from_slice(&body);
        Ok(out)
    }

    pub fn ssz_decode(input: &[u8]) -> Result<Self, TypesError> {
        let mut c = 0;
        let slot = Slot::new(decode_u64(input, &mut c)?);
        let proposer_index = ValidatorIndex::new(decode_u64(input, &mut c)?);
        let mut parent_root = [0u8; 32];
        let mut state_root = [0u8; 32];
        decode_fixed_bytes(input, &mut c, &mut parent_root)?;
        decode_fixed_bytes(input, &mut c, &mut state_root)?;
        let off = decode_u32(input, &mut c)? as usize;
        if off != c || off > input.len() {
            return Err(TypesError::Ssz(ethean_ssz::SszError::OffsetOutOfRange {
                offset: off,
                payload_len: input.len(),
            }));
        }
        let body = BlockBody::ssz_decode(&input[off..])?;
        Ok(Self {
            slot,
            proposer_index,
            parent_root,
            state_root,
            body,
        })
    }

    pub fn hash_tree_root(&self) -> Result<Root, TypesError> {
        Ok(hash_tree_root_container(&[
            hash_tree_root_u64(self.slot.get()),
            hash_tree_root_u64(self.proposer_index.get()),
            hash_tree_root_bytes(&self.parent_root),
            hash_tree_root_bytes(&self.state_root),
            self.body.hash_tree_root()?,
        ]))
    }
}
