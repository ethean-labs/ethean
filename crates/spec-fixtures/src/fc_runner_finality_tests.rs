//! Finality / reorg / LMD leanSpec FC vectors (cache-gated).

use super::*;
use crate::discover::fixtures_root_from_env;
use crate::FIXTURES_ENV;
use std::path::PathBuf;

fn ensure_cache_env() {
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
}

fn fixture(rel: &str) -> Option<PathBuf> {
    ensure_cache_env();
    let root = fixtures_root_from_env()?;
    let mut p = root.join("fixtures").join("consensus").join("fork_choice");
    p.push("lstar");
    p.push("fork_choice");
    for part in rel.split('/') {
        p.push(part);
    }
    p.is_file().then_some(p)
}

fn run_ok(rel: &str) -> Option<FcRunReport> {
    let path = fixture(rel)?;
    let bytes = std::fs::read(&path).unwrap();
    let reports = run_fork_choice_file(&bytes).unwrap();
    Some(reports[0].1.clone())
}

#[test]
fn lmd_higher_slot_vote_replaces_lower() {
    let Some(r) = run_ok(
        "test_lmd_latest_message/test_higher_slot_vote_replaces_lower_slot_vote.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 4);
    assert_eq!(r.attestations, 2);
    assert_eq!(r.rejections, 0);
    assert!(r.assertions >= 1);
}

#[test]
fn lmd_lexicographic_tiebreak_stable() {
    let Some(r) = run_ok(
        "test_lmd_latest_message/test_lexicographic_tiebreak_selects_larger_root_and_is_stable.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 5);
    assert_eq!(r.rejections, 0);
}

#[test]
fn simple_one_block_reorg() {
    let Some(r) = run_ok("test_fork_choice_reorgs/test_simple_one_block_reorg.json") else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 4);
    assert_eq!(r.rejections, 0);
}

#[test]
fn two_block_reorg_progressive() {
    let Some(r) =
        run_ok("test_fork_choice_reorgs/test_two_block_reorg_progressive_building.json")
    else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 6);
    assert_eq!(r.rejections, 0);
}

#[test]
fn three_block_deep_reorg() {
    let Some(r) = run_ok("test_fork_choice_reorgs/test_three_block_deep_reorg.json") else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 6);
    assert_eq!(r.rejections, 0);
}

#[test]
fn finalization_advances_mid_attestation_processing() {
    let Some(r) = run_ok(
        "test_finalization_mid_processing/test_finalization_advances_mid_attestation_processing.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 8);
    assert_eq!(r.rejections, 0);
}

#[test]
fn head_switches_to_heavier_fork() {
    let Some(r) = run_ok("test_fork_choice_head/test_head_switches_to_heavier_fork.json") else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 4);
    assert_eq!(r.rejections, 0);
}

#[test]
fn head_selection_by_weight_not_depth() {
    let Some(r) = run_ok("test_fork_choice_head/test_head_selection_by_weight_not_depth.json")
    else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 8);
    assert_eq!(r.rejections, 0);
}

#[test]
fn justifies_reanchors_within_one_import() {
    let Some(r) = run_ok(
        "test_head_movement/test_block_that_justifies_reanchors_within_one_import.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 5);
    assert_eq!(r.rejections, 0);
}

#[test]
fn duplicate_block_processed_idempotently() {
    let Some(r) =
        run_ok("test_fork_choice_head/test_duplicate_block_processed_idempotently.json")
    else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 3);
    assert_eq!(r.rejections, 0);
}
