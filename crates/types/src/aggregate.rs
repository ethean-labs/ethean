//! Participation bitlists and aggregate proof stubs.

use ethean_ssz::{
    chunk_from_bytes, decode_bitlist, decode_offset_list, encode_bitlist, encode_byte_list,
    encode_offset_list, hash_tree_root_bitlist, hash_tree_root_container, merkleize, mix_in_length,
    Root,
};

use crate::error::TypesError;
use crate::limits::{BYTE_LIST_512_KIB, VALIDATOR_REGISTRY_LIMIT};

fn next_pow2(n: usize) -> usize {
    if n == 0 {
        1
    } else {
        n.next_power_of_two()
    }
}

/// Validator participation bitfield (`AggregationBits`, limit = registry).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AggregationBits {
    pub bits: Vec<bool>,
}

impl AggregationBits {
    pub fn new(bits: Vec<bool>) -> Result<Self, TypesError> {
        if bits.len() > VALIDATOR_REGISTRY_LIMIT {
            return Err(TypesError::ListTooLong {
                got: bits.len(),
                limit: VALIDATOR_REGISTRY_LIMIT,
            });
        }
        Ok(Self { bits })
    }

    pub fn ssz_encode(&self) -> Vec<u8> {
        encode_bitlist(&self.bits)
    }

    pub fn ssz_decode(input: &[u8]) -> Result<Self, TypesError> {
        let bits = decode_bitlist(input, VALIDATOR_REGISTRY_LIMIT)?;
        Self::new(bits)
    }

    pub fn hash_tree_root(&self) -> Result<Root, TypesError> {
        Ok(hash_tree_root_bitlist(
            &self.bits,
            VALIDATOR_REGISTRY_LIMIT,
        )?)
    }
}

/// Single-message aggregate proof (participants + ≤512 KiB proof bytes).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SingleMessageAggregate {
    pub participants: AggregationBits,
    pub proof: Vec<u8>,
}

impl SingleMessageAggregate {
    pub fn new(participants: AggregationBits, proof: Vec<u8>) -> Result<Self, TypesError> {
        if proof.len() > BYTE_LIST_512_KIB {
            return Err(TypesError::BytesTooLong {
                got: proof.len(),
                max: BYTE_LIST_512_KIB,
            });
        }
        Ok(Self {
            participants,
            proof,
        })
    }

    pub fn ssz_encode(&self) -> Result<Vec<u8>, TypesError> {
        let parts = vec![
            self.participants.ssz_encode(),
            encode_byte_list(&self.proof),
        ];
        Ok(encode_offset_list(&parts)?)
    }

    pub fn ssz_decode(input: &[u8]) -> Result<Self, TypesError> {
        let parts = decode_offset_list(input, 2)?;
        if parts.len() != 2 {
            return Err(TypesError::InvalidByteLength {
                got: parts.len(),
                expected: 2,
            });
        }
        let participants = AggregationBits::ssz_decode(parts[0])?;
        if parts[1].len() > BYTE_LIST_512_KIB {
            return Err(TypesError::BytesTooLong {
                got: parts[1].len(),
                max: BYTE_LIST_512_KIB,
            });
        }
        Self::new(participants, parts[1].to_vec())
    }

    pub fn hash_tree_root(&self) -> Result<Root, TypesError> {
        let proof_root = mix_in_length(
            &merkleize_bytes(&self.proof, BYTE_LIST_512_KIB),
            self.proof.len() as u64,
        );
        Ok(hash_tree_root_container(&[
            self.participants.hash_tree_root()?,
            proof_root,
        ]))
    }
}

/// Multi-message aggregate proof (block envelope stub).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MultiMessageAggregate {
    pub proof: Vec<u8>,
}

impl MultiMessageAggregate {
    pub fn new(proof: Vec<u8>) -> Result<Self, TypesError> {
        if proof.len() > BYTE_LIST_512_KIB {
            return Err(TypesError::BytesTooLong {
                got: proof.len(),
                max: BYTE_LIST_512_KIB,
            });
        }
        Ok(Self { proof })
    }

    pub fn ssz_encode(&self) -> Vec<u8> {
        encode_byte_list(&self.proof)
    }

    pub fn ssz_decode(input: &[u8]) -> Result<Self, TypesError> {
        Self::new(input.to_vec())
    }

    pub fn hash_tree_root(&self) -> Result<Root, TypesError> {
        Ok(mix_in_length(
            &merkleize_bytes(&self.proof, BYTE_LIST_512_KIB),
            self.proof.len() as u64,
        ))
    }
}

fn merkleize_bytes(bytes: &[u8], limit: usize) -> Root {
    let chunk_limit = next_pow2(((limit + 31) / 32).max(1));
    let chunks: Vec<Root> = if bytes.is_empty() {
        Vec::new()
    } else {
        bytes.chunks(32).map(chunk_from_bytes).collect()
    };
    merkleize(&chunks, chunk_limit)
}
