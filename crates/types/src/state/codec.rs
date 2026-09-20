//! SSZ encode/decode for [`State`] (leanSpec / peer field order).

use ethean_primitives::{Hash32, Slot};
use ethean_ssz::{
    decode_bitlist, decode_container_offsets, decode_hash32_list, decode_offset_list, decode_u64,
    encode_bitlist, encode_fixed_bytes, encode_offset_list, encode_u32, encode_u64, need,
};

use crate::block::BlockHeader;
use crate::checkpoint::Checkpoint;
use crate::error::TypesError;
use crate::genesis::GenesisConfig;
use crate::limits::{
    HISTORICAL_ROOTS_LIMIT, JUSTIFICATION_VALIDATORS_LIMIT, VALIDATOR_REGISTRY_LIMIT,
};
use crate::state::State;
use crate::validator::Validator;

/// Fixed prefix size matching ethlambda / Ream Lean `State` layout:
/// config(8) + slot(8) + header(112) + justified(40) + finalized(40).
const STATE_FIXED_END: usize = 8 + 8 + 112 + 40 + 40;
const STATE_VAR_FIELDS: usize = 5;
const HEADER_BYTES: usize = 112;
const CHECKPOINT_BYTES: usize = 40;

impl State {
    /// Encode state (variable lists after fixed fields + offsets).
    pub fn ssz_encode(&self) -> Result<Vec<u8>, TypesError> {
        self.validate_bounds()?;
        let mut fixed = Vec::with_capacity(STATE_FIXED_END + STATE_VAR_FIELDS * 4);
        fixed.extend_from_slice(&self.config.ssz_encode());
        encode_u64(&mut fixed, self.slot.get());
        fixed.extend_from_slice(&self.latest_block_header.ssz_encode());
        fixed.extend_from_slice(&self.latest_justified.ssz_encode());
        fixed.extend_from_slice(&self.latest_finalized.ssz_encode());
        debug_assert_eq!(fixed.len(), STATE_FIXED_END);

        let hist = encode_hash_list(&self.historical_block_hashes);
        let just_bits = encode_bitlist(&self.justified_slots);
        let mut vals = Vec::with_capacity(self.validators.len());
        for v in &self.validators {
            vals.push(v.ssz_encode());
        }
        let vals_enc = encode_offset_list(&vals)?;
        let j_roots = encode_hash_list(&self.justifications_roots);
        let j_vals = encode_bitlist(&self.justifications_validators);

        let var_parts = [hist, just_bits, vals_enc, j_roots, j_vals];
        let mut cursor = (fixed.len() + STATE_VAR_FIELDS * 4) as u32;
        for part in &var_parts {
            encode_u32(&mut fixed, cursor);
            cursor = cursor
                .checked_add(part.len() as u32)
                .ok_or(TypesError::InvalidContainer(
                    "state variable section overflow".into(),
                ))?;
        }
        for part in &var_parts {
            fixed.extend_from_slice(part);
        }
        Ok(fixed)
    }

    /// Decode state. Field order matches ethlambda `State` and Ream `LeanState`.
    pub fn ssz_decode(input: &[u8]) -> Result<Self, TypesError> {
        if input.is_empty() {
            return Err(TypesError::InvalidContainer(
                "empty State SSZ payload".into(),
            ));
        }
        need(input, 0, STATE_FIXED_END)?;
        let config = GenesisConfig::ssz_decode(&input[0..8])?;
        let mut c = 8;
        let slot = Slot::new(decode_u64(input, &mut c)?);
        let latest_block_header = BlockHeader::ssz_decode(&input[c..c + HEADER_BYTES])?;
        c += HEADER_BYTES;
        let latest_justified = Checkpoint::ssz_decode(&input[c..c + CHECKPOINT_BYTES])?;
        c += CHECKPOINT_BYTES;
        let latest_finalized = Checkpoint::ssz_decode(&input[c..c + CHECKPOINT_BYTES])?;
        c += CHECKPOINT_BYTES;
        debug_assert_eq!(c, STATE_FIXED_END);

        let parts = decode_container_offsets(input, STATE_FIXED_END, STATE_VAR_FIELDS)?;
        let state = Self {
            config,
            slot,
            latest_block_header,
            latest_justified,
            latest_finalized,
            historical_block_hashes: decode_hash32_list(parts[0], HISTORICAL_ROOTS_LIMIT)?,
            justified_slots: decode_bitlist(parts[1], HISTORICAL_ROOTS_LIMIT)?,
            validators: decode_validators(parts[2])?,
            justifications_roots: decode_hash32_list(parts[3], HISTORICAL_ROOTS_LIMIT)?,
            justifications_validators: decode_bitlist(parts[4], JUSTIFICATION_VALIDATORS_LIMIT)?,
        };
        state.validate_bounds()?;
        Ok(state)
    }
}

fn encode_hash_list(hashes: &[Hash32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(hashes.len() * 32);
    for h in hashes {
        encode_fixed_bytes(&mut out, h);
    }
    out
}

fn decode_validators(input: &[u8]) -> Result<Vec<Validator>, TypesError> {
    let parts = decode_offset_list(input, VALIDATOR_REGISTRY_LIMIT)?;
    let mut out = Vec::with_capacity(parts.len());
    for p in parts {
        out.push(Validator::ssz_decode(p)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Bytes52, ValidatorIndex, HASH32_ZERO};

    fn sample_state() -> State {
        let v0 = Validator::new(Bytes52::ZERO, Bytes52([1u8; 52]), ValidatorIndex::new(0))
            .unwrap();
        let v1 = Validator::new(Bytes52([2u8; 52]), Bytes52([3u8; 52]), ValidatorIndex::new(1))
            .unwrap();
        State {
            config: GenesisConfig::new(1_700_000_000),
            slot: Slot::new(7),
            latest_block_header: BlockHeader {
                slot: Slot::new(7),
                proposer_index: ValidatorIndex::new(1),
                parent_root: [9u8; 32],
                state_root: HASH32_ZERO,
                body_root: [4u8; 32],
            },
            latest_justified: Checkpoint {
                root: [5u8; 32],
                slot: Slot::new(6),
            },
            latest_finalized: Checkpoint {
                root: [6u8; 32],
                slot: Slot::new(5),
            },
            historical_block_hashes: vec![[7u8; 32], [8u8; 32]],
            justified_slots: vec![true, false, true],
            validators: vec![v0, v1],
            justifications_roots: vec![[10u8; 32]],
            justifications_validators: vec![true, true, false, true],
        }
    }

    #[test]
    fn state_ssz_roundtrip() {
        let state = sample_state();
        let enc = state.ssz_encode().unwrap();
        let back = State::ssz_decode(&enc).unwrap();
        assert_eq!(back, state);
    }

    #[test]
    fn empty_payload_rejected() {
        assert!(State::ssz_decode(&[]).is_err());
    }

    #[test]
    fn empty_registry_roundtrip() {
        let state = State {
            config: GenesisConfig::new(42),
            ..State::default()
        };
        let enc = state.ssz_encode().unwrap();
        let back = State::ssz_decode(&enc).unwrap();
        assert_eq!(back, state);
    }
}
