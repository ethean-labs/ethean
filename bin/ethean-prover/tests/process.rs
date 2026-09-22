//! Drive the real `ethean-prover` binary through `ProverClient`.

use std::path::PathBuf;
use std::time::Duration;

use ethean_crypto::{CryptoBackend, ProductionBackend, PublicKey, SecretKeyMaterial};
use ethean_multisig::{verify_single, MultisigError, ProverClient, ProverConfig};

fn hex_field(doc: &str, key: &str) -> Vec<u8> {
    let needle = format!("\"{key}\": \"");
    let start = doc.find(&needle).unwrap() + needle.len();
    let end = doc[start..].find('"').unwrap() + start;
    let hex = &doc[start..end];
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

fn attester(index: usize) -> Option<(PublicKey, SecretKeyMaterial)> {
    let dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../leansig-test-keys/prod_scheme");
    let doc = std::fs::read_to_string(dir.join(format!("{index}.json"))).ok()?;
    let section = &doc[doc.find("attestation_keypair")?..];
    let pk = PublicKey::try_from_slice(&hex_field(section, "public_key")).ok()?;
    let sk = SecretKeyMaterial::from_xmss_ssz(hex_field(section, "secret_key")).ok()?;
    Some((pk, sk))
}

fn client(timeout: Duration) -> ProverClient {
    let mut config = ProverConfig::new(PathBuf::from(env!("CARGO_BIN_EXE_ethean-prover")));
    config.request_timeout = timeout;
    ProverClient::new(config)
}

#[test]
fn proves_through_child_process_and_recovers() {
    let Some(keys) = (0..2).map(attester).collect::<Option<Vec<_>>>() else {
        eprintln!("skipping: leansig-test-keys/prod_scheme not found");
        return;
    };
    let message = [0x33u8; 32];
    let slot = 5u64;
    let raw: Vec<_> = keys
        .iter()
        .map(|(pk, sk)| {
            (
                *pk,
                ProductionBackend.sign(sk, slot as u32, &message).unwrap(),
            )
        })
        .collect();
    let public_keys: Vec<PublicKey> = keys.iter().map(|(pk, _)| *pk).collect();

    let prover = client(Duration::from_secs(300));
    let proof = prover
        .aggregate_type1(vec![], raw.clone(), message, slot)
        .expect("prove");
    verify_single(&proof, &public_keys, &message, slot).expect("verifies");

    // A prover-side error is reported and leaves the process usable.
    let err = prover
        .aggregate_type1(vec![], vec![], message, slot)
        .unwrap_err();
    assert!(matches!(err, MultisigError::ProverFailed(_)), "{err:?}");
    let again = prover
        .aggregate_type1(vec![], raw[..1].to_vec(), message, slot)
        .expect("still serving");
    verify_single(&again, &public_keys[..1], &message, slot).expect("verifies");

    // A timeout kills the child; the next call respawns it.
    let impatient = client(Duration::from_millis(1));
    let err = impatient
        .aggregate_type1(vec![], raw.clone(), message, slot)
        .unwrap_err();
    assert!(
        matches!(err, MultisigError::ProverUnavailable(_)),
        "{err:?}"
    );
    drop(impatient);

    let missing = ProverClient::new(ProverConfig::new(PathBuf::from(
        "/nonexistent/ethean-prover",
    )));
    assert!(matches!(
        missing.merge_type2(vec![]),
        Err(MultisigError::ProverUnavailable(_))
    ));
}
