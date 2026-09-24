//! Drive every extracted leanSpec fixture the way hive's spec-asset suites do.

use super::*;
use crate::discover::{discover_json_fixtures, fixtures_root_from_env};
use crate::FIXTURES_ENV;
use std::path::PathBuf;

fn root() -> Option<PathBuf> {
    if fixtures_root_from_env().is_none() {
        let mut def = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        def.pop();
        def.pop();
        def.push(".cache");
        def.push("leanspec-fixtures");
        def.push("extracted");
        if def.is_dir() {
            std::env::set_var(FIXTURES_ENV, &def);
        }
    }
    let r = fixtures_root_from_env();
    if r.is_none() {
        eprintln!("skip: fixture archive not extracted");
    }
    r
}

fn files(suite: &str) -> Vec<PathBuf> {
    let Some(root) = root() else {
        return Vec::new();
    };
    discover_json_fixtures(&root)
        .unwrap()
        .into_iter()
        .filter(|p| {
            p.to_string_lossy()
                .contains(&format!("/consensus/{suite}/"))
        })
        .collect()
}

fn norm(v: &Value) -> String {
    v.as_str()
        .unwrap_or("")
        .trim_start_matches("0x")
        .to_ascii_lowercase()
}

fn check_snapshot(snap: &Value, checks: &Value, ctx: &str) {
    for (key, ptr) in [
        ("headSlot", "/headSlot"),
        ("time", "/time"),
        ("justifiedCheckpoint/slot", "/justifiedCheckpoint/slot"),
        ("finalizedCheckpoint/slot", "/finalizedCheckpoint/slot"),
    ] {
        if let Some(want) = checks.pointer(ptr).and_then(Value::as_u64) {
            assert_eq!(
                snap.pointer(ptr).and_then(Value::as_u64),
                Some(want),
                "{ctx}: {key}"
            );
        }
    }
    for (key, ptr) in [
        ("headRoot", "/headRoot"),
        ("justifiedCheckpoint/root", "/justifiedCheckpoint/root"),
        ("finalizedCheckpoint/root", "/finalizedCheckpoint/root"),
        ("safeTarget", "/safeTarget"),
    ] {
        if let Some(want) = checks.pointer(ptr) {
            let got = snap.pointer(ptr).cloned().unwrap_or(Value::Null);
            assert_eq!(norm(&got), norm(want), "{ctx}: {key}");
        }
    }
}

#[test]
fn every_fork_choice_fixture_drives_like_hive() {
    let mut cases = 0;
    let mut steps = 0;
    for file in files("fork_choice") {
        let raw = std::fs::read(&file).unwrap();
        let doc: serde_json::Map<String, Value> = serde_json::from_slice(&raw).unwrap();
        for (id, case) in &doc {
            let ctx = format!("{}::{id}", file.display());
            let genesis = case
                .pointer("/anchorState/config/genesisTime")
                .and_then(Value::as_u64);
            let init = init_driver_store(&case["anchorState"], &case["anchorBlock"], genesis, None);
            let fixture_steps = case
                .get("steps")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let expects_init_failure = fixture_steps.is_empty()
                && (case.get("expectException").is_some()
                    || case.get("rejectionReason").is_some()
                    || case
                        .pointer("/_info/description")
                        .and_then(Value::as_str)
                        .is_some_and(|d| d.contains("anchor_valid=False")));
            let Ok(mut d) = init else {
                assert!(
                    expects_init_failure,
                    "{ctx}: init failed unexpectedly: {:?}",
                    init.err()
                );
                cases += 1;
                continue;
            };
            assert!(!expects_init_failure, "{ctx}: invalid anchor was accepted");
            for (i, step) in fixture_steps.iter().enumerate() {
                if step.get("rejectionReason").and_then(Value::as_str) == Some("INVALID_SIGNATURE")
                {
                    // Needs the node's XMSS / leanMultisig verifier (node test, release).
                    continue;
                }
                let result = apply_driver_step(&mut d, step);
                let snap = driver_snapshot(&d);
                if let Some(valid) = step.get("valid").and_then(Value::as_bool) {
                    assert_eq!(
                        result.is_ok(),
                        valid,
                        "{ctx}: step {i} acceptance, error {:?}",
                        result.err()
                    );
                }
                if let Some(checks) = step.get("checks") {
                    check_snapshot(&snap, checks, &format!("{ctx}: step {i}"));
                }
                steps += 1;
            }
            cases += 1;
        }
    }
    eprintln!("fork_choice driver: {cases} cases, {steps} steps");
}

#[test]
fn every_state_transition_fixture_drives_like_hive() {
    let mut cases = 0;
    for file in files("state_transition") {
        let raw = std::fs::read(&file).unwrap();
        let doc: serde_json::Map<String, Value> = serde_json::from_slice(&raw).unwrap();
        for (id, case) in &doc {
            let ctx = format!("{}::{id}", file.display());
            let resp = run_state_transition_driver(case);
            let expect_reject = case
                .get("expectException")
                .or_else(|| case.get("rejectionReason"))
                .is_some();
            assert_eq!(
                resp["succeeded"].as_bool(),
                Some(!expect_reject),
                "{ctx}: {resp}"
            );
            if let Some(post) = case.get("post") {
                let got = &resp["post"];
                for key in [
                    "slot",
                    "latestBlockHeaderSlot",
                    "historicalBlockHashesCount",
                ] {
                    if let Some(want) = post.get(key).and_then(Value::as_u64) {
                        assert_eq!(got[key].as_u64(), Some(want), "{ctx}: post.{key}");
                    }
                }
                if let Some(want) = post.get("latestBlockHeaderStateRoot") {
                    assert_eq!(
                        norm(&got["latestBlockHeaderStateRoot"]),
                        norm(want),
                        "{ctx}: post root"
                    );
                }
            }
            cases += 1;
        }
    }
    eprintln!("state_transition driver: {cases} cases");
}

#[test]
fn hex_decoder_roundtrips() {
    assert_eq!(decode_hex_bytes("0x00ff10").unwrap(), vec![0, 255, 16]);
    assert!(decode_hex_bytes("0xabc").is_err());
}
