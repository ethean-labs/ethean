//! Integration tests for the leanSpec state-transition fixture runner.

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
    let mut p = root.join("fixtures").join("consensus").join("state_transition");
    p.push("lstar");
    p.push("state_transition");
    for part in rel.split('/') {
        p.push(part);
    }
    p.is_file().then_some(p)
}

#[test]
fn runs_block_at_large_slot_number() {
    let Some(path) =
        fixture("test_block_processing/test_block_at_large_slot_number.json")
    else {
        eprintln!("skip: cache missing");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let reports = run_state_transition_file(&bytes).unwrap();
    assert_eq!(reports[0].1.imports, 1);
    assert_eq!(reports[0].1.rejections, 0);
}

#[test]
fn runs_empty_aggregation_bits_rejection() {
    let Some(path) = fixture(
        "test_aggregation_bits/test_zero_length_aggregation_bits_rejects_block.json",
    ) else {
        eprintln!("skip: cache missing");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let reports = run_state_transition_file(&bytes).unwrap();
    assert_eq!(reports[0].1.rejections, 1);
}
