//! `--bootnodes` value parsing: `none`, CSV of multiaddrs / ENRs, or a YAML file.

use ethean_network::enr_to_multiaddr;
use std::fs;
use std::path::Path;

/// How the operator expressed the bootnode list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootnodesSpec {
    /// Explicit `none`: no dial targets and no file / env fallback.
    None,
    /// Resolved QUIC multiaddrs (ENRs already converted).
    List(Vec<String>),
}

/// Parse a `--bootnodes` / `ETHEAN_BOOTNODES` value.
///
/// Accepted shapes: `none` (or empty), a CSV / `;` / newline list whose entries
/// are QUIC multiaddrs or `enr:` records, or a path to a YAML file holding a
/// list of such strings (lean-quickstart `nodes.yaml`).
pub fn parse_bootnodes_value(raw: &str) -> Result<BootnodesSpec, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(BootnodesSpec::List(Vec::new()));
    }
    if trimmed.eq_ignore_ascii_case("none") {
        return Ok(BootnodesSpec::None);
    }
    let path = Path::new(trimmed);
    if path.is_file() {
        return load_bootnodes_file(path).map(BootnodesSpec::List);
    }
    resolve_entries(split_list(trimmed)).map(BootnodesSpec::List)
}

/// Load a bootnodes file: YAML list of strings, or one entry per line.
pub fn load_bootnodes_file(path: &Path) -> Result<Vec<String>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let entries = match serde_yaml::from_str::<serde_yaml::Value>(&text) {
        Ok(serde_yaml::Value::Sequence(items)) => items
            .into_iter()
            .filter_map(|v| v.as_str().map(str::trim).map(str::to_string))
            .filter(|s| !s.is_empty())
            .collect(),
        _ => split_list(&text),
    };
    resolve_entries(entries).map_err(|e| format!("{}: {e}", path.display()))
}

fn split_list(raw: &str) -> Vec<String> {
    raw.split(|c| c == ',' || c == ';' || c == '\n')
        .map(str::trim)
        .map(|s| s.trim_start_matches("- ").trim())
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .map(|s| s.to_string())
        .collect()
}

fn resolve_entries(entries: Vec<String>) -> Result<Vec<String>, String> {
    let mut out = Vec::with_capacity(entries.len());
    for entry in entries {
        let e = entry.trim().trim_matches('"').trim_matches('\'');
        if e.eq_ignore_ascii_case("none") {
            continue;
        }
        if e.starts_with("enr:") {
            let addr = enr_to_multiaddr(e).map_err(|err| format!("bootnode ENR: {err}"))?;
            out.push(addr);
        } else if e.starts_with('/') {
            out.push(e.to_string());
        } else {
            return Err(format!(
                "bootnode entry '{e}' is neither a multiaddr, an enr:, nor an existing file"
            ));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENR: &str = "enr:-IW4QMn2QUYENcnsEpITZLph3YZee8Y3B92INUje_riQUOFQQ5Zm5kASi7E_IuQoGCWgcmCYrH920Q52kH7tQcWcPhEBgmlkgnY0gmlwhH8AAAGEcXVpY4IjKIlzZWNwMjU2azGhAhMMnGF1rmIPQ9tWgqfkNmvsG-aIyc9EJU5JFo3Tegys";
    const ENR_ADDR: &str =
        "/ip4/127.0.0.1/udp/9000/quic-v1/p2p/16Uiu2HAkvi2sxT75Bpq1c7yV2FjnSQJJ432d6jeshbmfdJss1i6f";

    #[test]
    fn none_and_empty() {
        assert_eq!(parse_bootnodes_value("none").unwrap(), BootnodesSpec::None);
        assert_eq!(
            parse_bootnodes_value(" NONE ").unwrap(),
            BootnodesSpec::None
        );
        assert_eq!(
            parse_bootnodes_value("").unwrap(),
            BootnodesSpec::List(Vec::new())
        );
    }

    #[test]
    fn csv_multiaddrs_and_enr() {
        let v = parse_bootnodes_value(&format!(
            "/ip4/1.2.3.4/udp/9/quic-v1, {ENR} ;/ip4/5.6.7.8/udp/9/quic-v1"
        ))
        .unwrap();
        assert_eq!(
            v,
            BootnodesSpec::List(vec![
                "/ip4/1.2.3.4/udp/9/quic-v1".into(),
                ENR_ADDR.into(),
                "/ip4/5.6.7.8/udp/9/quic-v1".into(),
            ])
        );
        assert!(parse_bootnodes_value("garbage").is_err());
    }

    #[test]
    fn yaml_nodes_file() {
        let dir = std::env::temp_dir().join(format!("ethean-bootnodes-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("nodes.yaml");
        fs::write(
            &path,
            format!("- {ENR}\n- \"/ip4/9.9.9.9/udp/9000/quic-v1/p2p/16Uiu2HAkvi2sxT75Bpq1c7yV2FjnSQJJ432d6jeshbmfdJss1i6f\"\n"),
        )
        .unwrap();
        let v = parse_bootnodes_value(path.to_str().unwrap()).unwrap();
        let BootnodesSpec::List(list) = v else {
            panic!("expected list");
        };
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], ENR_ADDR);
        assert!(list[1].starts_with("/ip4/9.9.9.9/"));
        let _ = fs::remove_dir_all(&dir);
    }
}
