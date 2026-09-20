//! Parse Hive / lean-quickstart `validators.yaml` node → index assignments.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Indices assigned to a node id in the validator registry file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorAssignment {
    pub node_id: String,
    pub indices: Vec<u64>,
}

/// Load the registry and return the assignment for `node_id` (empty if missing).
pub fn load_validator_assignment(
    path: &Path,
    node_id: &str,
) -> Result<ValidatorAssignment, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("read {}: {e}", path.display()))?;
    parse_validator_assignment(&text, node_id)
}

/// Parse registry YAML text for one node id.
pub fn parse_validator_assignment(
    text: &str,
    node_id: &str,
) -> Result<ValidatorAssignment, String> {
    let map: BTreeMap<String, serde_yaml::Value> = serde_yaml::from_str(text)
        .map_err(|e| format!("validators.yaml: {e}"))?;
    let Some(value) = map.get(node_id) else {
        return Ok(ValidatorAssignment {
            node_id: node_id.to_string(),
            indices: Vec::new(),
        });
    };
    let indices = extract_indices(value)?;
    Ok(ValidatorAssignment {
        node_id: node_id.to_string(),
        indices,
    })
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
                        // Dual-key Ream rows repeat the same index twice; keep unique order.
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
    privkey_file: "a.ssz"
  - index: 1
    pubkey_hex: "bb"
    privkey_file: "b.ssz"
"#;
        let a = parse_validator_assignment(yaml, "ethean_0").unwrap();
        assert_eq!(a.indices, vec![1]);
    }
}
