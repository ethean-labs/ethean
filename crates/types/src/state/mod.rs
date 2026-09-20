//! Consensus state container (leanSpec lstar `State`).

mod codec;

use ethean_primitives::{Hash32, Slot};
use ethean_ssz::{
    hash_tree_root_bitlist, hash_tree_root_bytes, hash_tree_root_container, hash_tree_root_list,
    hash_tree_root_u64, Root,
};

use crate::block::BlockHeader;
use crate::checkpoint::Checkpoint;
use crate::error::TypesError;
use crate::genesis::GenesisConfig;
use crate::limits::{
    HISTORICAL_ROOTS_LIMIT, JUSTIFICATION_VALIDATORS_LIMIT, VALIDATOR_REGISTRY_LIMIT,
};
use crate::validator::Validator;

/// Main consensus state object.
///
/// Field order matches ethlambda `State` and Ream `LeanState` for SSZ interop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub config: GenesisConfig,
    pub slot: Slot,
    pub latest_block_header: BlockHeader,
    pub latest_justified: Checkpoint,
    pub latest_finalized: Checkpoint,
    pub historical_block_hashes: Vec<Hash32>,
    /// Justified-slot bitlist (window helpers live in transition).
    pub justified_slots: Vec<bool>,
    pub validators: Vec<Validator>,
    pub justifications_roots: Vec<Hash32>,
    /// Flattened per-root validator vote bits.
    pub justifications_validators: Vec<bool>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            config: GenesisConfig::default(),
            slot: Slot::ZERO,
            latest_block_header: BlockHeader::default(),
            latest_justified: Checkpoint::genesis(),
            latest_finalized: Checkpoint::genesis(),
            historical_block_hashes: Vec::new(),
            justified_slots: Vec::new(),
            validators: Vec::new(),
            justifications_roots: Vec::new(),
            justifications_validators: Vec::new(),
        }
    }
}

impl State {
    pub fn genesis_time(&self) -> u64 {
        self.config.genesis_time
    }

    pub fn validator(&self, index: usize) -> Option<&Validator> {
        self.validators.get(index)
    }

    pub fn hash_tree_root(&self) -> Result<Root, TypesError> {
        self.validate_bounds()?;
        let hist_roots: Vec<Root> = self
            .historical_block_hashes
            .iter()
            .map(|h| hash_tree_root_bytes(h))
            .collect();
        let val_roots: Vec<Root> = self.validators.iter().map(|v| v.hash_tree_root()).collect();
        let j_roots: Vec<Root> = self
            .justifications_roots
            .iter()
            .map(|h| hash_tree_root_bytes(h))
            .collect();
        Ok(hash_tree_root_container(&[
            self.config.hash_tree_root(),
            hash_tree_root_u64(self.slot.get()),
            self.latest_block_header.hash_tree_root(),
            self.latest_justified.hash_tree_root(),
            self.latest_finalized.hash_tree_root(),
            hash_tree_root_list(&hist_roots, HISTORICAL_ROOTS_LIMIT)?,
            hash_tree_root_bitlist(&self.justified_slots, HISTORICAL_ROOTS_LIMIT)?,
            hash_tree_root_list(&val_roots, VALIDATOR_REGISTRY_LIMIT)?,
            hash_tree_root_list(&j_roots, HISTORICAL_ROOTS_LIMIT)?,
            hash_tree_root_bitlist(
                &self.justifications_validators,
                JUSTIFICATION_VALIDATORS_LIMIT,
            )?,
        ]))
    }

    pub(crate) fn validate_bounds(&self) -> Result<(), TypesError> {
        if self.historical_block_hashes.len() > HISTORICAL_ROOTS_LIMIT {
            return Err(TypesError::ListTooLong {
                got: self.historical_block_hashes.len(),
                limit: HISTORICAL_ROOTS_LIMIT,
            });
        }
        if self.justified_slots.len() > HISTORICAL_ROOTS_LIMIT {
            return Err(TypesError::ListTooLong {
                got: self.justified_slots.len(),
                limit: HISTORICAL_ROOTS_LIMIT,
            });
        }
        if self.validators.len() > VALIDATOR_REGISTRY_LIMIT {
            return Err(TypesError::ListTooLong {
                got: self.validators.len(),
                limit: VALIDATOR_REGISTRY_LIMIT,
            });
        }
        if self.justifications_roots.len() > HISTORICAL_ROOTS_LIMIT {
            return Err(TypesError::ListTooLong {
                got: self.justifications_roots.len(),
                limit: HISTORICAL_ROOTS_LIMIT,
            });
        }
        if self.justifications_validators.len() > JUSTIFICATION_VALIDATORS_LIMIT {
            return Err(TypesError::ListTooLong {
                got: self.justifications_validators.len(),
                limit: JUSTIFICATION_VALIDATORS_LIMIT,
            });
        }
        Ok(())
    }
}
