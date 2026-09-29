//! Prover timing against signature and component counts. Ignored by default:
//!
//! ```text
//! cargo test --release -p ethean-prover --test timing -- --ignored --nocapture
//! ```
//!
//! Keys are generated in-process (about 20 s each in release, two PROD bottom
//! trees), so `ETHEAN_PROBE_KEYS` distinct keys (default 4) are cycled to fill
//! larger raw sets. The proving cost depends on how many signatures leanVM
//! verifies, not on whether keys repeat. `ETHEAN_PROBE_SIZES` (default
//! `1,8,32,64,128`) and `ETHEAN_PROBE_COMPONENTS` (default `1,2,4,8`) pick the
//! points. Every key signs several messages at one epoch here, which a real
//! signer never does; this probe only measures time.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use ethean_crypto::{CryptoBackend, ProductionBackend, PublicKey, SecretKeyMaterial, Signature};
use ethean_multisig::{verify_single, KeyedProof, ProverClient, ProverConfig};

const SLOT: u64 = 5;

fn env_list(name: &str, default: &[usize]) -> Vec<usize> {
    std::env::var(name)
        .ok()
        .map(|v| v.split(',').filter_map(|s| s.trim().parse().ok()).collect())
        .unwrap_or_else(|| default.to_vec())
}

fn keys(count: usize) -> Vec<(PublicKey, SecretKeyMaterial)> {
    let started = Instant::now();
    let keys: Vec<_> = (0..count)
        .map(|_| ProductionBackend.key_gen(0, 16).expect("key_gen"))
        .collect();
    eprintln!("generated {count} PROD keys in {:?}", started.elapsed());
    keys
}

fn raw_set(
    keys: &[(PublicKey, SecretKeyMaterial)],
    message: &[u8; 32],
    size: usize,
) -> Vec<(PublicKey, Signature)> {
    let signed: Vec<_> = keys
        .iter()
        .map(|(pk, sk)| {
            let sig = ProductionBackend
                .sign(sk, SLOT as u32, message)
                .expect("sign");
            (*pk, sig)
        })
        .collect();
    signed.iter().cycle().take(size).cloned().collect()
}

fn timed<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let started = Instant::now();
    let out = f();
    (out, started.elapsed())
}

#[test]
#[ignore]
fn timing_probe_type1_and_type2_scaling() {
    let mut config = ProverConfig::new(PathBuf::from(env!("CARGO_BIN_EXE_ethean-prover")));
    config.request_timeout = Duration::from_secs(900);
    let prover = ProverClient::new(config);
    let keys = keys(env_list("ETHEAN_PROBE_KEYS", &[4])[0].max(1));

    let warm = raw_set(&keys, &[0u8; 32], 1);
    let (_, cold) = timed(|| prover.aggregate_type1(vec![], warm, [0u8; 32], SLOT));
    eprintln!("cold start + first Type-1: {cold:?}");

    for size in env_list("ETHEAN_PROBE_SIZES", &[1, 8, 32, 64, 128]) {
        let message = [size as u8; 32];
        let raw = raw_set(&keys, &message, size);
        let public_keys: Vec<PublicKey> = raw.iter().map(|(pk, _)| *pk).collect();
        let (proof, took) = timed(|| prover.aggregate_type1(vec![], raw, message, SLOT));
        match proof {
            Ok(proof) => {
                let (checked, verify) =
                    timed(|| verify_single(&proof, &public_keys, &message, SLOT));
                eprintln!(
                    "type1 raw={size:>4}: prove {took:?}, verify {verify:?} ({}), {} bytes",
                    if checked.is_ok() { "ok" } else { "FAILED" },
                    proof.len()
                );
            }
            Err(e) => eprintln!("type1 raw={size:>4}: error after {took:?}: {e}"),
        }
    }

    let per_component = keys.len();
    for count in env_list("ETHEAN_PROBE_COMPONENTS", &[1, 2, 4, 8]) {
        let mut components = Vec::with_capacity(count);
        for i in 0..count {
            let message = [0x80 | i as u8; 32];
            let raw = raw_set(&keys, &message, per_component);
            let public_keys = raw.iter().map(|(pk, _)| *pk).collect();
            let proof = prover
                .aggregate_type1(vec![], raw, message, SLOT)
                .expect("component Type-1");
            components.push(KeyedProof { public_keys, proof });
        }
        let (merged, took) = timed(|| prover.merge_type2(components));
        match merged {
            Ok(proof) => eprintln!(
                "type2 components={count:>2} ({per_component} sigs each): merge {took:?}, {} bytes",
                proof.len()
            ),
            Err(e) => eprintln!("type2 components={count:>2}: error after {took:?}: {e}"),
        }
    }
}
