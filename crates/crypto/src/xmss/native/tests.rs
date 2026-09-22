//! End-to-end checks against leanSpec-generated keys (test and prod schemes).

use super::keys::{XmssPublicKey, XmssSecretKey, XmssSignature};
use super::leaves::bottom_tree_leaves;
use super::params::{PROD, TEST};
use super::rand::SeededRandom;
use super::scheme::{
    advance_preparation, expand_activation_time, key_gen, prepare_for_epoch, sign, verify,
};
use std::path::PathBuf;

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    let hex = hex.trim_start_matches("0x");
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("hex"))
        .collect()
}

/// Extract `"key": "<hex>"` from a flat JSON document.
fn json_hex_field(doc: &str, key: &str) -> Vec<u8> {
    let needle = format!("\"{key}\": \"");
    let start = doc.find(&needle).expect("field present") + needle.len();
    let end = doc[start..].find('"').expect("closing quote") + start;
    hex_to_bytes(&doc[start..end])
}

fn test_key(index: usize, role: &str) -> (XmssPublicKey, XmssSecretKey) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/xmss_test_keys")
        .join(format!("{index}.json"));
    let doc = std::fs::read_to_string(path).expect("vendored test key");
    let pk = XmssPublicKey::from_ssz(&json_hex_field(&doc, &format!("{role}_public"))).unwrap();
    let sk = XmssSecretKey::from_ssz(&json_hex_field(&doc, &format!("{role}_secret"))).unwrap();
    (pk, sk)
}

#[test]
fn test_scheme_keys_decode_and_match() {
    let (pk, sk) = test_key(0, "attestation");
    assert_eq!(sk.public_key(), pk);
    assert_eq!(sk.activation_interval(), 0..112);
    assert_eq!(sk.prepared_interval(&TEST), 0..32);
    assert_eq!(sk.top_tree.depth, 8);
    assert_eq!(sk.top_tree.lowest_layer, 4);
    // Recomputing bottom-tree leaves from the PRF key must reproduce the
    // stored layer-0 nodes: this pins PRF, chains, tweaks and the leaf sponge.
    let leaves = bottom_tree_leaves(&TEST, &sk.prf_key, &sk.parameter, 1);
    assert_eq!(sk.right_bottom_tree.layers[0].nodes, leaves);
    let bytes = sk.to_ssz();
    assert_eq!(XmssSecretKey::from_ssz(&bytes).unwrap().to_ssz(), bytes);
}

#[test]
fn test_scheme_sign_verify_roundtrip() {
    let (pk, mut sk) = test_key(1, "proposal");
    let message = [0x42u8; 32];
    for epoch in [0u32, 1, 17, 31] {
        let sig = sign(&TEST, &sk, epoch, &message).unwrap();
        assert!(verify(&TEST, &pk, epoch, &message, &sig));
        assert!(!verify(&TEST, &pk, epoch + 1, &message, &sig));
        assert!(!verify(&TEST, &pk, epoch, &[0x43u8; 32], &sig));
        let bytes = sig.to_ssz();
        assert_eq!(bytes.len(), TEST.signature_bytes());
        let decoded = XmssSignature::from_ssz(&TEST, &bytes).unwrap();
        assert_eq!(decoded, sig);
        assert_eq!(
            sign(&TEST, &sk, epoch, &message).unwrap(),
            sig,
            "deterministic"
        );
    }
    assert!(sign(&TEST, &sk, 40, &message).is_err(), "unprepared epoch");
    prepare_for_epoch(&TEST, &mut sk, 40).unwrap();
    let sig = sign(&TEST, &sk, 40, &message).unwrap();
    assert!(verify(&TEST, &pk, 40, &message, &sig));
    prepare_for_epoch(&TEST, &mut sk, 111).unwrap();
    assert!(verify(
        &TEST,
        &pk,
        111,
        &message,
        &sign(&TEST, &sk, 111, &message).unwrap()
    ));
    assert!(prepare_for_epoch(&TEST, &mut sk, 112).is_err());
}

#[test]
fn test_scheme_rejects_tampered_signature() {
    let (pk, sk) = test_key(2, "attestation");
    let message = [9u8; 32];
    let sig = sign(&TEST, &sk, 5, &message).unwrap();
    let mut bytes = sig.to_ssz();
    bytes[40] ^= 1;
    let tampered = XmssSignature::from_ssz(&TEST, &bytes).unwrap();
    assert!(!verify(&TEST, &pk, 5, &message, &tampered));
    let mut short = sig.to_ssz();
    short.pop();
    assert!(XmssSignature::from_ssz(&TEST, &short).is_err());
    let mut noncanonical = sig.to_ssz();
    noncanonical[4..8].copy_from_slice(&crate::field::P.to_le_bytes());
    assert!(XmssSignature::from_ssz(&TEST, &noncanonical).is_err());
}

