//! Shared fixtures: leanSpec PROD keys from a `leansig-test-keys` checkout
//! next to the workspace. Tests skip when the checkout is absent.

use std::path::PathBuf;

use ethean_crypto::{CryptoBackend, ProductionBackend, PublicKey, SecretKeyMaterial, Signature};

pub struct ProdKey {
    pub public_key: PublicKey,
    pub secret: SecretKeyMaterial,
}

fn keys_dir() -> PathBuf {
    std::env::var_os("ETHEAN_PROD_TEST_KEYS")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../leansig-test-keys/prod_scheme")
        })
}

fn hex_field(doc: &str, key: &str) -> Vec<u8> {
    let needle = format!("\"{key}\": \"");
    let start = doc.find(&needle).expect("field") + needle.len();
    let end = doc[start..].find('"').expect("quote") + start;
    let hex = doc[start..end].trim_start_matches("0x");
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("hex"))
        .collect()
}

/// Load `role` ("attestation_keypair" / "proposal_keypair") of validator `index`.
pub fn prod_key(index: usize, role: &str) -> Option<ProdKey> {
    let doc = std::fs::read_to_string(keys_dir().join(format!("{index}.json"))).ok()?;
    let section = &doc[doc.find(role)?..];
    let public_key = PublicKey::try_from_slice(&hex_field(section, "public_key")).ok()?;
    let secret = SecretKeyMaterial::from_xmss_ssz(hex_field(section, "secret_key")).ok()?;
    Some(ProdKey { public_key, secret })
}

pub fn sign(key: &ProdKey, slot: u32, message: &[u8; 32]) -> Signature {
    ProductionBackend
        .sign(&key.secret, slot, message)
        .expect("native XMSS sign")
}
