//! Load genesis state from SSZ bytes.

use ethean_ssz::Root;
use ethean_types::State;

use crate::error::GenesisError;

/// Decode genesis from SSZ and optionally verify the state root.
///
/// Uses full [`State::ssz_decode`] (same field order as ethlambda / Ream Lean).
pub fn load_genesis_ssz(
    bytes: &[u8],
    expected_root: Option<&Root>,
) -> Result<State, GenesisError> {
    if bytes.is_empty() {
        return Err(GenesisError::TruncatedOrEmpty);
    }

    let state = State::ssz_decode(bytes).map_err(|e| GenesisError::Types(e.to_string()))?;

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
    use crate::GenesisBuilder;
    use ethean_primitives::Bytes52;

    #[test]
    fn rejects_empty_bytes() {
        assert_eq!(
            load_genesis_ssz(&[], None).unwrap_err(),
            GenesisError::TruncatedOrEmpty
        );
    }

    #[test]
    fn rejects_garbage_bytes() {
        let err = load_genesis_ssz(&[1, 2, 3, 4], None).unwrap_err();
        assert!(matches!(
            err,
            GenesisError::TruncatedOrEmpty | GenesisError::Types(_)
        ));
    }

    #[test]
    fn roundtrip_builder_genesis() {
        let built = GenesisBuilder::new(1_700_000_000)
            .push_validator(Bytes52::ZERO, Bytes52::ZERO)
            .push_validator(Bytes52([1u8; 52]), Bytes52([2u8; 52]))
            .build()
            .unwrap();
        let enc = built.state.ssz_encode().unwrap();
        let loaded = load_genesis_ssz(&enc, Some(&built.state_root)).unwrap();
        assert_eq!(loaded.validators.len(), 2);
        assert_eq!(loaded.hash_tree_root().unwrap(), built.state_root);
    }
}
