//! Profile validation entry points.

use crate::error::ProfileError;
use crate::profile::ChainProfile;

/// Validate a profile (same rules as [`ChainProfile::validate`]).
pub fn validate_profile(profile: &ChainProfile) -> Result<(), ProfileError> {
    profile.validate()
}

/// Ensure the profile fork name matches the pinned lstar string.
pub fn require_lstar_fork(profile: &ChainProfile) -> Result<(), ProfileError> {
    profile.validate()?;
    if profile.fork_name != "lstar" {
        return Err(ProfileError::ForkNameMismatch {
            expected: "lstar",
            got: profile.fork_name.to_string(),
        });
    }
    Ok(())
}
