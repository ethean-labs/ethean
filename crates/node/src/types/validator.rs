//! Validator types — single responsibility.

use ethean_primitives::{Epoch, ValidatorIndex};
use serde::{Deserialize, Serialize};

pub use ethean_primitives::ValidatorIndex;

/// Public key bytes (legacy BLS-sized placeholder).
pub type PublicKey = Vec<u8>;

/// Single validator record
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validator {
    pub pubkey: PublicKey,
    pub withdrawal_credentials: [u8; 32],
    pub effective_balance: u64,
    pub slashed: bool,
    pub activation_epoch: Epoch,
    pub exit_epoch: Epoch,
    pub activation_eligibility_epoch: Epoch,
    pub withdrawable_epoch: Epoch,
}

/// Set of validators
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorSet {
    pub validators: Vec<Validator>,
}

impl Validator {
    /// Create new validator
    pub fn new(pubkey: PublicKey, withdrawal_credentials: [u8; 32]) -> Self {
        let far = Epoch::new(u64::MAX);
        Self {
            pubkey,
            withdrawal_credentials,
            effective_balance: 0,
            slashed: false,
            activation_epoch: far,
            exit_epoch: far,
            activation_eligibility_epoch: far,
            withdrawable_epoch: far,
        }
    }

    /// Check if active at epoch
    pub fn is_active(&self, epoch: Epoch) -> bool {
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
        let index = ValidatorIndex::new(self.validators.len() as u64);
        self.validators.push(validator);
        index
    }

    /// Get validator
    pub fn get(&self, index: ValidatorIndex) -> Option<&Validator> {
        self.validators.get(index.as_usize())
    }

    /// Get validator mutably
    pub fn get_mut(&mut self, index: ValidatorIndex) -> Option<&mut Validator> {
        self.validators.get_mut(index.as_usize())
    }

    /// Get active validators
    pub fn active_at(&self, epoch: Epoch) -> Vec<ValidatorIndex> {
        self.validators
            .iter()
            .enumerate()
            .filter(|(_, v)| v.is_active(epoch))
            .map(|(i, _)| ValidatorIndex::new(i as u64))
            .collect()
    }

    /// Iterate validators mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Validator> {
        self.validators.iter_mut()
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

        assert_eq!(index, ValidatorIndex::ZERO);
        assert!(set.get(ValidatorIndex::ZERO).is_some());
    }
}
