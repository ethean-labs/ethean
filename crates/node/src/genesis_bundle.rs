//! Operator `genesis.json` for a durable data-dir (profile + full genesis state).

use crate::chain_snap::StateSnap;
use crate::persist_paths::{write_atomic, PersistPaths};
use crate::{Error, Result};
use ethean_genesis::BuiltGenesis;
use ethean_profile::ChainProfile;
use ethean_types::State;
use serde::{Deserialize, Serialize};
use std::fs;
use tracing::info;

const SCHEMA: &str = "ethean-genesis-v1";
const CLIENT: &str = "ethean";

/// Broad genesis document written once when a data-dir is first opened.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenesisDocument {
    pub schema: String,
    pub client: String,
    pub fork_name: String,
    pub genesis_time: u64,
    pub seconds_per_slot: u64,
    pub intervals_per_slot: u64,
    pub milliseconds_per_slot: u64,
    pub milliseconds_per_interval: u64,
    pub gossip_disparity_intervals: u64,
    pub justification_lookback_slots: u64,
    pub validator_registry_limit: u64,
    pub historical_roots_limit: u64,
    pub attestation_committee_count: u64,
    pub max_attestations_data: u64,
    pub xmss_public_key_bytes: u64,
    pub xmss_signature_bytes: u64,
    pub validator_count: usize,
    pub state_root: String,
    pub state: StateSnap,
}

impl GenesisDocument {
    pub fn from_built(profile: &ChainProfile, built: &BuiltGenesis) -> Self {
        Self {
            schema: SCHEMA.to_string(),
            client: CLIENT.to_string(),
            fork_name: profile.fork_name.to_string(),
            genesis_time: built.state.config.genesis_time,
            seconds_per_slot: profile.seconds_per_slot,
            intervals_per_slot: profile.intervals_per_slot,
            milliseconds_per_slot: profile.milliseconds_per_slot,
            milliseconds_per_interval: profile.milliseconds_per_interval,
            gossip_disparity_intervals: profile.gossip_disparity_intervals,
            justification_lookback_slots: profile.justification_lookback_slots,
            validator_registry_limit: profile.validator_registry_limit,
            historical_roots_limit: profile.historical_roots_limit,
            attestation_committee_count: profile.attestation_committee_count,
            max_attestations_data: profile.max_attestations_data,
            xmss_public_key_bytes: profile.xmss_public_key_bytes,
            xmss_signature_bytes: profile.xmss_signature_bytes,
            validator_count: built.state.validators.len(),
            state_root: crate::persist_ssz::hex32(&built.state_root),
            state: StateSnap::from_state(&built.state),
        }
    }

    pub fn into_state(self) -> Result<State> {
        self.state
            .into_state()
            .map_err(|e| Error::Config(format!("genesis.json state: {e}")))
    }
}

/// Write `genesis.json` (pretty JSON for operators).
pub fn save_genesis_json(
    paths: &PersistPaths,
    profile: &ChainProfile,
    built: &BuiltGenesis,
) -> Result<()> {
    paths.ensure_dir()?;
    let doc = GenesisDocument::from_built(profile, built);
    let raw = serde_json::to_string_pretty(&doc)
        .map_err(|e| Error::Config(format!("encode genesis.json: {e}")))?;
    write_atomic(&paths.genesis_json(), raw.as_bytes())?;
    info!(
        path = %paths.genesis_json().display(),
        genesis_time = doc.genesis_time,
        validators = doc.validator_count,
        "wrote genesis.json"
    );
    Ok(())
}

/// Load `genesis.json` when present.
pub fn load_genesis_json(paths: &PersistPaths) -> Result<Option<GenesisDocument>> {
    let path = paths.genesis_json();
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)
        .map_err(|e| Error::Config(format!("read {}: {e}", path.display())))?;
    let doc: GenesisDocument = serde_json::from_str(&raw)
        .map_err(|e| Error::Config(format!("parse {}: {e}", path.display())))?;
    if doc.schema != SCHEMA {
        return Err(Error::Config(format!(
            "genesis.json schema {} is not {SCHEMA}",
            doc.schema
        )));
    }
    info!(
        path = %path.display(),
        genesis_time = doc.genesis_time,
        validators = doc.validator_count,
        "loaded genesis.json"
    );
    Ok(Some(doc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persist_paths::PersistPaths;
    use ethean_genesis::GenesisBuilder;
    use ethean_primitives::Bytes52;
    use ethean_profile::lstar_devnet;

    #[test]
    fn json_roundtrip_keeps_time_and_registry() {
        let dir = tempfile::tempdir().unwrap();
        let paths = PersistPaths::new(dir.path());
        let profile = lstar_devnet().unwrap();
        let built = GenesisBuilder::new(1_700_000_111)
            .push_validator(Bytes52::ZERO, Bytes52::ZERO)
            .push_validator(Bytes52([1u8; 52]), Bytes52([2u8; 52]))
            .build()
            .unwrap();
        save_genesis_json(&paths, &profile, &built).unwrap();
        let doc = load_genesis_json(&paths).unwrap().unwrap();
        assert_eq!(doc.client, "ethean");
        assert_eq!(doc.fork_name, "lstar");
        assert_eq!(doc.genesis_time, 1_700_000_111);
        assert_eq!(doc.validator_count, 2);
        assert_eq!(doc.seconds_per_slot, 4);
        let state = doc.into_state().unwrap();
        assert_eq!(state.validators.len(), 2);
    }
}
