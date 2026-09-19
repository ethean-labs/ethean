//! Unit tests for the lstar fork-choice store.

use crate::{create_store, ForkChoiceError, ForkChoiceOpts, ForkChoiceStore};
use ethean_primitives::{Bytes52, Hash32, Slot, ValidatorIndex, HASH32_ZERO};
use ethean_profile::lstar_devnet;
use ethean_transition::{apply_block_unverified, process_slots, TransitionContext};
use ethean_types::{
    AttestationData, Block, BlockBody, BlockHeader, Checkpoint, GenesisConfig, State, Validator,
};

fn ctx() -> TransitionContext {
    TransitionContext::new(lstar_devnet().unwrap())
}

fn sample_state(validators: usize) -> State {
    let mut vals = Vec::new();
    for i in 0..validators {
        vals.push(
            Validator::new(Bytes52::ZERO, Bytes52::ZERO, ValidatorIndex::new(i as u64)).unwrap(),
        );
    }
    State {
        config: GenesisConfig::new(1_700_000_000),
        slot: Slot::ZERO,
        latest_block_header: BlockHeader::default(),
        latest_justified: Checkpoint::genesis(),
        latest_finalized: Checkpoint::genesis(),
        historical_block_hashes: Vec::new(),
        justified_slots: Vec::new(),
        validators: vals,
        justifications_roots: Vec::new(),
        justifications_validators: Vec::new(),
    }
}

fn empty_block(slot: u64, proposer: u64, parent: Hash32) -> Block {
    Block {
        slot: Slot::new(slot),
        proposer_index: ValidatorIndex::new(proposer),
        parent_root: parent,
        state_root: HASH32_ZERO,
        body: BlockBody::default(),
    }
}

fn make_anchor(validators: usize) -> (State, Block) {
    let mut state = sample_state(validators);
    let mut block = empty_block(0, 0, HASH32_ZERO);
    block.state_root = state.hash_tree_root().unwrap();
    state.latest_block_header = block.header().unwrap();
    state.latest_block_header.state_root = HASH32_ZERO;
    block.state_root = state.hash_tree_root().unwrap();
    (state, block)
}

fn anchor_store(validators: usize) -> (ForkChoiceStore, Hash32, State) {
    let profile = lstar_devnet().unwrap();
    let (state, block) = make_anchor(validators);
    let root = block.hash_tree_root().unwrap();
    let store = create_store(
        state.clone(),
        block,
        &profile,
        ForkChoiceOpts::STRUCTURAL,
    )
    .unwrap();
    (store, root, state)
}

fn import_child(
    store: &mut ForkChoiceStore,
    pre: &State,
    slot: u64,
    proposer: u64,
) -> (Hash32, State) {
    let mut advanced = pre.clone();
    process_slots(&mut advanced, Slot::new(slot)).unwrap();
    let parent_root = advanced.latest_block_header.hash_tree_root();
    let mut block = empty_block(slot, proposer, parent_root);
    let mut trial = pre.clone();
    process_slots(&mut trial, Slot::new(slot)).unwrap();
    ethean_transition::process_block(&mut trial, &block, &ctx()).unwrap();
    block.state_root = trial.hash_tree_root().unwrap();
    let out = apply_block_unverified(pre, &block, &ctx()).unwrap();
    let need = slot * store.intervals_per_slot;
    if store.time < need {
        store.on_tick_with(need, false).unwrap();
    }
    let root = block.hash_tree_root().unwrap();
    store.on_block(block, out.post_state.clone()).unwrap();
    (root, out.post_state)
}

#[test]
fn create_store_and_chain_advances_head() {
    let (mut store, anchor, state) = anchor_store(3);
    assert_eq!(store.head(), anchor);
    let (child, _) = import_child(&mut store, &state, 1, 1);
    assert_eq!(store.head(), child);
    assert!(store.blocks.contains_key(&child));
}

#[test]
fn unknown_parent_rejected() {
    let (mut store, _, _) = anchor_store(2);
    let orphan = empty_block(1, 0, [9u8; 32]);
    let err = store.on_block(orphan, sample_state(2)).unwrap_err();
    assert_eq!(err, ForkChoiceError::UnknownParent);
}

#[test]
fn tick_promotes_votes() {
    let (mut store, anchor, state) = anchor_store(4);
    let (child, _) = import_child(&mut store, &state, 1, 1);
    let data = AttestationData {
        slot: Slot::new(1),
        head: Checkpoint::new(child, Slot::new(1)),
        target: Checkpoint::new(child, Slot::new(1)),
        source: Checkpoint::new(anchor, Slot::new(0)),
    };
    store
        .on_attestation_data(ValidatorIndex::new(0), data)
        .unwrap();
    assert!(store.latest_known_attestations.is_empty());
    assert_eq!(store.latest_new_attestations.len(), 1);
    let slot_start = (store.time / store.intervals_per_slot + 1) * store.intervals_per_slot;
    store.on_tick_with(slot_start + 4, false).unwrap();
    assert!(store.latest_new_attestations.is_empty());
    assert_eq!(store.latest_known_attestations.len(), 1);
}

#[test]
fn tie_break_prefers_lexicographically_larger_root() {
    let (mut store, anchor, state) = anchor_store(3);
    store.on_tick_with(store.intervals_per_slot, false).unwrap();

    let mut lo_block = empty_block(1, 1, anchor);
    lo_block.state_root = [0x01; 32];
    let mut hi_block = empty_block(1, 2, anchor);
    hi_block.state_root = [0xfe; 32];

    let lo_root = lo_block.hash_tree_root().unwrap();
    let hi_root = hi_block.hash_tree_root().unwrap();
    assert_ne!(lo_root, hi_root);

    let mut post = state.clone();
    post.slot = Slot::new(1);
    post.latest_justified = store.justified();
    post.latest_finalized = store.finalized();

    // Insert smaller root first, then larger — head must still be max root.
    let (first, second) = if lo_root < hi_root {
        (lo_block, hi_block)
    } else {
        (hi_block, lo_block)
    };
    let first_root = first.hash_tree_root().unwrap();
    let second_root = second.hash_tree_root().unwrap();
    store.on_block(first, post.clone()).unwrap();
    store.on_block(second, post).unwrap();

    let expected = first_root.max(second_root);
    assert_eq!(store.head(), expected);
    assert_eq!(store.head(), store.head());
}

#[test]
fn require_proofs_rejects_attestation_data() {
    let profile = lstar_devnet().unwrap();
    let (state, block) = make_anchor(1);
    let mut store =
        create_store(state, block, &profile, ForkChoiceOpts::REQUIRE_PROOFS).unwrap();
    let err = store
        .on_attestation_data(ValidatorIndex::ZERO, AttestationData::default())
        .unwrap_err();
    assert!(matches!(err, ForkChoiceError::UnsupportedSignature(_)));
}
