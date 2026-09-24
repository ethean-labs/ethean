//! Canonical finalized SSZ pair for `/lean/v0/states|blocks/finalized`.

use ethean_primitives::{Hash32, Slot, ValidatorIndex, HASH32_ZERO};
use ethean_types::{Block, BlockBody, MultiMessageAggregate, SignedBlock, State};

/// Copy of `state` with `latest_block_header.state_root` zeroed.
///
/// Matches ethlambda's served post-state: `process_block_header` leaves the
/// field zero, and `process_slots` fills it only on the next slot.
pub fn canonical_state(state: &State) -> State {
    let mut out = state.clone();
    out.latest_block_header.state_root = HASH32_ZERO;
    out
}

/// SSZ of the canonical (zeroed-header) state.
pub fn canonical_state_ssz(state: &State) -> Result<Vec<u8>, String> {
    canonical_state(state)
        .ssz_encode()
        .map_err(|e| e.to_string())
}

/// Synthesize the genesis `SignedBlock` (empty body, blank proof) paired with
/// the canonical genesis state.
///
/// Hive pairing: `block.slot == state.slot`, `block.state_root ==
/// hash_tree_root(canonical_state)`, `hash_tree_root(block) ==
/// fork_choice.finalized.root`. When the sealed header's `body_root` is the
/// empty-body tree hash, that last equality holds against `owner.head_root`.
pub fn genesis_signed_pair(state: &State) -> Result<(SignedBlock, State, Hash32), String> {
    let canonical = canonical_state(state);
    let state_root = canonical.hash_tree_root().map_err(|e| e.to_string())?;
    let empty_body = BlockBody::default();
    let empty_body_root = empty_body.hash_tree_root().map_err(|e| e.to_string())?;
    let header_body_root = canonical.latest_block_header.body_root;
    let body = if header_body_root == HASH32_ZERO || header_body_root == empty_body_root {
        empty_body
    } else {
        empty_body
    };
    let block = Block {
        slot: Slot::ZERO,
        proposer_index: ValidatorIndex::ZERO,
        parent_root: HASH32_ZERO,
        state_root,
        body,
    };
    let root = block.hash_tree_root().map_err(|e| e.to_string())?;
    Ok((
        SignedBlock::new(block, MultiMessageAggregate::default()),
        canonical,
        root,
    ))
}

/// Encode a signed block to SSZ.
pub fn signed_block_ssz(signed: &SignedBlock) -> Result<Vec<u8>, String> {
    signed.ssz_encode().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Bytes52, ValidatorIndex};
    use ethean_types::{BlockHeader, Checkpoint, GenesisConfig, Validator};

    fn sample_genesis() -> State {
        State {
            config: GenesisConfig::new(1_700_000_000),
            slot: Slot::ZERO,
            latest_block_header: BlockHeader::default(),
            latest_justified: Checkpoint::genesis(),
            latest_finalized: Checkpoint::genesis(),
            historical_block_hashes: Vec::new(),
            justified_slots: Vec::new(),
            validators: vec![
                Validator::new(Bytes52::ZERO, Bytes52::ZERO, ValidatorIndex::new(0)).unwrap(),
            ],
            justifications_roots: Vec::new(),
            justifications_validators: Vec::new(),
        }
    }

    #[test]
    fn genesis_pair_invariants() {
        let state = sample_genesis();
        let (signed, canonical, root) = genesis_signed_pair(&state).unwrap();
        assert_eq!(signed.block.slot, canonical.slot);
        assert_eq!(
            signed.block.state_root,
            canonical.hash_tree_root().unwrap()
        );
        assert_eq!(signed.block.hash_tree_root().unwrap(), root);
        assert!(signed.proof.proof.is_empty());
        assert_eq!(canonical.latest_block_header.state_root, HASH32_ZERO);
    }
}
