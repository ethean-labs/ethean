//! Decode leanSpec JSON containers into Ethean types (fork-choice subset).

use crate::hex::{decode_hex_fixed, HexError};
use ethean_primitives::{Bytes52, Hash32, Slot, ValidatorIndex};
use ethean_types::{
    Block, BlockBody, BlockHeader, Checkpoint, GenesisConfig, State, Validator,
};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JsonTypesError {
    #[error(transparent)]
    Hex(#[from] HexError),
    #[error("types: {0}")]
    Types(String),
    #[error("serde: {0}")]
    Serde(String),
}

#[derive(Debug, Deserialize)]
struct JsonList<T> {
    data: Vec<T>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonConfig {
    genesis_time: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonCheckpoint {
    root: String,
    slot: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonHeader {
    slot: u64,
    proposer_index: u64,
    parent_root: String,
    state_root: String,
    body_root: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonValidator {
    attestation_public_key: String,
    proposal_public_key: String,
    index: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonState {
    config: JsonConfig,
    slot: u64,
    latest_block_header: JsonHeader,
    latest_justified: JsonCheckpoint,
    latest_finalized: JsonCheckpoint,
    historical_block_hashes: JsonList<String>,
    justified_slots: JsonList<bool>,
    validators: JsonList<JsonValidator>,
    justifications_roots: JsonList<String>,
    justifications_validators: JsonList<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonBody {
    attestations: JsonList<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonBlock {
    slot: u64,
    proposer_index: u64,
    parent_root: String,
    state_root: String,
    body: JsonBody,
}

fn hash32(s: &str) -> Result<Hash32, JsonTypesError> {
    Ok(decode_hex_fixed::<32>(s)?)
}

fn bytes52(s: &str) -> Result<Bytes52, JsonTypesError> {
    Ok(Bytes52(decode_hex_fixed::<52>(s)?))
}

fn checkpoint(j: &JsonCheckpoint) -> Result<Checkpoint, JsonTypesError> {
    Ok(Checkpoint {
        root: hash32(&j.root)?,
        slot: Slot::new(j.slot),
    })
}

fn header(j: &JsonHeader) -> Result<BlockHeader, JsonTypesError> {
    Ok(BlockHeader {
        slot: Slot::new(j.slot),
        proposer_index: ValidatorIndex::new(j.proposer_index),
        parent_root: hash32(&j.parent_root)?,
        state_root: hash32(&j.state_root)?,
        body_root: hash32(&j.body_root)?,
    })
}

/// Decode a leanSpec `State` JSON object.
pub fn state_from_value(v: &serde_json::Value) -> Result<State, JsonTypesError> {
    let j: JsonState =
        serde_json::from_value(v.clone()).map_err(|e| JsonTypesError::Serde(e.to_string()))?;
    let mut validators = Vec::with_capacity(j.validators.data.len());
    for row in &j.validators.data {
        validators.push(
            Validator::new(
                bytes52(&row.attestation_public_key)?,
                bytes52(&row.proposal_public_key)?,
                ValidatorIndex::new(row.index),
            )
            .map_err(|e| JsonTypesError::Types(e.to_string()))?,
        );
    }
    let mut historical = Vec::with_capacity(j.historical_block_hashes.data.len());
    for h in &j.historical_block_hashes.data {
        historical.push(hash32(h)?);
    }
    let mut j_roots = Vec::with_capacity(j.justifications_roots.data.len());
    for h in &j.justifications_roots.data {
        j_roots.push(hash32(h)?);
    }
    Ok(State {
        config: GenesisConfig::new(j.config.genesis_time),
        slot: Slot::new(j.slot),
        latest_block_header: header(&j.latest_block_header)?,
        latest_justified: checkpoint(&j.latest_justified)?,
        latest_finalized: checkpoint(&j.latest_finalized)?,
        historical_block_hashes: historical,
        justified_slots: j.justified_slots.data,
        validators,
        justifications_roots: j_roots,
        justifications_validators: j.justifications_validators.data,
    })
}

/// Decode a leanSpec `Block` JSON object (attestations must be empty for now).
pub fn block_from_value(v: &serde_json::Value) -> Result<Block, JsonTypesError> {
    let j: JsonBlock =
        serde_json::from_value(v.clone()).map_err(|e| JsonTypesError::Serde(e.to_string()))?;
    if !j.body.attestations.data.is_empty() {
        return Err(JsonTypesError::Types(
            "fixture block attestations not supported yet".into(),
        ));
    }
    Ok(Block {
        slot: Slot::new(j.slot),
        proposer_index: ValidatorIndex::new(j.proposer_index),
        parent_root: hash32(&j.parent_root)?,
        state_root: hash32(&j.state_root)?,
        body: BlockBody::default(),
    })
}
