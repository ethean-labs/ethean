//! Checkpoint and epoch types

use ethean_primitives::Hash32;
use serde::{Deserialize, Serialize};

pub use ethean_primitives::Epoch;

/// Finality checkpoint
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checkpoint {
    pub epoch: Epoch,
    pub root: Hash32,
}

impl Checkpoint {
    /// New checkpoint
    pub fn new(epoch: Epoch, root: Hash32) -> Self {
        Self { epoch, root }
    }

    /// Genesis checkpoint
    pub fn genesis() -> Self {
        Self::new(Epoch::ZERO, [0u8; 32])
    }
}

impl Default for Checkpoint {
    fn default() -> Self {
        Self::genesis()
    }
}
