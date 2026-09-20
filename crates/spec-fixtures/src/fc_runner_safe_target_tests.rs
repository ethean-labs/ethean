//! Safe-target / head-movement / store-prune leanSpec FC vectors.

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
fn safe_target_holds_below_supermajority() {
    let Some(r) = run_ok(
        "test_safe_target/test_safe_target_does_not_advance_below_supermajority.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 2);
    assert_eq!(r.attestations, 1);
    assert_eq!(r.rejections, 0);
}

#[test]
fn safe_target_advances_incrementally() {
    let Some(r) = run_ok(
        "test_safe_target/test_safe_target_advances_incrementally_along_the_chain.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 3);
    assert_eq!(r.attestations, 3);
    assert_eq!(r.rejections, 0);
}

#[test]
fn supermajority_four_of_five_advances() {
    let Some(r) = run_ok(
        "test_safe_target_supermajority/test_odd_five_validators_four_votes_advance_safe_target.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 2);
    assert_eq!(r.attestations, 1);
}

#[test]
fn supermajority_three_of_five_holds_genesis() {
    let Some(r) = run_ok(
        "test_safe_target_supermajority/test_odd_five_validators_three_votes_hold_safe_target_at_genesis.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 2);
    assert_eq!(r.attestations, 1);
}

#[test]
fn head_retreats_onto_shorter_justified_fork() {
    let Some(r) = run_ok(
        "test_head_movement/test_head_retreats_onto_shorter_justified_fork.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 11);
    assert_eq!(r.rejections, 0);
}

#[test]
fn equal_slot_justified_keeps_original_root() {
    let Some(r) = run_ok(
        "test_head_movement/test_equal_slot_justified_candidate_keeps_original_root.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 4);
    assert_eq!(r.rejections, 0);
}

#[test]
fn finalization_prunes_stale_aggregated_payloads() {
    let Some(r) = run_ok(
        "test_store_pruning/test_finalization_prunes_stale_aggregated_payloads.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 6);
    assert_eq!(r.attestations, 2);
    assert_eq!(r.rejections, 0);
}

#[test]
fn finalization_prunes_vote_on_orphaned_branch() {
    let Some(r) = run_ok(
        "test_prune_finalized_orphaned_branch/test_finalization_prunes_vote_on_orphaned_branch.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 7);
    assert_eq!(r.attestations, 1);
    assert_eq!(r.rejections, 0);
}

#[test]
fn re_gossip_of_pruned_orphaned_vote_is_rejected() {
    let Some(r) = run_ok(
        "test_prune_finalized_orphaned_branch/test_re_gossip_of_pruned_orphaned_vote_is_rejected.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 7);
    assert!(r.rejections >= 1);
}
