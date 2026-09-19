//! Load genesis state from SSZ bytes.

use ethean_ssz::Root;
use ethean_types::State;

use crate::error::GenesisError;

/// Decode genesis from SSZ and optionally verify the state root.
///
/// Rejects empty/truncated payloads. Full `State::ssz_decode` for non-trivial
/// containers remains incomplete until Phase 05; loaders still refuse silent
/// `State::default()` substitution.
pub fn load_genesis_ssz(
    bytes: &[u8],
    expected_root: Option<&Root>,
) -> Result<State, GenesisError> {
    if bytes.is_empty() {
        return Err(GenesisError::TruncatedOrEmpty);
    }

    let state = State::ssz_decode(bytes).map_err(|e| {
        // Incomplete decode path surfaces as Types / Profile errors today.
        let msg = e.to_string();
        if msg.contains("deferred") || msg.contains("truncated") {
            GenesisError::TruncatedOrEmpty
        } else {
            GenesisError::Types(msg)
        }
    })?;

    if state.validators.is_empty() {
        return Err(GenesisError::EmptyValidators);
    }

    if let Some(expected) = expected_root {
        let got = state
            .hash_tree_root()
            .map_err(|e| GenesisError::Types(e.to_string()))?;
        if &got != expected {
            return Err(GenesisError::GenesisRootMismatch);
        }
    }

    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_bytes() {
        assert_eq!(
            load_genesis_ssz(&[], None).unwrap_err(),
            GenesisError::TruncatedOrEmpty
        );
    }

    #[test]
    fn rejects_nonempty_until_full_decode() {
        // Non-empty bytes currently hit the deferred decode path.
        let err = load_genesis_ssz(&[1, 2, 3, 4], None).unwrap_err();
        assert!(matches!(
            err,
            GenesisError::TruncatedOrEmpty | GenesisError::Types(_)
        ));
    }
}
