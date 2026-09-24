//! JSON DTOs for durable local head/genesis snapshots (file persist).

use ethean_primitives::{Bytes52, Hash32, Slot, ValidatorIndex};
use ethean_types::{BlockHeader, Checkpoint, GenesisConfig, State, Validator};
use serde::{Deserialize, Serialize};

/// On-disk pin: same genesis for every restart under a data-dir.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenesisPin {
    pub genesis_time: u64,
    pub validators: usize,
    pub seconds_per_slot: u64,
}

/// Restorable head snapshot written after local finality advances.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeadSnap {
    pub head_root: String,
    pub state: StateSnap,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateSnap {
    pub genesis_time: u64,
    pub slot: u64,
    pub latest_block_header: HeaderSnap,
    pub latest_justified: CheckpointSnap,
    pub latest_finalized: CheckpointSnap,
    pub historical_block_hashes: Vec<String>,
    pub justified_slots: Vec<bool>,
    pub validators: Vec<ValidatorSnap>,
    pub justifications_roots: Vec<String>,
    pub justifications_validators: Vec<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeaderSnap {
    pub slot: u64,
    pub proposer_index: u64,
    pub parent_root: String,
    pub state_root: String,
    pub body_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckpointSnap {
    pub root: String,
    pub slot: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidatorSnap {
    pub attestation_public_key: String,
    pub proposal_public_key: String,
    pub index: u64,
}

impl HeadSnap {
    pub fn from_owner(head_root: &Hash32, state: &State) -> Self {
        Self {
            head_root: hex32(head_root),
            state: StateSnap::from_state(state),
        }
    }

    pub fn into_parts(self) -> Result<(Hash32, State), String> {
        let head_root = parse_hex32(&self.head_root)?;
        let state = self.state.into_state()?;
        Ok((head_root, state))
    }
}

impl StateSnap {
    pub fn from_state(state: &State) -> Self {
        Self {
            genesis_time: state.config.genesis_time,
            slot: state.slot.get(),
            latest_block_header: HeaderSnap::from_header(&state.latest_block_header),
            latest_justified: CheckpointSnap::from_cp(&state.latest_justified),
            latest_finalized: CheckpointSnap::from_cp(&state.latest_finalized),
            historical_block_hashes: state.historical_block_hashes.iter().map(hex32).collect(),
            justified_slots: state.justified_slots.clone(),
            validators: state.validators.iter().map(ValidatorSnap::from_v).collect(),
            justifications_roots: state.justifications_roots.iter().map(hex32).collect(),
            justifications_validators: state.justifications_validators.clone(),
        }
    }

    pub fn into_state(self) -> Result<State, String> {
        let mut validators = Vec::with_capacity(self.validators.len());
        for v in self.validators {
            validators.push(v.into_validator()?);
        }
        Ok(State {
            config: GenesisConfig::new(self.genesis_time),
            slot: Slot::new(self.slot),
            latest_block_header: self.latest_block_header.into_header()?,
            latest_justified: self.latest_justified.into_cp()?,
            latest_finalized: self.latest_finalized.into_cp()?,
            historical_block_hashes: map_hex32_list(&self.historical_block_hashes)?,
            justified_slots: self.justified_slots,
            validators,
            justifications_roots: map_hex32_list(&self.justifications_roots)?,
            justifications_validators: self.justifications_validators,
        })
    }
}

impl HeaderSnap {
    fn from_header(h: &BlockHeader) -> Self {
        Self {
            slot: h.slot.get(),
            proposer_index: h.proposer_index.get(),
            parent_root: hex32(&h.parent_root),
            state_root: hex32(&h.state_root),
            body_root: hex32(&h.body_root),
        }
    }

    fn into_header(self) -> Result<BlockHeader, String> {
        Ok(BlockHeader {
            slot: Slot::new(self.slot),
            proposer_index: ValidatorIndex::new(self.proposer_index),
            parent_root: parse_hex32(&self.parent_root)?,
            state_root: parse_hex32(&self.state_root)?,
            body_root: parse_hex32(&self.body_root)?,
        })
    }
}

impl CheckpointSnap {
    fn from_cp(c: &Checkpoint) -> Self {
        Self {
            root: hex32(&c.root),
            slot: c.slot.get(),
        }
    }

    fn into_cp(self) -> Result<Checkpoint, String> {
        Ok(Checkpoint {
            root: parse_hex32(&self.root)?,
            slot: Slot::new(self.slot),
        })
    }
}

impl ValidatorSnap {
    fn from_v(v: &Validator) -> Self {
        Self {
            attestation_public_key: hex_bytes(v.attestation_public_key.as_bytes()),
            proposal_public_key: hex_bytes(v.proposal_public_key.as_bytes()),
            index: v.index.get(),
        }
    }

    fn into_validator(self) -> Result<Validator, String> {
        let att = parse_bytes52(&self.attestation_public_key)?;
        let prop = parse_bytes52(&self.proposal_public_key)?;
        Validator::new(att, prop, ValidatorIndex::new(self.index)).map_err(|e| e.to_string())
    }
}

fn hex32(h: &Hash32) -> String {
    hex_bytes(h.as_ref())
}

fn hex_bytes(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn parse_hex32(s: &str) -> Result<Hash32, String> {
    let bytes = parse_hex(s)?;
    if bytes.len() != 32 {
        return Err(format!("expected 32 bytes, got {}", bytes.len()));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn parse_bytes52(s: &str) -> Result<Bytes52, String> {
    let bytes = parse_hex(s)?;
    Bytes52::from_slice(&bytes).map_err(|e| e.to_string())
}

fn parse_hex(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim().trim_start_matches("0x");
    if s.len() % 2 != 0 {
        return Err("odd hex length".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

fn map_hex32_list(list: &[String]) -> Result<Vec<Hash32>, String> {
    list.iter().map(|s| parse_hex32(s)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::HASH32_ZERO;

    #[test]
    fn roundtrip_emptyish_state() {
        let state = State::default();
        let snap = HeadSnap::from_owner(&HASH32_ZERO, &state);
        let (root, back) = snap.into_parts().unwrap();
        assert_eq!(root, HASH32_ZERO);
        assert_eq!(back.slot, state.slot);
        assert_eq!(back.config.genesis_time, state.config.genesis_time);
    }
}
