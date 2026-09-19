//! Validator registry entry (dual XMSS keys).

use ethean_primitives::{Bytes52, ValidatorIndex};
use ethean_ssz::{
    decode_fixed_bytes, decode_u64, encode_fixed_bytes, encode_u64, expect_exhausted,
    hash_tree_root_bytes, hash_tree_root_container, hash_tree_root_u64, Root,
};

use crate::error::TypesError;
use crate::limits::VALIDATOR_REGISTRY_LIMIT;

/// Static validator registry row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Validator {
    pub attestation_public_key: Bytes52,
    pub proposal_public_key: Bytes52,
    pub index: ValidatorIndex,
}

impl Validator {
    pub fn new(
        attestation_public_key: Bytes52,
        proposal_public_key: Bytes52,
        index: ValidatorIndex,
    ) -> Result<Self, TypesError> {
        if index.get() >= VALIDATOR_REGISTRY_LIMIT as u64 {
            return Err(TypesError::ValidatorIndexOutOfRange {
                index: index.get(),
                limit: VALIDATOR_REGISTRY_LIMIT as u64,
            });
        }
        Ok(Self {
            attestation_public_key,
            proposal_public_key,
            index,
        })
    }

    pub fn ssz_encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(52 + 52 + 8);
        encode_fixed_bytes(&mut out, self.attestation_public_key.as_bytes());
        encode_fixed_bytes(&mut out, self.proposal_public_key.as_bytes());
        encode_u64(&mut out, self.index.get());
        out
    }

    pub fn ssz_decode(input: &[u8]) -> Result<Self, TypesError> {
        let mut c = 0;
        let mut att = [0u8; 52];
        let mut prop = [0u8; 52];
        decode_fixed_bytes(input, &mut c, &mut att)?;
        decode_fixed_bytes(input, &mut c, &mut prop)?;
        let index = ValidatorIndex::new(decode_u64(input, &mut c)?);
        expect_exhausted(input, c)?;
        Self::new(Bytes52(att), Bytes52(prop), index)
    }

    pub fn hash_tree_root(&self) -> Root {
        hash_tree_root_container(&[
            hash_tree_root_bytes(self.attestation_public_key.as_bytes()),
            hash_tree_root_bytes(self.proposal_public_key.as_bytes()),
            hash_tree_root_u64(self.index.get()),
        ])
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self {
            attestation_public_key: Bytes52::ZERO,
            proposal_public_key: Bytes52::ZERO,
            index: ValidatorIndex::ZERO,
        }
    }
}
