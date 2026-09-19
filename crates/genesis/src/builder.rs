//! Deterministic genesis state construction.

use ethean_primitives::{Bytes52, Slot, ValidatorIndex, HASH32_ZERO};
use ethean_ssz::Root;
use ethean_types::{BlockHeader, Checkpoint, GenesisConfig, State, Validator};

use crate::error::GenesisError;

/// Built genesis state plus its `hash_tree_root`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltGenesis {
    pub state: State,
    pub state_root: Root,
}

/// Constructs a slot-0 Lean [`State`] from validator public keys.
#[derive(Debug, Clone)]
pub struct GenesisBuilder {
    genesis_time_secs: u64,
    keys: Vec<(Bytes52, Bytes52)>,
    allow_empty: bool,
}

impl GenesisBuilder {
    pub fn new(genesis_time_secs: u64) -> Self {
        Self {
            genesis_time_secs,
            keys: Vec::new(),
            allow_empty: false,
        }
    }

    /// Append `(attestation_pk, proposal_pk)` pairs in registry order.
    pub fn with_validator_keys(mut self, keys: Vec<(Bytes52, Bytes52)>) -> Self {
        self.keys = keys;
        self
    }

    pub fn push_validator(mut self, attestation_pk: Bytes52, proposal_pk: Bytes52) -> Self {
        self.keys.push((attestation_pk, proposal_pk));
        self
    }

    /// Test helper: allow an empty validator registry.
    pub fn allow_empty_for_tests(mut self) -> Self {
        self.allow_empty = true;
        self
    }

    pub fn build(self) -> Result<BuiltGenesis, GenesisError> {
        if self.keys.is_empty() && !self.allow_empty {
            return Err(GenesisError::EmptyValidators);
        }

        let mut validators = Vec::with_capacity(self.keys.len());
        for (i, (att, prop)) in self.keys.into_iter().enumerate() {
            let expected = i as u64;
            let index = ValidatorIndex::new(expected);
            let v = Validator::new(att, prop, index).map_err(|e| match e {
                ethean_types::TypesError::ValidatorIndexOutOfRange { index, .. } => {
                    GenesisError::InvalidValidatorIndex {
                        index,
                        expected,
                    }
                }
                other => GenesisError::Types(other.to_string()),
            })?;
            if v.index.get() != expected {
                return Err(GenesisError::InvalidValidatorIndex {
                    index: v.index.get(),
                    expected,
                });
            }
            validators.push(v);
        }

        let state = State {
            config: GenesisConfig::new(self.genesis_time_secs),
            slot: Slot::ZERO,
            latest_block_header: BlockHeader {
                slot: Slot::ZERO,
                proposer_index: ValidatorIndex::ZERO,
                parent_root: HASH32_ZERO,
                state_root: HASH32_ZERO,
                body_root: HASH32_ZERO,
            },
            latest_justified: Checkpoint::genesis(),
            latest_finalized: Checkpoint::genesis(),
            historical_block_hashes: Vec::new(),
            justified_slots: Vec::new(),
            validators,
            justifications_roots: Vec::new(),
            justifications_validators: Vec::new(),
        };

        let state_root = state
            .hash_tree_root()
            .map_err(|e| GenesisError::Types(e.to_string()))?;

        Ok(BuiltGenesis { state, state_root })
    }
}

/// Convenience: single zero-key validator for local node smoke starts.
pub fn local_smoke_genesis(genesis_time_secs: u64) -> Result<BuiltGenesis, GenesisError> {
    GenesisBuilder::new(genesis_time_secs)
        .push_validator(Bytes52::ZERO, Bytes52::ZERO)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_slot_zero_and_validator_count() {
        let keys = vec![
            (Bytes52([1u8; 52]), Bytes52([2u8; 52])),
            (Bytes52([3u8; 52]), Bytes52([4u8; 52])),
        ];
        let built = GenesisBuilder::new(1_700_000_000)
            .with_validator_keys(keys)
            .build()
            .unwrap();
        assert_eq!(built.state.slot, Slot::ZERO);
        assert_eq!(built.state.validators.len(), 2);
        assert_eq!(built.state.validators[1].index.get(), 1);
        assert_eq!(built.state.config.genesis_time, 1_700_000_000);
        assert_eq!(built.state.latest_block_header.parent_root, HASH32_ZERO);
        assert_eq!(built.state_root.len(), 32);
    }

    #[test]
    fn rejects_empty_network_genesis() {
        assert_eq!(
            GenesisBuilder::new(0).build().unwrap_err(),
            GenesisError::EmptyValidators
        );
    }

    #[test]
    fn allow_empty_for_tests() {
        let built = GenesisBuilder::new(10)
            .allow_empty_for_tests()
            .build()
            .unwrap();
        assert!(built.state.validators.is_empty());
    }
}
