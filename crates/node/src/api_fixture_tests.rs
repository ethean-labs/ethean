//! leanSpec `api_endpoint` suite through the `/lean/v0` handlers.

use ethean_genesis::GenesisBuilder;
use ethean_primitives::{Bytes52, Slot, ValidatorIndex, HASH32_ZERO};
use ethean_rpc::handlers::handle_route;
use ethean_rpc::{match_route, ForkChoiceView, SharedApiState};
use ethean_spec_fixtures::decode_hex_bytes;
use ethean_types::{Block, BlockBody, BlockHeader, MultiMessageAggregate, SignedBlock, State};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn fixtures() -> Option<PathBuf> {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.pop();
    dir.pop();
    dir.push(".cache/leanspec-fixtures/extracted/fixtures");
    if dir.is_dir() {
        Some(dir)
    } else {
        eprintln!("skip: fixture archive not extracted");
        None
    }
}

fn json_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let p = entry.unwrap().path();
        if p.is_dir() {
            out.extend(json_files(&p));
        } else if p.extension().is_some_and(|e| e == "json") {
            out.push(p);
        }
    }
    out.sort();
    out
}

fn key_pairs(root: &Path, n: usize) -> Vec<(Bytes52, Bytes52)> {
    (0..n)
        .map(|i| {
            let v: Value = serde_json::from_slice(
                &std::fs::read(root.join(format!("keys/prod_scheme/{i}.json"))).unwrap(),
            )
            .unwrap();
            let pk = |role: &str| {
                let hex = v.pointer(&format!("/{role}_keypair/public_key")).unwrap();
                Bytes52::from_slice(&decode_hex_bytes(hex.as_str().unwrap()).unwrap()).unwrap()
            };
            (pk("attestation"), pk("proposal"))
        })
        .collect()
}

/// The anchor state for `genesisParams`: a fresh genesis, or the `sync`
/// suite's served state for an advanced anchor slot.
fn anchor_state(root: &Path, params: &Value) -> State {
    let n = params["numValidators"].as_u64().unwrap() as usize;
    let anchor = params
        .get("anchorSlot")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if anchor == 0 {
        let t = params["genesisTime"].as_u64().unwrap();
        return GenesisBuilder::new(t)
            .with_validator_keys(key_pairs(root, n))
            .build()
            .unwrap()
            .state;
    }
    for file in json_files(&root.join("consensus/sync")) {
        let doc: serde_json::Map<String, Value> =
            serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
        for (_, case) in doc {
            let op = &case["operation"];
            if op["numValidators"].as_u64() == Some(n as u64)
                && op["anchorSlot"].as_u64() == Some(anchor)
            {
                let bytes =
                    decode_hex_bytes(case["output"]["stateBytes"].as_str().unwrap()).unwrap();
                return State::ssz_decode(&bytes).unwrap();
            }
        }
    }
    panic!("no sync fixture state for {n} validators at slot {anchor}");
}

/// Anchor block: the state's latest header with the state root filled in.
fn anchor_block(state: &State) -> SignedBlock {
    let h: &BlockHeader = &state.latest_block_header;
    let block = Block {
        slot: h.slot,
        proposer_index: h.proposer_index,
        parent_root: h.parent_root,
        state_root: state.hash_tree_root().unwrap(),
        body: BlockBody::default(),
    };
    SignedBlock::new(block, MultiMessageAggregate::default())
}

fn view_for(state: &State) -> (ForkChoiceView, [u8; 32]) {
    let signed = anchor_block(state);
    let header_root = BlockHeader {
        slot: signed.block.slot,
        proposer_index: signed.block.proposer_index,
        parent_root: signed.block.parent_root,
        state_root: signed.block.state_root,
        body_root: state.latest_block_header.body_root,
    }
    .hash_tree_root();
    let mut view = ForkChoiceView::genesis(
        header_root,
        state.validators.len() as u64,
        state.ssz_encode().unwrap(),
        signed.ssz_encode().unwrap(),
    );
    let node = &mut view.fork_choice.nodes[0];
    node.slot = state.latest_block_header.slot.get();
    node.parent_root = state.latest_block_header.parent_root;
    node.proposer_index = state.latest_block_header.proposer_index.get();
    view.fork_choice.justified.slot = node.slot;
    view.fork_choice.finalized.slot = node.slot;
    (view, header_root)
}

#[test]
fn api_endpoint_fixtures_when_present() {
    let Some(root) = fixtures() else {
        return;
    };
    let mut checked = 0;
    for file in json_files(&root.join("consensus/api_endpoint")) {
        let doc: serde_json::Map<String, Value> =
            serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
        for (id, case) in &doc {
            let endpoint = case["endpoint"].as_str().unwrap();
            let method = case["method"].as_str().unwrap();
            let want_status = case["expectedStatusCode"].as_u64().unwrap() as u16;
            let want_type = case["expectedContentType"].as_str().unwrap();
            let expected = &case["expectedBody"];
            if endpoint == "/metrics" {
                let text = ethean_metrics::lean::export_lean_text();
                for name in expected["required_metric_names"].as_array().unwrap() {
                    let name = name.as_str().unwrap();
                    assert!(
                        text.contains(&format!("# TYPE {name} ")),
                        "{id}: {name} missing"
                    );
                }
                checked += 1;
                continue;
            }
            let state = anchor_state(&root, &case["genesisParams"]);
            let (view, _) = view_for(&state);
            let api = SharedApiState::new("");
            let mut snap = api.snapshot();
            snap.fork_choice = view;
            api.publish(snap);
            api.set_ready(true);
            api.set_aggregator(case["initialIsAggregator"].as_bool().unwrap_or(false));
            let body = case
                .get("requestBody")
                .filter(|v| !v.is_null())
                .map(|v| v.to_string())
                .unwrap_or_default();
            let route = match_route(method, endpoint).unwrap_or_else(|e| panic!("{id}: {e}"));
            let reply = handle_route(route, &api, body.as_bytes());
            assert_eq!(reply.status, want_status, "{id}: status");
            assert_eq!(reply.content_type, want_type, "{id}: content type");
            if want_type == "application/octet-stream" {
                let want = decode_hex_bytes(expected.as_str().unwrap()).unwrap();
                assert_eq!(reply.body, want, "{id}: SSZ body");
            } else {
                let got: Value = serde_json::from_slice(&reply.body).unwrap();
                assert_eq!(&got, expected, "{id}: JSON body");
            }
            checked += 1;
        }
    }
    eprintln!("api_endpoint: {checked} cases");
    assert!(checked > 0);
}

#[test]
fn anchor_block_root_matches_genesis_header() {
    let state = GenesisBuilder::new(0)
        .with_validator_keys(vec![(Bytes52::ZERO, Bytes52::ZERO)])
        .build()
        .unwrap()
        .state;
    let (view, root) = view_for(&state);
    assert_eq!(view.fork_choice.head, root);
    assert_eq!(view.fork_choice.nodes[0].slot, Slot::ZERO.get());
    assert_eq!(view.fork_choice.nodes[0].parent_root, HASH32_ZERO);
    assert_eq!(
        view.fork_choice.nodes[0].proposer_index,
        ValidatorIndex::new(0).get()
    );
}
