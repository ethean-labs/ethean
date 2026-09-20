//! Load Hive registry privkey files into in-memory key handles.

use crate::validator_registry::{load_registry_key_rows, RegistryKeyRow};
use ethean_crypto::{PublicKey, SecretKeyMaterial, PUBLIC_KEY_BYTES};
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
    let base = registry_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
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
    let pk_bytes = if row.pubkey_hex.trim().is_empty() {
        [0u8; PUBLIC_KEY_BYTES]
    } else {
        decode_hex_fixed::<PUBLIC_KEY_BYTES>(&row.pubkey_hex)
            .map_err(|e| format!("pubkey_hex for index {}: {e}", row.index))?
    };
    let mut key_id_bytes = [0u8; 16];
    key_id_bytes[..8].copy_from_slice(&row.index.to_le_bytes());
    key_id_bytes[8] = match row.role {
        SigningRole::Attestation => 0xa1,
        SigningRole::Proposal => 0xb2,
    };
    Ok(KeyRecord {
        key_id: KeyId::from_bytes(key_id_bytes),
        role: row.role,
        public_key: PublicKey::from_bytes(pk_bytes),
        secret: SecretKeyMaterial::from_imported(secret_bytes, 0, 1_000_000),
        activation_slot: 0,
        num_active_slots: 1_000_000,
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

    #[test]
    fn loads_proposal_and_attestation_files() {
        let dir = tempdir().unwrap();
        let keys = dir.path().join("hash-sig-keys");
        fs::create_dir_all(&keys).unwrap();
        fs::write(keys.join("att.ssz"), [1u8; 8]).unwrap();
        fs::write(keys.join("prop.ssz"), [2u8; 8]).unwrap();
        let pk = "aa".repeat(52);
        let yaml = format!(
            "ethean_0:\n  - index: 0\n    pubkey_hex: \"{pk}\"\n    privkey_file: \"att.ssz\"\n  - index: 0\n    pubkey_hex: \"{pk}\"\n    privkey_file: \"prop.ssz\"\n"
        );
        let reg = dir.path().join("validators.yaml");
        let mut f = fs::File::create(&reg).unwrap();
        f.write_all(yaml.as_bytes()).unwrap();
        let loaded = load_node_keys(&reg, "ethean_0").unwrap();
        assert_eq!(loaded.indices, vec![0]);
        assert!(loaded.attestation.is_some());
        assert!(loaded.proposal.is_some());
        assert_eq!(
            loaded.proposal.as_ref().unwrap().secret.as_bytes(),
            &[2u8; 8]
        );
    }
}
