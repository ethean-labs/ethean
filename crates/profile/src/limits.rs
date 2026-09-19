//! Limit helpers derived from a [`ChainProfile`].

use crate::profile::ChainProfile;

/// Snapshot of registry / attestation bounds from a profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileLimits {
    pub validator_registry_limit: u64,
    pub historical_roots_limit: u64,
    pub max_attestations_data: u64,
    pub attestation_committee_count: u64,
    pub xmss_public_key_bytes: u64,
    pub xmss_signature_bytes: u64,
}

impl ProfileLimits {
    pub fn from_profile(profile: &ChainProfile) -> Self {
        Self {
            validator_registry_limit: profile.validator_registry_limit,
            historical_roots_limit: profile.historical_roots_limit,
            max_attestations_data: profile.max_attestations_data,
            attestation_committee_count: profile.attestation_committee_count,
            xmss_public_key_bytes: profile.xmss_public_key_bytes,
            xmss_signature_bytes: profile.xmss_signature_bytes,
        }
    }
}

/// Convenience: limits for the given profile.
pub fn limits_of(profile: &ChainProfile) -> ProfileLimits {
    ProfileLimits::from_profile(profile)
}
