//! Transition context: profile only (no wall clock, no I/O).

use ethean_profile::ChainProfile;

/// Immutable inputs required to apply a state transition.
#[derive(Debug, Clone)]
pub struct TransitionContext {
    pub profile: ChainProfile,
}

impl TransitionContext {
    pub fn new(profile: ChainProfile) -> Self {
        Self { profile }
    }

    /// Cap on distinct attestation data entries per block.
    pub fn max_attestations_data(&self) -> usize {
        self.profile.max_attestations_data as usize
    }
}
