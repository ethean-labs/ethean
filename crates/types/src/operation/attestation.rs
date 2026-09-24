//! Attestation envelopes (Lean lstar).

use ethean_primitives::ValidatorIndex;
use ethean_ssz::{
    decode_fixed_bytes, decode_u64, encode_fixed_bytes, encode_u64, expect_exhausted,
    hash_tree_root_container, hash_tree_root_u64, Root,
};

use crate::aggregate::{AggregationBits, SingleMessageAggregate};
use crate::checkpoint::AttestationData;
use crate::error::TypesError;
use crate::limits::XMSS_SIGNATURE_BYTES;

const ATTESTATION_DATA_SSZ_LEN: usize = 8 + 40 * 3;

/// Validator-specific attestation wrapping shared data.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Attestation {
    pub validator_index: ValidatorIndex,
    pub data: AttestationData,
}

impl Attestation {
    pub fn ssz_encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(8 + ATTESTATION_DATA_SSZ_LEN);
        encode_u64(&mut out, self.validator_index.get());
        out.extend_from_slice(&self.data.ssz_encode());
        out
    }

    pub fn ssz_decode(input: &[u8]) -> Result<Self, TypesError> {
        let mut c = 0;
        let validator_index = ValidatorIndex::new(decode_u64(input, &mut c)?);
        let data = AttestationData::ssz_decode(&input[c..])?;
        Ok(Self {
            validator_index,
            data,
        })
    }

    pub fn hash_tree_root(&self) -> Root {
        hash_tree_root_container(&[
            hash_tree_root_u64(self.validator_index.get()),
            self.data.hash_tree_root(),
        ])
    }
}

/// Attestation plus XMSS signature (exact 2536 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedAttestation {
    pub validator_index: ValidatorIndex,
    pub data: AttestationData,
    pub signature: Vec<u8>,
}

impl SignedAttestation {
    pub fn new(
        validator_index: ValidatorIndex,
        data: AttestationData,
        signature: Vec<u8>,
    ) -> Result<Self, TypesError> {
        if signature.len() != XMSS_SIGNATURE_BYTES {
            return Err(TypesError::InvalidByteLength {
                got: signature.len(),
                expected: XMSS_SIGNATURE_BYTES,
            });
        }
        Ok(Self {
            validator_index,
            data,
            signature,
        })
    }

    pub fn ssz_encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(8 + ATTESTATION_DATA_SSZ_LEN + XMSS_SIGNATURE_BYTES);
        encode_u64(&mut out, self.validator_index.get());
        out.extend_from_slice(&self.data.ssz_encode());
        encode_fixed_bytes(&mut out, &self.signature);
        out
    }

    pub fn ssz_decode(input: &[u8]) -> Result<Self, TypesError> {
        let mut c = 0;
        let validator_index = ValidatorIndex::new(decode_u64(input, &mut c)?);
        let data_end = c + ATTESTATION_DATA_SSZ_LEN;
        if input.len() < data_end + XMSS_SIGNATURE_BYTES {
            return Err(TypesError::Ssz(ethean_ssz::SszError::BufferTooShort {
                need: data_end + XMSS_SIGNATURE_BYTES,
                have: input.len(),
            }));
        }
        let data = AttestationData::ssz_decode(&input[c..data_end])?;
        c = data_end;
        let mut sig = vec![0u8; XMSS_SIGNATURE_BYTES];
        decode_fixed_bytes(input, &mut c, &mut sig)?;
        expect_exhausted(input, c)?;
        Self::new(validator_index, data, sig)
    }

    /// Root with the signature hashed as the leanSpec `Signature` container.
    pub fn hash_tree_root(&self) -> Result<Root, TypesError> {
        Ok(hash_tree_root_container(&[
            hash_tree_root_u64(self.validator_index.get()),
            self.data.hash_tree_root(),
            crate::xmss_root::xmss_signature_root(&self.signature)?,
        ]))
    }
}

/// Aggregated attestation: participation bits + shared data.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AggregatedAttestation {
    pub aggregation_bits: AggregationBits,
    pub data: AttestationData,
}

impl AggregatedAttestation {
    pub fn ssz_encode(&self) -> Vec<u8> {
        let bits = self.aggregation_bits.ssz_encode();
        let data = self.data.ssz_encode();
        let fixed_end = 4 + data.len();
        let mut out = Vec::with_capacity(fixed_end + bits.len());
        out.extend_from_slice(&(fixed_end as u32).to_le_bytes());
        out.extend_from_slice(&data);
        out.extend_from_slice(&bits);
        out
    }

