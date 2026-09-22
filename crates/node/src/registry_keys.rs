//! Load Hive registry privkey files into in-memory key handles.

use crate::validator_registry::{load_registry_key_rows, RegistryKeyRow};
use ethean_crypto::{SecretKeyMaterial, PUBLIC_KEY_BYTES};
use ethean_genesis::decode_hex_fixed;
use ethean_validator::{KeyId, KeyRecord, SigningRole};
use std::fs;
use std::path::{Path, PathBuf};

/// Loaded proposal + attestation secrets for one Hive node id.
#[derive(Debug, Clone)]
pub struct LoadedNodeKeys {
    pub node_id: String,
    pub indices: Vec<u64>,
    pub proposal: Option<KeyRecord>,
    pub attestation: Option<KeyRecord>,
}

/// Load key rows for `node_id` and read `privkey_file` bytes beside the registry.
pub fn load_node_keys(registry_path: &Path, node_id: &str) -> Result<LoadedNodeKeys, String> {
    let rows = load_registry_key_rows(registry_path, node_id)?;
    let base = registry_path.parent().unwrap_or_else(|| Path::new("."));
    let mut indices = Vec::new();
    let mut proposal = None;
    let mut attestation = None;
    for row in rows {
        if indices.last().copied() != Some(row.index) {
            indices.push(row.index);
        }
        let record = load_key_record(base, &row)?;
        match row.role {
            SigningRole::Proposal => proposal = Some(record),
            SigningRole::Attestation => attestation = Some(record),
        }
    }
    Ok(LoadedNodeKeys {
        node_id: node_id.to_string(),
        indices,
        proposal,
        attestation,
    })
}

fn load_key_record(base: &Path, row: &RegistryKeyRow) -> Result<KeyRecord, String> {
    let path = resolve_privkey_path(base, &row.privkey_file)?;
    let secret_bytes = fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    if secret_bytes.is_empty() {
        return Err(format!("empty privkey file {}", path.display()));
    }
    let secret = SecretKeyMaterial::from_xmss_ssz(secret_bytes)
        .map_err(|e| format!("decode XMSS privkey {}: {e}", path.display()))?;
    let derived = secret
        .xmss_public_key()
        .map_err(|e| format!("derive pubkey from {}: {e}", path.display()))?;
    if !row.pubkey_hex.trim().is_empty() {
        let declared = decode_hex_fixed::<PUBLIC_KEY_BYTES>(&row.pubkey_hex)
            .map_err(|e| format!("pubkey_hex for index {}: {e}", row.index))?;
        if declared != *derived.as_bytes() {
            return Err(format!(
                "pubkey_hex for index {} does not match the key derived from {}",
                row.index,
                path.display()
            ));
        }
    }
    let activation_slot = secret.activation_epoch();
    let num_active_slots = secret.num_active_epochs();
    let mut key_id_bytes = [0u8; 16];
    key_id_bytes[..8].copy_from_slice(&row.index.to_le_bytes());
    key_id_bytes[8] = match row.role {
        SigningRole::Attestation => 0xa1,
        SigningRole::Proposal => 0xb2,
    };
    Ok(KeyRecord {
        key_id: KeyId::from_bytes(key_id_bytes),
        role: row.role,
        public_key: derived,
        secret,
        activation_slot,
        num_active_slots,
        journal_generation: 1,
    })
}

fn resolve_privkey_path(base: &Path, file: &str) -> Result<PathBuf, String> {
    let direct = base.join(file);
    if direct.is_file() {
        return Ok(direct);
    }
    let hashed = base.join("hash-sig-keys").join(file);
    if hashed.is_file() {
        return Ok(hashed);
    }
    Err(format!(
        "privkey file not found at {} or {}",
        direct.display(),
        hashed.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    fn test_key_hex(index: usize, field: &str) -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../crypto/testdata/xmss_test_keys")
            .join(format!("{index}.json"));
        let doc = fs::read_to_string(path).expect("vendored XMSS test key");
        let needle = format!("\"{field}\": \"");
        let start = doc.find(&needle).unwrap() + needle.len();
        let end = doc[start..].find('"').unwrap() + start;
        doc[start..end].to_string()
    }

    fn hex_bytes(hex: &str) -> Vec<u8> {
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect()
    }

    fn write_registry(dir: &Path, att_pk: &str, prop_pk: &str) -> PathBuf {
        let keys = dir.join("hash-sig-keys");
        fs::create_dir_all(&keys).unwrap();
        fs::write(
            keys.join("att.ssz"),
            hex_bytes(&test_key_hex(0, "attestation_secret")),
        )
        .unwrap();
        fs::write(
            keys.join("prop.ssz"),
            hex_bytes(&test_key_hex(0, "proposal_secret")),
        )
        .unwrap();
        let yaml = format!(
            "ethean_0:\n  - index: 0\n    pubkey_hex: \"{att_pk}\"\n    privkey_file: \"att.ssz\"\n  - index: 0\n    pubkey_hex: \"{prop_pk}\"\n    privkey_file: \"prop.ssz\"\n"
        );
        let reg = dir.join("validators.yaml");
        let mut f = fs::File::create(&reg).unwrap();
        f.write_all(yaml.as_bytes()).unwrap();
        reg
    }

    #[test]
    fn loads_proposal_and_attestation_files() {
        let dir = tempdir().unwrap();
        let att_pk = test_key_hex(0, "attestation_public");
        let prop_pk = test_key_hex(0, "proposal_public");
        let reg = write_registry(dir.path(), &att_pk, &prop_pk);
        let loaded = load_node_keys(&reg, "ethean_0").unwrap();
        assert_eq!(loaded.indices, vec![0]);
        let prop = loaded.proposal.as_ref().unwrap();
        assert_eq!(prop.public_key.as_bytes().to_vec(), hex_bytes(&prop_pk));
        assert_eq!(prop.activation_slot, 0);
        assert_eq!(prop.num_active_slots, 112);
        assert_eq!(
            loaded
                .attestation
                .as_ref()
                .unwrap()
                .public_key
                .as_bytes()
                .to_vec(),
            hex_bytes(&att_pk)
        );
    }

    #[test]
    fn rejects_pubkey_that_does_not_match_privkey() {
        let dir = tempdir().unwrap();
        let att_pk = test_key_hex(0, "attestation_public");
        let wrong = test_key_hex(1, "proposal_public");
        let reg = write_registry(dir.path(), &att_pk, &wrong);
        let err = load_node_keys(&reg, "ethean_0").unwrap_err();
        assert!(err.contains("does not match"), "{err}");
    }
}
