//! Validator types - single responsibility
//!
//! Handles validator data structures only.

use serde::{Deserialize, Serialize};

pub type ValidatorIndex = u64;

// Use Vec for now to avoid serde issues with large arrays
pub type PublicKey = Vec<u8>; // 48 bytes

/// Single validator record
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validator {
    pub pubkey: PublicKey,
    pub withdrawal_credentials: [u8; 32],
    pub effective_balance: u64,
    pub slashed: bool,
    pub activation_epoch: super::checkpoint::Epoch,
    pub exit_epoch: super::checkpoint::Epoch,
}

/// Set of validators
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorSet {
    pub validators: Vec<Validator>,
}

impl Validator {
    /// Create new validator
    pub fn new(pubkey: PublicKey, withdrawal_credentials: [u8; 32]) -> Self {
        Self {
            pubkey,
            withdrawal_credentials,
            effective_balance: 0,
            slashed: false,
            activation_epoch: u64::MAX,
            exit_epoch: u64::MAX,
        }
    }

    /// Check if active at epoch
    pub fn is_active(&self, epoch: super::checkpoint::Epoch) -> bool {
        self.activation_epoch <= epoch && epoch < self.exit_epoch
    }
}

impl ValidatorSet {
    /// Create empty set
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    /// Add validator
    pub fn add(&mut self, validator: Validator) -> ValidatorIndex {
        let index = self.validators.len() as ValidatorIndex;
        self.validators.push(validator);
        index
    }

    /// Get validator
    pub fn get(&self, index: ValidatorIndex) -> Option<&Validator> {
        self.validators.get(index as usize)
    }

    /// Get active validators
    pub fn active_at(&self, epoch: super::checkpoint::Epoch) -> Vec<ValidatorIndex> {
        self.validators
            .iter()
            .enumerate()
            .filter(|(_, v)| v.is_active(epoch))
            .map(|(i, _)| i as ValidatorIndex)
            .collect()
    }
}

impl Default for ValidatorSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = Validator::new(vec![1u8; 48], [2u8; 32]);
        assert_eq!(validator.pubkey, vec![1u8; 48]);
        assert!(!validator.slashed);
    }

    #[test]
    fn test_validator_set() {
        let mut set = ValidatorSet::new();
        let validator = Validator::new(vec![1u8; 48], [2u8; 32]);
        let index = set.add(validator);
        
        assert_eq!(index, 0);
        assert!(set.get(0).is_some());
    }
}
