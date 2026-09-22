use super::*;
use crate::local_proposer::LocalProposer;
use ethean_primitives::{Bytes52, Slot, HASH32_ZERO};
use ethean_profile::lstar_devnet;
use ethean_transition::process_slots;
use ethean_types::{BlockHeader, Checkpoint, GenesisConfig, State, Validator};

fn sample_state(validators: usize) -> State {
    let vals = (0..validators)
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
        validators: vals,
        justifications_roots: Vec::new(),
        justifications_validators: Vec::new(),
    }
}

fn owner_at_slot_one() -> ChainOwner {
    let pre = sample_state(3);
    let mut advanced = pre.clone();
    process_slots(&mut advanced, Slot::new(1)).unwrap();
    let parent = advanced.latest_block_header.hash_tree_root();
    assert_ne!(parent, HASH32_ZERO);
    let mut owner = ChainOwner::new(2);
    owner.head_root = parent;
    owner.head_state = Some(pre);
    owner.profile = Some(lstar_devnet().unwrap());
    owner.proposer = Some(LocalProposer::smoke().expect("proposer"));
    owner
}

fn tick(slot: u64) -> DutyTick {
    DutyTick {
        slot: Slot::new(slot),
        interval: 0,
        generation: 1,
    }
}

#[test]
fn never_gossips_a_block_without_a_verified_proof() {
    let mut owner = owner_at_slot_one();
    let events = try_plan_proposal(&mut owner, tick(1));
    assert!(matches!(
        events
            .iter()
            .find(|e| matches!(e, ChainEvent::ProposalPlanned { .. })),
        Some(ChainEvent::ProposalPlanned {
            publish_allowed: true,
            attestations: 0,
            ..
        })
    ));
    assert!(!events
        .iter()
        .any(|e| matches!(e, ChainEvent::ProposalGossipReady { .. })));
    assert!(
        owner.pending_block_gossip.is_none(),
        "no prover means no publish"
    );
    assert!(owner.planned_proposal.is_some());
}

#[test]
fn rejects_a_block_proof_that_does_not_verify() {
    let mut owner = owner_at_slot_one();
    try_plan_proposal(&mut owner, tick(1));
    let plan = owner.planned_proposal.clone().unwrap();
    let err = accept_block_proof(&mut owner, plan, vec![0u8; 64]).unwrap_err();
    assert!(err.contains("own block rejected"), "{err}");
    assert!(owner.pending_block_gossip.is_none());
}

#[test]
fn drops_stale_block_proofs() {
    let mut owner = owner_at_slot_one();
    try_plan_proposal(&mut owner, tick(1));
    let plan = owner.planned_proposal.clone().unwrap();
    owner.head_root = [7u8; 32];
    assert!(accept_block_proof(&mut owner, plan, vec![1])
        .unwrap_err()
        .contains("head moved"));
}

#[test]
fn skips_slots_owned_by_other_validators() {
    let mut owner = owner_at_slot_one();
    owner.owned_validator_indices = vec![0];
    assert!(try_plan_proposal(&mut owner, tick(1)).is_empty());
}
