use super::*;
use ethean_rpc::TestDriver;

#[test]
fn init_rejects_garbage_and_step_needs_init() {
    let driver = NodeTestDriver::default();
    assert!(driver.fork_choice_init(b"[]").is_err());
    assert!(driver.fork_choice_step(b"{}").is_err());
    let v: Value = serde_json::from_str(&driver.state_transition(b"{}").unwrap()).unwrap();
    assert_eq!(v["succeeded"], Value::Bool(false));
    let v: Value = serde_json::from_str(&driver.verify_signatures(b"{}").unwrap()).unwrap();
    assert_eq!(v["succeeded"], Value::Bool(false));
}

#[test]
fn env_switch_reads_one() {
    std::env::remove_var(DRIVER_ENV);
    assert!(!enabled_by_env());
    std::env::set_var(DRIVER_ENV, "1");
    assert!(enabled_by_env());
    std::env::remove_var(DRIVER_ENV);
}

/// Runs the three verify_signatures fixtures through the production verifier
/// when the archive is extracted (release builds only: real proofs).
#[test]
fn verify_signatures_fixtures_when_present() {
    if cfg!(debug_assertions) {
        eprintln!("skip: proof verification runs in release tests");
        return;
    }
    let mut dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.pop();
    dir.pop();
    dir.push(".cache/leanspec-fixtures/extracted/fixtures/consensus/verify_signatures");
    if !dir.is_dir() {
        eprintln!("skip: fixture archive not extracted");
        return;
    }
    let mut files = Vec::new();
    collect(&dir, &mut files);
    assert!(!files.is_empty());
    for file in files {
        let doc: serde_json::Map<String, Value> =
            serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
        for (id, case) in &doc {
            let expect_reject = case
                .get("expectException")
                .or_else(|| case.get("rejectionReason"))
                .is_some();
            let resp = verify_signatures_case(case);
            assert_eq!(
                resp["succeeded"].as_bool(),
                Some(!expect_reject),
                "{id}: {resp}"
            );
        }
    }
}

/// Every fork_choice fixture through the node driver with the real
/// verifier, exactly as hive's spec-asset suite drives it (release only).
#[test]
fn fork_choice_fixtures_with_real_verifier() {
    if cfg!(debug_assertions) {
        eprintln!("skip: vote verification runs in release tests");
        return;
    }
    let mut dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.pop();
    dir.pop();
    dir.push(".cache/leanspec-fixtures/extracted/fixtures/consensus/fork_choice");
    if !dir.is_dir() {
        eprintln!("skip: fixture archive not extracted");
        return;
    }
    let mut files = Vec::new();
    collect(&dir, &mut files);
    let driver = NodeTestDriver::default();
    let (mut cases, mut steps) = (0, 0);
    for file in files {
        let doc: serde_json::Map<String, Value> =
            serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
        for (id, case) in &doc {
            let init = json!({
                "anchorState": case["anchorState"],
                "anchorBlock": case["anchorBlock"],
                "genesisTime": case.pointer("/anchorState/config/genesisTime"),
            });
            if driver
                .fork_choice_init(init.to_string().as_bytes())
                .is_err()
            {
                continue;
            }
            for (i, step) in case["steps"].as_array().unwrap().iter().enumerate() {
                let resp: Value = serde_json::from_str(
                    &driver
                        .fork_choice_step(step.to_string().as_bytes())
                        .unwrap(),
                )
                .unwrap();
                if let Some(valid) = step.get("valid").and_then(Value::as_bool) {
                    assert_eq!(
                        resp["accepted"].as_bool(),
                        Some(valid),
                        "{id} step {i}: {resp}"
                    );
                }
                steps += 1;
            }
            cases += 1;
        }
    }
    eprintln!("node driver: {cases} fork_choice cases, {steps} steps");
    assert!(cases > 0);
}

/// leanSpec `verify_single_message_proofs` through the production verifier (release only).
#[test]
fn single_message_proof_fixtures_when_present() {
    if cfg!(debug_assertions) {
        eprintln!("skip: proof verification runs in release tests");
        return;
    }
    let mut dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.pop();
    dir.pop();
    dir.push(".cache/leanspec-fixtures/extracted/fixtures/consensus/verify_single_message_proofs");
    if !dir.is_dir() {
        eprintln!("skip: fixture archive not extracted");
        return;
    }
    let mut files = Vec::new();
    collect(&dir, &mut files);
    assert!(!files.is_empty());
    for file in files {
        let doc: serde_json::Map<String, Value> =
            serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
        for (id, case) in &doc {
            let keys: Vec<PublicKey> = case["publicKeys"]
                .as_array()
                .unwrap()
                .iter()
                .map(|k| {
                    PublicKey::try_from_slice(&decode_hex_bytes(k.as_str().unwrap()).unwrap())
                        .unwrap()
                })
                .collect();
            let message: [u8; 32] = decode_hex_bytes(case["message"].as_str().unwrap())
                .unwrap()
                .try_into()
                .unwrap();
            let slot = case["slot"].as_u64().unwrap();
            let proof = decode_hex_bytes(case["proof"]["data"].as_str().unwrap()).unwrap();
            let expect_reject = case.get("rejectionReason").is_some();
            let ok = LeanMultisigVerifier
                .verify_single(&proof, &keys, &message, slot)
                .is_ok();
            assert_eq!(ok, !expect_reject, "{id}");
        }
    }
}

fn collect(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let p = entry.unwrap().path();
        if p.is_dir() {
            collect(&p, out);
        } else if p.extension().is_some_and(|e| e == "json") {
            out.push(p);
        }
    }
}
