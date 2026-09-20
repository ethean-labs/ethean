//! Additional leanSpec FC vectors (safe-target, reorg, divergence, equivocation).

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
fn safe_target_follows_heavier_fork_on_split() {
    let Some(r) =
        run_ok("test_safe_target/test_safe_target_follows_heavier_fork_on_split.json")
    else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 3);
    assert_eq!(r.attestations, 2);
}

#[test]
fn safe_target_ignores_known_pool_at_interval_3() {
    let Some(r) =
        run_ok("test_safe_target/test_safe_target_ignores_known_pool_at_interval_3.json")
    else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 3);
    assert_eq!(r.attestations, 1);
}

#[test]
fn supermajority_five_of_seven_advances() {
    let Some(r) = run_ok(
        "test_safe_target_supermajority/test_odd_seven_validators_five_votes_advance_safe_target.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 2);
    assert_eq!(r.attestations, 1);
}

#[test]
fn supermajority_four_of_seven_holds_genesis() {
    let Some(r) = run_ok(
        "test_safe_target_supermajority/test_odd_seven_validators_four_votes_hold_safe_target_at_genesis.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 2);
    assert_eq!(r.attestations, 1);
}

#[test]
fn same_slot_equivocating_attesters_count_once() {
    let Some(r) =
        run_ok("test_equivocation/test_same_slot_equivocating_attesters_count_once.json")
    else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 3);
    assert_eq!(r.attestations, 2);
}

#[test]
fn finalization_prunes_stale_attestation_signatures() {
    let Some(r) = run_ok(
        "test_store_pruning/test_finalization_prunes_stale_attestation_signatures.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 6);
    assert_eq!(r.rejections, 0);
}

#[test]
fn honest_vote_sources_from_head_chain() {
    let Some(r) = run_ok(
        "test_attestation_source_divergence/test_honest_vote_sources_from_head_chain_not_store.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 4);
    assert_eq!(r.attestations, 1);
}

#[test]
fn justified_divergence_self_heals() {
    let Some(r) = run_ok(
        "test_attestation_source_divergence/test_justified_divergence_self_heals_in_next_block.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 5);
}

#[test]
fn reorg_on_newly_justified_slot() {
    let Some(r) =
        run_ok("test_fork_choice_reorgs/test_reorg_on_newly_justified_slot.json")
    else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 6);
}

#[test]
fn reorg_depth_across_deep_chain_split() {
    let Some(r) =
        run_ok("test_fork_choice_reorgs/test_reorg_depth_across_deep_chain_split.json")
    else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 22);
}

#[test]
fn non_genesis_anchor_internally_consistent() {
    let Some(r) =
        run_ok("test_checkpoint_sync/test_non_genesis_anchor_is_internally_consistent.json")
    else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 0);
    assert!(r.assertions >= 1);
}

#[test]
fn losing_fork_higher_finalized_does_not_latch() {
    let Some(r) = run_ok(
        "test_finalized_safety/test_losing_fork_higher_finalized_does_not_latch.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert!(r.imports >= 1);
    assert_eq!(r.rejections, 0);
}

#[test]
fn fork_above_finalized_wins_at_or_below_loses() {
    let Some(r) = run_ok(
        "test_finalized_safety/test_fork_above_finalized_wins_at_or_below_loses.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 9);
    assert_eq!(r.rejections, 0);
}

#[test]
fn heavier_fork_below_finalized_slot_never_wins() {
    let Some(r) = run_ok(
        "test_finalized_safety/test_heavier_fork_below_finalized_slot_never_wins.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 9);
    assert_eq!(r.rejections, 0);
}

#[test]
fn tick_interval_0_skips_acceptance_when_not_proposer() {
    let Some(r) = run_ok(
        "test_tick_system/test_tick_interval_0_skips_acceptance_when_not_proposer.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 2);
    assert_eq!(r.ticks, 6);
    assert_eq!(r.attestations, 3);
}

#[test]
fn tick_interval_progression_advances_safe_target() {
    let Some(r) = run_ok(
        "test_tick_system/test_tick_interval_progression_through_full_slot.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    assert_eq!(r.imports, 2);
    assert_eq!(r.ticks, 5);
    assert_eq!(r.attestations, 1);
}
