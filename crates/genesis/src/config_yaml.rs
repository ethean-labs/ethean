//! Load Lean Hive / lean-quickstart `config.yaml` into a genesis state.

use crate::builder::{BuiltGenesis, GenesisBuilder};
use crate::error::GenesisError;
use crate::hex::decode_hex_fixed;
use ethean_primitives::Bytes52;
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Parsed Lean network config (Hive / quickstart shape).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanNetworkConfig {
    pub genesis_time: u64,
    pub attestation_committee_count: u64,
    pub validators: Vec<(Bytes52, Bytes52)>,
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    #[serde(rename = "GENESIS_TIME")]
    genesis_time: u64,
    #[serde(rename = "ATTESTATION_COMMITTEE_COUNT", default)]
    attestation_committee_count: Option<u64>,
    #[serde(rename = "GENESIS_VALIDATORS", default)]
    genesis_validators: Vec<RawValidator>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawValidator {
    Dual(RawDualKeys),
    Single(String),
}

#[derive(Debug, Deserialize)]
struct RawDualKeys {
    #[serde(alias = "attestation_pubkey", alias = "attestation_public_key")]
    attestation_public_key: Option<String>,
    #[serde(alias = "proposal_pubkey", alias = "proposal_public_key")]
    proposal_public_key: Option<String>,
}

/// Read `config.yaml` from disk.
pub fn load_lean_network_config(path: &Path) -> Result<LeanNetworkConfig, GenesisError> {
    let text = fs::read_to_string(path).map_err(|e| {
        GenesisError::LeanConfig(format!("read {}: {e}", path.display()))
    })?;
    parse_lean_network_config(&text)
}

/// Parse Lean `config.yaml` text.
pub fn parse_lean_network_config(text: &str) -> Result<LeanNetworkConfig, GenesisError> {
    let raw: RawConfig = serde_yaml::from_str(text)
        .map_err(|e| GenesisError::LeanConfig(format!("yaml: {e}")))?;
    if raw.genesis_validators.is_empty() {
        return Err(GenesisError::EmptyValidators);
    }
    let mut validators = Vec::with_capacity(raw.genesis_validators.len());
    for (i, entry) in raw.genesis_validators.into_iter().enumerate() {
        let (att, prop) = match entry {
            RawValidator::Single(hex) => {
                let pk = Bytes52(decode_hex_fixed::<52>(&hex)?);
                (pk, pk)
            }
            RawValidator::Dual(d) => {
                let att_s = d.attestation_public_key.ok_or_else(|| {
                    GenesisError::LeanConfig(format!(
                        "GENESIS_VALIDATORS[{i}] missing attestation pubkey"
                    ))
                })?;
                let prop_s = d.proposal_public_key.ok_or_else(|| {
                    GenesisError::LeanConfig(format!(
                        "GENESIS_VALIDATORS[{i}] missing proposal pubkey"
                    ))
                })?;
                (
                    Bytes52(decode_hex_fixed::<52>(&att_s)?),
                    Bytes52(decode_hex_fixed::<52>(&prop_s)?),
                )
            }
        };
        validators.push((att, prop));
    }
    Ok(LeanNetworkConfig {
        genesis_time: raw.genesis_time,
        attestation_committee_count: raw.attestation_committee_count.unwrap_or(1),
        validators,
    })
}

/// Build genesis from a Lean network config file.
pub fn genesis_from_lean_config(path: &Path) -> Result<BuiltGenesis, GenesisError> {
    let cfg = load_lean_network_config(path)?;
    GenesisBuilder::new(cfg.genesis_time)
        .with_validator_keys(cfg.validators)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> String {
        format!("{byte:02x}").repeat(52)
    }

    #[test]
    fn parses_ream_dual_key_shape() {
        let yaml = format!(
            "GENESIS_TIME: 1700000000\nNUM_VALIDATORS: 2\nGENESIS_VALIDATORS:\n  - attestation_public_key: \"{}\"\n    proposal_public_key: \"{}\"\n  - attestation_public_key: \"{}\"\n    proposal_public_key: \"{}\"\n",
            key(1),
            key(2),
            key(3),
            key(4)
        );
        let cfg = parse_lean_network_config(&yaml).unwrap();
        assert_eq!(cfg.genesis_time, 1_700_000_000);
        assert_eq!(cfg.validators.len(), 2);
        assert_eq!(cfg.validators[0].0.as_bytes()[0], 1);
        assert_eq!(cfg.validators[1].1.as_bytes()[0], 4);
        let built = GenesisBuilder::new(cfg.genesis_time)
            .with_validator_keys(cfg.validators)
            .build()
            .unwrap();
        assert_eq!(built.state.validators.len(), 2);
    }

    #[test]
    fn parses_pubkey_aliases_and_0x() {
        let yaml = format!(
            "GENESIS_TIME: 1\nATTESTATION_COMMITTEE_COUNT: 2\nGENESIS_VALIDATORS:\n  - attestation_pubkey: \"0x{}\"\n    proposal_pubkey: \"0x{}\"\n",
            key(0xaa),
            key(0xbb)
        );
        let cfg = parse_lean_network_config(&yaml).unwrap();
        assert_eq!(cfg.attestation_committee_count, 2);
        assert_eq!(cfg.validators[0].0.as_bytes()[0], 0xaa);
        assert_eq!(cfg.validators[0].1.as_bytes()[0], 0xbb);
    }
}
