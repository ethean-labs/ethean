//! Immutable protocol profile values.

use crate::error::ProfileError;

/// Pinned Lean Consensus chain parameters (lstar surface).
///
/// Operational node settings (API bind, P2P listen, etc.) do not belong here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainProfile {
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
    pub fork_name: &'static str,
}

impl ChainProfile {
    /// Validate cross-field invariants. Rejects zero timing/bounds and empty fork name.
    pub fn validate(&self) -> Result<(), ProfileError> {
        if self.seconds_per_slot == 0 {
            return Err(ProfileError::ZeroSecondsPerSlot);
        }
        if self.intervals_per_slot == 0 {
            return Err(ProfileError::ZeroIntervalsPerSlot);
        }
        if self.milliseconds_per_slot == 0 {
            return Err(ProfileError::ZeroMillisecondsPerSlot);
        }
        if self.milliseconds_per_interval == 0 {
            return Err(ProfileError::ZeroMillisecondsPerInterval);
        }
        if self.max_attestations_data == 0 {
            return Err(ProfileError::ZeroMaxAttestationsData);
        }
        if self.validator_registry_limit == 0 {
            return Err(ProfileError::ZeroValidatorRegistryLimit);
        }
        if self.historical_roots_limit == 0 {
            return Err(ProfileError::ZeroHistoricalRootsLimit);
        }
        if self.attestation_committee_count == 0 {
            return Err(ProfileError::ZeroAttestationCommitteeCount);
        }
        if self.xmss_public_key_bytes == 0 {
            return Err(ProfileError::ZeroXmssPublicKeyBytes);
        }
        if self.xmss_signature_bytes == 0 {
            return Err(ProfileError::ZeroXmssSignatureBytes);
        }
        if self.fork_name.is_empty() {
            return Err(ProfileError::EmptyForkName);
        }
        if self.milliseconds_per_slot != self.seconds_per_slot.saturating_mul(1000) {
            return Err(ProfileError::InconsistentSlotMilliseconds);
        }
        if self.milliseconds_per_interval
            != self.milliseconds_per_slot / self.intervals_per_slot
        {
            return Err(ProfileError::InconsistentIntervalMilliseconds);
        }
        Ok(())
    }
}
