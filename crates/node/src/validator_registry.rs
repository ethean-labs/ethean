//! Parse Hive / lean-quickstart `validators.yaml` assignments and key rows.

use ethean_validator::SigningRole;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Indices assigned to a node id in the validator registry file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorAssignment {
    pub node_id: String,
    pub indices: Vec<u64>,
}

/// One registry row that names a privkey file (Ream / ethlambda dual-key shape).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryKeyRow {
    pub index: u64,
    pub pubkey_hex: String,
    pub privkey_file: String,
    pub role: SigningRole,
}

/// Load the registry and return the assignment for `node_id` (empty if missing).
pub fn load_validator_assignment(
    path: &Path,
    node_id: &str,
) -> Result<ValidatorAssignment, String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    parse_validator_assignment(&text, node_id)
}

/// Parse registry YAML text for one node id.
pub fn parse_validator_assignment(
    text: &str,
    node_id: &str,
) -> Result<ValidatorAssignment, String> {
    let rows = parse_registry_key_rows(text, node_id)?;
    let mut indices = Vec::new();
    for row in &rows {
        if indices.last().copied() != Some(row.index) {
            indices.push(row.index);
        }
    }
    // Simple index-only registries still need a path through extract_indices.
    if indices.is_empty() {
        let map: BTreeMap<String, serde_yaml::Value> = serde_yaml::from_str(text)
            .map_err(|e| format!("validators.yaml: {e}"))?;
        if let Some(value) = map.get(node_id) {
            indices = extract_indices(value)?;
        }
    }
    Ok(ValidatorAssignment {
        node_id: node_id.to_string(),
        indices,
    })
}

/// Parse key rows (with `privkey_file`) for `node_id`.
pub fn load_registry_key_rows(
    path: &Path,
    node_id: &str,
) -> Result<Vec<RegistryKeyRow>, String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    parse_registry_key_rows(&text, node_id)
}

/// Parse key rows from YAML text.
pub fn parse_registry_key_rows(
    text: &str,
    node_id: &str,
) -> Result<Vec<RegistryKeyRow>, String> {
    let map: BTreeMap<String, serde_yaml::Value> =
        serde_yaml::from_str(text).map_err(|e| format!("validators.yaml: {e}"))?;
    let Some(value) = map.get(node_id) else {
        return Ok(Vec::new());
    };
    let serde_yaml::Value::Sequence(items) = value else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    let mut seen_for_index: BTreeMap<u64, usize> = BTreeMap::new();
    for item in items {
        let serde_yaml::Value::Mapping(m) = item else {
            continue;
        };
        let Some(idx) = m
            .get(serde_yaml::Value::String("index".into()))
            .and_then(|v| v.as_u64())
        else {
            continue;
        };
        let pubkey_hex = m
            .get(serde_yaml::Value::String("pubkey_hex".into()))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let privkey_file = m
            .get(serde_yaml::Value::String("privkey_file".into()))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if privkey_file.is_empty() {
            continue;
        }
        let role = infer_role(&privkey_file, seen_for_index.get(&idx).copied().unwrap_or(0));
        *seen_for_index.entry(idx).or_insert(0) += 1;
        out.push(RegistryKeyRow {
            index: idx,
            pubkey_hex,
            privkey_file,
            role,
        });
    }
    Ok(out)
}

fn infer_role(privkey_file: &str, ordinal_for_index: usize) -> SigningRole {
    let lower = privkey_file.to_ascii_lowercase();
    if lower.contains("proposal") || lower.contains("proposer") {
        SigningRole::Proposal
    } else if lower.contains("attestation") || lower.contains("attester") {
        SigningRole::Attestation
    } else if ordinal_for_index == 0 {
        SigningRole::Attestation
    } else {
        SigningRole::Proposal
    }
}

fn extract_indices(value: &serde_yaml::Value) -> Result<Vec<u64>, String> {
    match value {
        serde_yaml::Value::Sequence(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    serde_yaml::Value::Number(n) => {
                        let i = n
                            .as_u64()
                            .ok_or_else(|| format!("invalid validator index {n}"))?;
                        out.push(i);
                    }
                    serde_yaml::Value::Mapping(m) => {
                        let idx = m
                            .get(serde_yaml::Value::String("index".into()))
                            .and_then(|v| v.as_u64())
                            .ok_or_else(|| "registry entry missing index".to_string())?;
                        if out.last().copied() != Some(idx) {
                            out.push(idx);
                        }
                    }
                    other => {
                        return Err(format!("unsupported validators.yaml entry: {other:?}"));
                    }
                }
            }
            Ok(out)
        }
        serde_yaml::Value::Null => Ok(Vec::new()),
        other => Err(format!("expected list for node assignment, got {other:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_index_list() {
        let yaml = "ethean_0:\n  - 0\n  - 2\nethean_1: []\n";
        let a = parse_validator_assignment(yaml, "ethean_0").unwrap();
        assert_eq!(a.indices, vec![0, 2]);
        let b = parse_validator_assignment(yaml, "ethean_1").unwrap();
        assert!(b.indices.is_empty());
    }

    #[test]
    fn parses_ream_dual_key_rows() {
        let yaml = r#"
ethean_0:
  - index: 1
    pubkey_hex: "aa"
    privkey_file: "validator_1_attestation_sk.ssz"
  - index: 1
    pubkey_hex: "bb"
    privkey_file: "validator_1_proposal_sk.ssz"
"#;
        let a = parse_validator_assignment(yaml, "ethean_0").unwrap();
        assert_eq!(a.indices, vec![1]);
        let rows = parse_registry_key_rows(yaml, "ethean_0").unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].role, SigningRole::Attestation);
        assert_eq!(rows[1].role, SigningRole::Proposal);
    }
}
