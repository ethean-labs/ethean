//! Enumerate every extracted leanSpec suite Ethean can run.

use crate::discover::{discover_json_fixtures, fixtures_root_from_env};
use crate::suite_networking::{run_networking_case, NetOutcome};
use crate::suite_scalar::{
    run_justifiability_case, run_poseidon_case, run_slot_clock_case, run_sync_case,
};
use crate::suite_ssz::{run_ssz_case, SszOutcome};
use crate::FIXTURES_ENV;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn root() -> Option<PathBuf> {
    if fixtures_root_from_env().is_none() {
        let mut def = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        def.pop();
        def.pop();
        def.push(".cache/leanspec-fixtures/extracted");
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

fn cases(suite: &str) -> Vec<(String, Value)> {
    let Some(root) = root() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for file in discover_json_fixtures(&root).unwrap() {
        if !file
            .to_string_lossy()
            .contains(&format!("/consensus/{suite}/"))
        {
            continue;
        }
        let doc: serde_json::Map<String, Value> =
            serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
        for (id, case) in doc {
            out.push((format!("{}::{id}", file.display()), case));
        }
    }
    out
}

fn report(
    suite: &str,
    checked: usize,
    rejected: usize,
    unsupported: &BTreeMap<String, usize>,
    failures: &[String],
) {
    eprintln!("{suite}: {checked} checked, {rejected} rejected, unsupported {unsupported:?}");
    assert!(
        failures.is_empty(),
        "{suite} failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn ssz_suite() {
    let (mut checked, mut rejected) = (0, 0);
    let mut unsupported = BTreeMap::new();
    let mut failures = Vec::new();
    for (id, case) in cases("ssz") {
        match run_ssz_case(&case) {
            Ok(SszOutcome::Checked) => checked += 1,
            Ok(SszOutcome::Rejected) => rejected += 1,
            Ok(SszOutcome::Unsupported(t)) => *unsupported.entry(t).or_insert(0) += 1,
            Err(e) => failures.push(format!("{id}: {e}")),
        }
    }
    report("ssz", checked, rejected, &unsupported, &failures);
}

#[test]
fn networking_codec_suite() {
    let (mut checked, mut rejected) = (0, 0);
    let mut unsupported = BTreeMap::new();
    let mut failures = Vec::new();
    for (id, case) in cases("networking_codec") {
        match run_networking_case(&case) {
            Ok(NetOutcome::Checked) => checked += 1,
            Ok(NetOutcome::Rejected) => rejected += 1,
            Ok(NetOutcome::Unsupported(t)) => *unsupported.entry(t).or_insert(0) += 1,
            Err(e) => failures.push(format!("{id}: {e}")),
        }
    }
    report(
        "networking_codec",
        checked,
        rejected,
        &unsupported,
        &failures,
    );
}

#[test]
fn scalar_suites() {
    for (suite, run) in [
        (
            "slot_clock",
            run_slot_clock_case as fn(&Value) -> Result<(), String>,
        ),
        ("justifiability", run_justifiability_case),
        ("poseidon_permutation", run_poseidon_case),
    ] {
        let mut checked = 0;
        let mut failures = Vec::new();
        for (id, case) in cases(suite) {
            match run(&case) {
                Ok(()) => checked += 1,
                Err(e) => failures.push(format!("{id}: {e}")),
            }
        }
        report(suite, checked, 0, &BTreeMap::new(), &failures);
    }
}

#[test]
fn sync_suite() {
    let Some(root) = root() else {
        return;
    };
    let keys = root.join("fixtures").join("keys").join("prod_scheme");
    let mut checked = 0;
    let mut failures = Vec::new();
    for (id, case) in cases("sync") {
        match run_sync_case(&case, &keys) {
            Ok(()) => checked += 1,
            Err(e) => failures.push(format!("{id}: {e}")),
        }
    }
    report("sync", checked, 0, &BTreeMap::new(), &failures);
}
