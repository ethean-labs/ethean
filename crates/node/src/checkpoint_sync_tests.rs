use super::*;
use ethean_genesis::local_smoke_genesis;
use ethean_primitives::{Slot, ValidatorIndex, HASH32_ZERO};
use ethean_rpc::{spawn_lean_http, ForkChoiceView, SharedApiState};
use ethean_types::{Block, BlockBody, MultiMessageAggregate};

fn genesis_pair() -> (State, SignedBlock) {
    let state = local_smoke_genesis(1_700_000_000).unwrap().state;
    let block = Block {
        slot: Slot::ZERO,
        proposer_index: ValidatorIndex::new(0),
        parent_root: HASH32_ZERO,
        state_root: state.hash_tree_root().unwrap(),
        body: BlockBody::default(),
    };
    (
        state,
        SignedBlock::new(block, MultiMessageAggregate::default()),
    )
}

#[test]
fn url_derivation_handles_state_block_and_base_forms() {
    assert_eq!(
        blocks_url("http://h:5052/lean/v0/states/finalized").unwrap(),
        "http://h:5052/lean/v0/blocks/finalized"
    );
    assert_eq!(
        states_url("http://h:5052"),
        "http://h:5052/lean/v0/states/finalized"
    );
    assert_eq!(
        states_url("http://h/lean/v0/blocks/finalized"),
        "http://h/lean/v0/states/finalized"
    );
}

#[test]
fn pair_verification_rejects_mismatches() {
    let (state, signed) = genesis_pair();
    let s = state.ssz_encode().unwrap();
    let b = signed.ssz_encode().unwrap();
    let anchor = verify_pair(&s, &b).unwrap();
    assert_eq!(anchor.state, state);
    assert_eq!(anchor.block_root, signed.block.hash_tree_root().unwrap());

    let mut wrong = signed.clone();
    wrong.block.state_root = [9u8; 32];
    assert!(verify_pair(&s, &wrong.ssz_encode().unwrap()).is_err());
    let mut wrong = signed.clone();
    wrong.block.slot = Slot::new(3);
    assert!(verify_pair(&s, &wrong.ssz_encode().unwrap()).is_err());
    assert!(verify_pair(&s[..10], &b).is_err());
}

#[test]
fn anchor_replaces_an_empty_local_chain() {
    let (state, signed) = genesis_pair();
    let bytes = signed.ssz_encode().unwrap();
    let anchor = verify_pair(&state.ssz_encode().unwrap(), &bytes).unwrap();
    let mut owner = ChainOwner::new(2);
    assert!(apply_anchor(&mut owner, anchor.clone()));
    assert_eq!(owner.head_root, anchor.block_root);
    assert_eq!(owner.head_state.as_ref().unwrap().slot, Slot::ZERO);
    assert!(owner.known_block_roots().contains(&anchor.block_root));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fetches_the_pair_from_a_lean_api() {
    let (state, signed) = genesis_pair();
    let s = state.ssz_encode().unwrap();
    let b = signed.ssz_encode().unwrap();
    let root = signed.block.hash_tree_root().unwrap();
    let api = SharedApiState::new("");
    let mut snap = api.snapshot();
    snap.fork_choice = ForkChoiceView::genesis(root, state.validators.len() as u64, s, b);
    api.publish(snap);
    api.set_ready(true);
    let addr = spawn_lean_http("127.0.0.1:0".parse().unwrap(), api)
        .await
        .unwrap();
    let url = format!("http://{addr}/lean/v0/states/finalized");
    let anchor = tokio::task::spawn_blocking(move || fetch_checkpoint(&url))
        .await
        .unwrap()
        .expect("checkpoint fetched");
    assert_eq!(anchor.block_root, root);
    assert_eq!(anchor.state, state);
}
