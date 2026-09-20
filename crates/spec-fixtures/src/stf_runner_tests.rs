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

/// Walk the cached STF tree and print pass/fail (manual inventory; not a hard gate).
#[test]
fn inventory_cached_stf_vectors() {
    ensure_cache_env();
    let Some(root) = fixtures_root_from_env() else {
        eprintln!("skip: cache missing");
        return;
    };
    let st = root
        .join("fixtures")
        .join("consensus")
        .join("state_transition");
    if !st.is_dir() {
        eprintln!("skip: no state_transition tree");
        return;
    }
    let mut files = Vec::new();
    fn walk(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().and_then(|s| s.to_str()) == Some("json") {
                out.push(p);
            }
        }
    }
    walk(&st, &mut files);
    files.sort();
    let mut ok = 0usize;
    let mut fail = 0usize;
    for p in &files {
        let rel = p.strip_prefix(&st).unwrap_or(p);
        let bytes = std::fs::read(p).unwrap();
        match run_state_transition_file(&bytes) {
            Ok(_) => {
                ok += 1;
                eprintln!("OK  {}", rel.display());
            }
            Err(e) => {
                fail += 1;
                eprintln!("ERR {} :: {e}", rel.display());
            }
        }
    }
    eprintln!("summary ok={ok} fail={fail} total={}", files.len());
    assert_eq!(fail, 0, "all cached STF vectors should pass");
    assert_eq!(ok, files.len());
}