#[test]
fn test_scheme_keygen_from_seed() {
    let mut rng = SeededRandom::new(b"keygen-test");
    let (pk, mut sk) = key_gen(&TEST, &mut rng, 20, 30).unwrap();
    assert_eq!(sk.activation_interval(), 16..64);
    assert_eq!(pk.to_ssz().len(), 52);
    let message = [7u8; 32];
    let sig = sign(&TEST, &sk, 20, &message).unwrap();
    assert!(verify(&TEST, &pk, 20, &message, &sig));
    assert!(sign(&TEST, &sk, 50, &message).is_err());
    assert!(advance_preparation(&TEST, &mut sk).unwrap());
    assert!(verify(
        &TEST,
        &pk,
        50,
        &message,
        &sign(&TEST, &sk, 50, &message).unwrap()
    ));
    assert!(!advance_preparation(&TEST, &mut sk).unwrap());
    let sk2 = XmssSecretKey::from_ssz(&sk.to_ssz()).unwrap();
    assert_eq!(
        sign(&TEST, &sk2, 50, &message).unwrap(),
        sign(&TEST, &sk, 50, &message).unwrap()
    );
    let mut rng2 = SeededRandom::new(b"keygen-test");
    assert_eq!(
        key_gen(&TEST, &mut rng2, 20, 30).unwrap().0,
        pk,
        "seeded keygen is reproducible"
    );
}

#[test]
fn activation_window_expansion() {
    assert_eq!(expand_activation_time(&TEST, 0, 1), (0, 2));
    assert_eq!(expand_activation_time(&TEST, 20, 30), (1, 4));
    assert_eq!(expand_activation_time(&TEST, 250, 6), (14, 16));
    assert_eq!(expand_activation_time(&TEST, 0, 256), (0, 16));
    assert_eq!(expand_activation_time(&PROD, 0, 1), (0, 2));
    assert_eq!(
        expand_activation_time(&PROD, (1u64 << 32) - 1, 1),
        ((1 << 16) - 2, 1 << 16)
    );
}

/// Prod-scheme keys from `leansig-test-keys/prod_scheme` (33 MB each); the
/// checkout lives next to the workspace and is skipped when absent.
#[test]
fn prod_scheme_sign_verify_with_published_keys() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../leansig-test-keys/prod_scheme/0.json");
    let Ok(doc) = std::fs::read_to_string(&path) else {
        eprintln!("skipping: {} not found", path.display());
        return;
    };
    let pk = XmssPublicKey::from_ssz(&json_hex_field(&doc, "public_key")).unwrap();
    let sk = XmssSecretKey::from_ssz(&json_hex_field(&doc, "secret_key")).unwrap();
    assert_eq!(sk.public_key(), pk);
    assert_eq!(sk.prepared_interval(&PROD), 0..131_072);
    let message = [0xabu8; 32];
    for epoch in [0u32, 1, 65_535, 65_536, 131_071] {
        let sig = sign(&PROD, &sk, epoch, &message).unwrap();
        let bytes = sig.to_ssz();
        assert_eq!(bytes.len(), 2536);
        let decoded = XmssSignature::from_ssz(&PROD, &bytes).unwrap();
        assert!(
            verify(&PROD, &pk, epoch, &message, &decoded),
            "epoch {epoch}"
        );
        assert!(!verify(&PROD, &pk, epoch, &[0u8; 32], &decoded));
    }
    let sig = sign(&PROD, &sk, 7, &message).unwrap();
    let start = std::time::Instant::now();
    for _ in 0..20 {
        assert!(verify(&PROD, &pk, 7, &message, &sig));
    }
    eprintln!(
        "prod-scheme verify: {:?} per signature",
        start.elapsed() / 20
    );
}

/// Rough timing probe; run with `--release --nocapture` to read the numbers.
#[test]
fn timing_probe_test_scheme() {
    let (pk, sk) = test_key(0, "attestation");
    let message = [1u8; 32];
    let sig = sign(&TEST, &sk, 3, &message).unwrap();
    let start = std::time::Instant::now();
    let iterations = 200;
    for _ in 0..iterations {
        assert!(verify(&TEST, &pk, 3, &message, &sig));
    }
    eprintln!(
        "test-scheme verify: {:?} per signature",
        start.elapsed() / iterations
    );
    let start = std::time::Instant::now();
    let mut state = [crate::field::Fp::ONE; 24];
    for _ in 0..10_000 {
        crate::poseidon::permute24(&mut state);
    }
    eprintln!("poseidon24: {:?} per permutation", start.elapsed() / 10_000);
    let start = std::time::Instant::now();
    let mut state = [crate::field::Fp::ONE; 16];
    for _ in 0..10_000 {
        crate::poseidon::permute16(&mut state);
    }
    eprintln!("poseidon16: {:?} per permutation", start.elapsed() / 10_000);
}
