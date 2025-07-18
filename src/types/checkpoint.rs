//! Checkpoint and epoch types

use serde::{Deserialize, Serialize};

pub type Epoch = u64;

/// Finality checkpoint
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checkpoint {
    pub epoch: Epoch,
    pub root: [u8; 32],
}

impl Checkpoint {
    /// New checkpoint
    pub fn new(epoch: Epoch, root: [u8; 32]) -> Self {
        Self { epoch, root }
    }

    /// Genesis checkpoint
    pub fn genesis() -> Self {
        Self::new(0, [0u8; 32])
    }
}

impl Default for Checkpoint {
    fn default() -> Self {
        Self::genesis()
    }
}
