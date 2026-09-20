//! Named built-in presets.

use crate::error::ProfileError;
use crate::profile::ChainProfile;

/// lstar devnet preset pinned to Phase 00 protocol-surface values.
pub fn lstar_devnet() -> Result<ChainProfile, ProfileError> {
    let profile = ChainProfile {
        seconds_per_slot: 4,
        intervals_per_slot: 5,
        milliseconds_per_slot: 4000,
        milliseconds_per_interval: 800,
        gossip_disparity_intervals: 1,
        justification_lookback_slots: 3,
        validator_registry_limit: 4096,
        historical_roots_limit: 262_144,
        attestation_committee_count: 1,
        max_attestations_data: 8,
        xmss_public_key_bytes: 52,
        xmss_signature_bytes: 2536,
        fork_name: "lstar",
    };
    profile.validate()?;
    Ok(profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lstar_devnet_validates() {
        let p = lstar_devnet().expect("lstar preset must validate");
        assert_eq!(p.seconds_per_slot, 4);
        assert_eq!(p.fork_name, "lstar");
        assert_eq!(p.xmss_public_key_bytes, 52);
        assert_eq!(p.xmss_signature_bytes, 2536);
    }

    #[test]
    fn rejects_zero_seconds() {
        let mut p = lstar_devnet().unwrap();
        // Bypass constructor: mutate then validate.
        p.seconds_per_slot = 0;
        assert_eq!(p.validate(), Err(ProfileError::ZeroSecondsPerSlot));
    }

    #[test]
    fn rejects_zero_max_attestations() {
        let mut p = lstar_devnet().unwrap();
        p.max_attestations_data = 0;
        assert_eq!(p.validate(), Err(ProfileError::ZeroMaxAttestationsData));
    }

    #[test]
    fn rejects_zero_intervals() {
        let mut p = lstar_devnet().unwrap();
        p.intervals_per_slot = 0;
        assert_eq!(p.validate(), Err(ProfileError::ZeroIntervalsPerSlot));
    }

    #[test]
    fn with_attestation_committee_count_overrides() {
        let p = lstar_devnet()
            .unwrap()
            .with_attestation_committee_count(4)
            .expect("count 4");
        assert_eq!(p.attestation_committee_count, 4);
        assert_eq!(p.attestation_subnet_count(), 4);
        assert!(lstar_devnet()
            .unwrap()
            .with_attestation_committee_count(0)
            .is_err());
    }
}
