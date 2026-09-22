//! Resolve validator public keys from a state registry, bounds-checked.

use ethean_crypto::PublicKey;
use ethean_types::State;

/// Attestation key of `index`, or an error when outside the registry.
pub fn attestation_key(state: &State, index: u64) -> Result<PublicKey, String> {
    let validator = state
        .validators
        .get(index as usize)
        .ok_or_else(|| format!("validator {index} not in registry"))?;
    PublicKey::try_from_slice(validator.attestation_public_key.as_bytes())
        .map_err(|e| e.to_string())
}

/// Proposal key of `index`, or an error when outside the registry.
pub fn proposal_key(state: &State, index: u64) -> Result<PublicKey, String> {
    let validator = state
        .validators
        .get(index as usize)
        .ok_or_else(|| format!("validator {index} not in registry"))?;
    PublicKey::try_from_slice(validator.proposal_public_key.as_bytes()).map_err(|e| e.to_string())
}

/// Attestation keys of every set bit, in validator-index order.
pub fn attestation_keys_for_bits(state: &State, bits: &[bool]) -> Result<Vec<PublicKey>, String> {
    ethean_transition::participant_indices(bits)
        .into_iter()
        .map(|i| attestation_key(state, i))
        .collect()
}
