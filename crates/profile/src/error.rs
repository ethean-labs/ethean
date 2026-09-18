//! Profile validation errors.

use thiserror::Error;

/// Errors from `ChainProfile::validate`.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProfileError {
    #[error("seconds_per_slot must be non-zero")]
    ZeroSecondsPerSlot,

    #[error("intervals_per_slot must be non-zero")]
    ZeroIntervalsPerSlot,

    #[error("milliseconds_per_slot must be non-zero")]
    ZeroMillisecondsPerSlot,

    #[error("milliseconds_per_interval must be non-zero")]
    ZeroMillisecondsPerInterval,

    #[error("max_attestations_data must be non-zero")]
    ZeroMaxAttestationsData,

    #[error("validator_registry_limit must be non-zero")]
    ZeroValidatorRegistryLimit,

    #[error("historical_roots_limit must be non-zero")]
    ZeroHistoricalRootsLimit,

    #[error("attestation_committee_count must be non-zero")]
    ZeroAttestationCommitteeCount,

    #[error("xmss_public_key_bytes must be non-zero")]
    ZeroXmssPublicKeyBytes,

    #[error("xmss_signature_bytes must be non-zero")]
    ZeroXmssSignatureBytes,

    #[error("fork_name must be non-empty")]
    EmptyForkName,

    #[error("milliseconds_per_slot must equal seconds_per_slot * 1000")]
    InconsistentSlotMilliseconds,

    #[error("milliseconds_per_interval must equal milliseconds_per_slot / intervals_per_slot")]
    InconsistentIntervalMilliseconds,
}
