use super::*;
use ethean_primitives::Bytes52;
use ethean_profile::lstar_devnet;
use ethean_types::{BlockHeader, GenesisConfig, Validator};

fn genesis_state(n: usize) -> State {
    let validators = (0..n)
        .map(|i| {
            Validator::new(Bytes52::ZERO, Bytes52::ZERO, ValidatorIndex::new(i as u64)).unwrap()
        })
        .collect();
    State {
        config: GenesisConfig::new(1_700_000_000),
        slot: Slot::ZERO,
        latest_block_header: BlockHeader::default(),
        latest_justified: Checkpoint::genesis(),
        latest_finalized: Checkpoint::genesis(),
        historical_block_hashes: Vec::new(),
        justified_slots: Vec::new(),
        validators,
        justifications_roots: Vec::new(),
        justifications_validators: Vec::new(),
    }
}

fn parent_of(pre: &State) -> Hash32 {
    let mut advanced = pre.clone();
    process_slots(&mut advanced, Slot::new(1)).unwrap();
    advanced.latest_block_header.hash_tree_root()
}

fn vote(root: Hash32, slot: u64) -> AttestationData {
    let cp = Checkpoint::new(root, Slot::ZERO);
    AttestationData {
        slot: Slot::new(slot),
        head: cp,
        target: cp,
        source: cp,
    }
}

fn variant(bits: &[bool], tag: u8) -> ProofVariant {
    ProofVariant {
        bits: bits.to_vec(),
        proof: vec![tag; 4],
    }
}

#[test]
fn genesis_self_vote_is_included_with_the_widest_variant() {
    let pre = genesis_state(4);
    let parent = parent_of(&pre);
    let known: HashSet<Hash32> = [parent].into_iter().collect();
    let candidates = vec![(
        vote(parent, 1),
        vec![
            variant(&[true, false, false, false], 1),
            variant(&[true, true, true, false], 2),
        ],
    )];
    let body = select_body(
        &candidates,
        &pre,
        Slot::new(1),
        ValidatorIndex::new(1),
        parent,
        &known,
        lstar_devnet().unwrap(),
    )
    .unwrap();
    assert_eq!(body.attestations.len(), 1);
    assert_eq!(
        body.attestations[0].aggregation_bits.bits,
        vec![true, true, true, false]
    );
    assert_eq!(body.proofs, vec![vec![2u8; 4]]);
}

#[test]
fn unknown_head_and_wrong_source_are_skipped() {
    let pre = genesis_state(4);
    let parent = parent_of(&pre);
    let known: HashSet<Hash32> = [parent].into_iter().collect();
    let unknown_head = vote([7u8; 32], 1);
    let mut wrong_source = vote(parent, 1);
    wrong_source.source = Checkpoint::new(parent, Slot::new(1));
    let candidates = vec![
        (unknown_head, vec![variant(&[true], 1)]),
        (wrong_source, vec![variant(&[true], 2)]),
    ];
    let body = select_body(
        &candidates,
        &pre,
        Slot::new(1),
        ValidatorIndex::new(1),
        parent,
        &known,
        lstar_devnet().unwrap(),
    )
    .unwrap();
    assert!(body.attestations.is_empty());
}

#[test]
fn off_chain_target_is_skipped() {
    let pre = genesis_state(4);
    let parent = parent_of(&pre);
    let known: HashSet<Hash32> = [parent, [9u8; 32]].into_iter().collect();
    let mut off_chain = vote(parent, 1);
    off_chain.target = Checkpoint::new([9u8; 32], Slot::ZERO);
    let body = select_body(
        &[(off_chain, vec![variant(&[true], 1)])],
        &pre,
        Slot::new(1),
        ValidatorIndex::new(1),
        parent,
        &known,
        lstar_devnet().unwrap(),
    )
    .unwrap();
    assert!(body.attestations.is_empty());
}

#[test]
fn empty_candidates_give_an_empty_body() {
    let pre = genesis_state(2);
    let parent = parent_of(&pre);
    let body = select_body(
        &[],
        &pre,
        Slot::new(1),
        ValidatorIndex::new(1),
        parent,
        &HashSet::new(),
        lstar_devnet().unwrap(),
    )
    .unwrap();
    assert!(body.attestations.is_empty() && body.proofs.is_empty());
}

#[test]
fn chain_view_appends_parent_and_skipped_slots() {
    let pre = genesis_state(1);
    let view = extended_chain_view(&pre, [1u8; 32], Slot::new(3));
    assert_eq!(view, vec![[1u8; 32], HASH32_ZERO, HASH32_ZERO]);
}
