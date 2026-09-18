//! Beacon state types

use ethean_primitives::{Epoch, Slot, ValidatorIndex};
use serde::{Deserialize, Serialize};

pub type StateRoot = [u8; 32];

/// Main beacon state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconState {
    pub genesis_time: u64,
    pub slot: Slot,
    pub latest_block_header: super::block::BeaconBlockHeader,
    pub validators: super::validator::ValidatorSet,
    pub balances: Vec<u64>,
    pub finalized_checkpoint: super::checkpoint::Checkpoint,
}

impl BeaconState {
    /// Get current epoch using an explicit slots-per-epoch period.
    pub fn current_epoch(&self, slots_per_epoch: u64) -> Epoch {
        Epoch::from_slot(self.slot, slots_per_epoch).unwrap_or(Epoch::ZERO)
    }

    /// Get validator
    pub fn validator(&self, index: ValidatorIndex) -> Option<&super::validator::Validator> {
        self.validators.get(index)
    }

    /// Get balance
    pub fn balance(&self, index: ValidatorIndex) -> Option<u64> {
        self.balances.get(index.as_usize()).copied()
    }
}

impl Default for BeaconState {
    fn default() -> Self {
        Self {
            genesis_time: 0,
            slot: Slot::ZERO,
            latest_block_header: super::block::BeaconBlockHeader::default(),
            validators: super::validator::ValidatorSet::default(),
            balances: Vec::new(),
            finalized_checkpoint: super::checkpoint::Checkpoint::default(),
        }
    }
}