    pub fn ssz_decode(input: &[u8]) -> Result<Self, TypesError> {
        if input.len() < 4 {
            return Err(TypesError::Ssz(ethean_ssz::SszError::BufferTooShort {
                need: 4,
                have: input.len(),
            }));
        }
        let offset = u32::from_le_bytes(input[0..4].try_into().unwrap()) as usize;
        if offset > input.len() || offset < 4 {
            return Err(TypesError::Ssz(ethean_ssz::SszError::OffsetOutOfRange {
                offset,
                payload_len: input.len(),
            }));
        }
        let data = AttestationData::ssz_decode(&input[4..offset])?;
        let aggregation_bits = AggregationBits::ssz_decode(&input[offset..])?;
        Ok(Self {
            aggregation_bits,
            data,
        })
    }

    pub fn hash_tree_root(&self) -> Result<Root, TypesError> {
        Ok(hash_tree_root_container(&[
            self.aggregation_bits.hash_tree_root()?,
            self.data.hash_tree_root(),
        ]))
    }
}

/// Signed aggregated attestation for gossip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedAggregatedAttestation {
    pub data: AttestationData,
    pub proof: SingleMessageAggregate,
}

impl SignedAggregatedAttestation {
    /// SSZ container `{data: AttestationData, proof: SingleMessageAggregate}`:
    /// the fixed 128-byte `data`, then a 4-byte offset, then `proof`.
    pub fn ssz_encode(&self) -> Result<Vec<u8>, TypesError> {
        let data = self.data.ssz_encode();
        let proof = self.proof.ssz_encode()?;
        let fixed_end = data.len() + 4;
        let mut out = Vec::with_capacity(fixed_end + proof.len());
        out.extend_from_slice(&data);
        out.extend_from_slice(&(fixed_end as u32).to_le_bytes());
        out.extend_from_slice(&proof);
        Ok(out)
    }

    pub fn ssz_decode(input: &[u8]) -> Result<Self, TypesError> {
        let fixed_end = ATTESTATION_DATA_SSZ_LEN + 4;
        if input.len() < fixed_end {
            return Err(TypesError::Ssz(ethean_ssz::SszError::BufferTooShort {
                need: fixed_end,
                have: input.len(),
            }));
        }
        let offset = u32::from_le_bytes(
            input[ATTESTATION_DATA_SSZ_LEN..fixed_end]
                .try_into()
                .expect("4-byte offset"),
        ) as usize;
        if offset != fixed_end {
            return Err(TypesError::Ssz(ethean_ssz::SszError::OffsetOutOfRange {
                offset,
                payload_len: input.len(),
            }));
        }
        Ok(Self {
            data: AttestationData::ssz_decode(&input[..ATTESTATION_DATA_SSZ_LEN])?,
            proof: SingleMessageAggregate::ssz_decode(&input[fixed_end..])?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkpoint::Checkpoint;
    use ethean_primitives::Slot;

    #[test]
    fn signed_aggregate_places_fixed_data_before_proof_offset() {
        let data = AttestationData {
            slot: Slot::new(5),
            head: Checkpoint::genesis(),
            target: Checkpoint::genesis(),
            source: Checkpoint::genesis(),
        };
        let proof = SingleMessageAggregate::new(
            AggregationBits::new(vec![true, false, true]).unwrap(),
            vec![0xab; 10],
        )
        .unwrap();
        let signed = SignedAggregatedAttestation {
            data,
            proof: proof.clone(),
        };
        let bytes = signed.ssz_encode().unwrap();
        assert_eq!(&bytes[..ATTESTATION_DATA_SSZ_LEN], &data.ssz_encode()[..]);
        assert_eq!(
            u32::from_le_bytes(bytes[128..132].try_into().unwrap()),
            132,
            "offset to the variable proof field"
        );
        assert_eq!(&bytes[132..], &proof.ssz_encode().unwrap()[..]);
        assert_eq!(
            SignedAggregatedAttestation::ssz_decode(&bytes).unwrap(),
            signed
        );
        let mut bad = bytes.clone();
        bad[128] = 4;
        assert!(SignedAggregatedAttestation::ssz_decode(&bad).is_err());
    }
}
