//! Integration tests for the leanSpec fork-choice fixture runner.

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

#[test]
fn runs_beyond_future_horizon_when_cache_present() {
    let Some(path) = fixture(
        "test_block_future_horizon/test_block_beyond_future_horizon_rejected.json",
    ) else {
        eprintln!("skip: fetch-leanspec-fixtures.ps1 cache missing");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let reports = run_fork_choice_file(&bytes).unwrap();
    assert_eq!(reports[0].1.rejections, 1);
}

#[test]
fn runs_one_past_horizon_after_tick() {
    let Some(path) =
        fixture("test_block_future_horizon/test_block_one_past_horizon_rejected.json")
    else {
        eprintln!("skip: cache missing");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let reports = run_fork_choice_file(&bytes).unwrap();
    assert_eq!(reports[0].1.ticks, 1);
    assert_eq!(reports[0].1.rejections, 1);
    assert!(reports[0].1.assertions >= 1);
}

#[test]
fn runs_unknown_parent_after_valid_import() {
    let Some(path) = fixture(
        "test_block_unknown_parent/test_block_with_fabricated_parent_is_rejected.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let reports = run_fork_choice_file(&bytes).unwrap();
    assert_eq!(reports[0].1.imports, 1);
    assert_eq!(reports[0].1.rejections, 1);
}

#[test]
fn runs_attestation_unknown_source() {
    let Some(path) = fixture(
        "test_gossip_attestation_validation/test_attestation_unknown_source_block_rejected.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let reports = run_fork_choice_file(&bytes).unwrap();
    assert_eq!(reports[0].1.imports, 2);
    assert_eq!(reports[0].1.rejections, 1);
}

#[test]
fn runs_attestation_too_far_in_future() {
    let Some(path) = fixture(
        "test_gossip_attestation_validation/test_attestation_too_far_in_future_rejected.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let reports = run_fork_choice_file(&bytes).unwrap();
    assert_eq!(reports[0].1.imports, 2);
    assert_eq!(reports[0].1.rejections, 1);
}

#[test]
fn runs_block_with_maximum_body_attestations() {
    let Some(path) =
        fixture("test_block_attestation_limits/test_block_with_maximum_attestations.json")
    else {
        eprintln!("skip: cache missing");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let reports = run_fork_choice_file(&bytes).unwrap();
    assert_eq!(reports[0].1.imports, 9);
    assert_eq!(reports[0].1.rejections, 0);
}

#[test]
fn runs_gossip_aggregate_head_slot_mismatch() {
    let Some(path) = fixture(
        "test_gossip_aggregated_attestation_validation/test_aggregated_attestation_head_slot_mismatch_rejected.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let reports = run_fork_choice_file(&bytes).unwrap();
    assert!(reports[0].1.imports >= 1);
    assert_eq!(reports[0].1.rejections, 1);
}

#[test]
fn runs_gossip_aggregate_empty_participants() {
    let Some(path) = fixture(
        "test_gossip_aggregated_empty_participants/test_gossip_aggregated_attestation_empty_participants_rejected.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let reports = run_fork_choice_file(&bytes).unwrap();
    assert_eq!(reports[0].1.rejections, 1);
}

#[test]
fn runs_valid_gossip_aggregated_attestation() {
    let Some(path) = fixture(
        "test_gossip_aggregated_attestation_validation/test_valid_gossip_aggregated_attestation.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let reports = run_fork_choice_file(&bytes).unwrap();
    assert!(reports[0].1.attestations >= 1);
    assert_eq!(reports[0].1.rejections, 0);
}

#[test]
fn runs_block_includes_genesis_self_vote() {
    let Some(path) =
        fixture("test_block_genesis_self_vote/test_block_includes_genesis_self_vote.json")
    else {
        eprintln!("skip: cache missing");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let reports = run_fork_choice_file(&bytes).unwrap();
    assert!(reports[0].1.imports >= 1);
    assert_eq!(reports[0].1.rejections, 0);
}

#[test]
fn runs_justification_fixed_point_with_gossip_aggregates() {
    let Some(path) = fixture(
        "test_block_production/test_block_builder_fixed_point_advances_justification.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let reports = run_fork_choice_file(&bytes).unwrap();
    assert_eq!(reports[0].1.imports, 6);
    assert_eq!(reports[0].1.ticks, 2);
    assert_eq!(reports[0].1.attestations, 2);
    assert_eq!(reports[0].1.rejections, 0);
}

#[test]
fn runs_attestation_target_advances_with_body_votes() {
    let Some(path) = fixture(
        "test_attestation_target_selection/test_attestation_target_advances_with_attestations.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let reports = run_fork_choice_file(&bytes).unwrap();
    assert_eq!(reports[0].1.imports, 5);
    assert_eq!(reports[0].1.rejections, 0);
}
