//! Pure helpers that turn `ethean start` flags into start-time decisions.
//!
//! Kept free of I/O side effects (except node-key files) so the Hive /
//! lean-quickstart flag contract is unit-testable without a running node.

use crate::start_config::LocalRoles;
use ethean_network::NodeKey;
use std::path::{Path, PathBuf};

/// Where genesis comes from once `--network` may name a `config.yaml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenesisSource {
    /// Network label handed to [`crate::NetworkTarget::from_cli`].
    pub label: String,
    /// Lean `config.yaml` path when a genesis file was given.
    pub genesis_path: Option<PathBuf>,
}

/// `--network <label|path>` plus optional `--lean-config`.
///
/// A file path for `--network` means "genesis from this config.yaml" under the
/// `local` profile (no built-in bootnodes). `--lean-config` wins when both name
/// a file.
pub fn resolve_genesis_source(network: &str, lean_config: Option<&str>) -> GenesisSource {
    let network_is_file = Path::new(network.trim()).is_file();
    let label = if network_is_file {
        "local".to_string()
    } else {
        network.trim().to_string()
    };
    let genesis_path = lean_config
        .map(PathBuf::from)
        .or_else(|| network_is_file.then(|| PathBuf::from(network.trim())));
    GenesisSource {
        label,
        genesis_path,
    }
}

/// Raw role flags from the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RoleFlags {
    pub validators: usize,
    pub no_aggregator: bool,
    pub is_aggregator: bool,
    pub no_local_finality: bool,
}

/// Roles for this start.
///
/// With a genesis file (`mesh` mode, Hive / quickstart) the aggregator role is
/// off unless `--is-aggregator` is present and local finality is off. Without
/// one (solo smoke) the historical defaults stay: both on unless `--no-*`.
pub fn resolve_roles(mesh: bool, flags: RoleFlags) -> LocalRoles {
    let validators = flags.validators.max(1);
    if mesh {
        LocalRoles {
            validators,
            is_aggregator: flags.is_aggregator && !flags.no_aggregator,
            local_finality: false,
        }
    } else {
        LocalRoles {
            validators,
            is_aggregator: !flags.no_aggregator,
            local_finality: !flags.no_local_finality,
        }
    }
}

/// `--aggregate-subnet-ids 0,1,2` → sorted unique ids.
pub fn parse_subnet_ids(csv: Option<&str>) -> Result<Vec<u64>, String> {
    let Some(csv) = csv else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for part in csv.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let id: u64 = part
            .parse()
            .map_err(|e| format!("--aggregate-subnet-ids entry '{part}': {e}"))?;
        if !out.contains(&id) {
            out.push(id);
        }
    }
    out.sort_unstable();
    Ok(out)
}

/// Where the node key came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKeySource {
    /// `--node-key <file>`.
    File(PathBuf),
    /// `<data-dir>/node.key` (`created` when generated on this start).
    DataDir { path: PathBuf, created: bool },
    /// Generated for this process only.
    Ephemeral,
}

/// Node key precedence: `--node-key` file, then `<data-dir>/node.key`, then per-run.
pub fn resolve_node_key(
    node_key: Option<&str>,
    data_dir: Option<&str>,
    ephemeral: bool,
) -> Result<(NodeKey, NodeKeySource), String> {
    if let Some(path) = node_key {
        let path = PathBuf::from(path);
        let key = NodeKey::load_file(&path).map_err(|e| format!("--node-key: {e}"))?;
        return Ok((key, NodeKeySource::File(path)));
    }
    if let (Some(dir), false) = (data_dir, ephemeral) {
        let path = Path::new(dir).join("node.key");
        let (key, created) =
            NodeKey::load_or_create(&path).map_err(|e| format!("data-dir node.key: {e}"))?;
        return Ok((key, NodeKeySource::DataDir { path, created }));
    }
    Ok((NodeKey::generate(), NodeKeySource::Ephemeral))
}

/// PeerId string for a node key (only derivable with the QUIC swarm compiled in).
pub fn node_key_peer_id(key: &NodeKey) -> Option<String> {
    #[cfg(feature = "libp2p-quic")]
    {
        return key.peer_id().ok().map(|p| p.to_string());
    }
    #[cfg(not(feature = "libp2p-quic"))]
    {
        let _ = key;
        None
    }
}

/// Flag value, else environment variable, else default (empty env is ignored).
pub fn flag_env_or(flag: Option<String>, env: &str, default: &str) -> String {
    if let Some(v) = flag.filter(|s| !s.trim().is_empty()) {
        return v;
    }
    match std::env::var(env) {
        Ok(v) if !v.trim().is_empty() => v.trim().to_string(),
        _ => default.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_path_means_local_genesis() {
        let dir = std::env::temp_dir().join(format!("ethean-cli-resolve-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cfg = dir.join("config.yaml");
        std::fs::write(&cfg, "GENESIS_TIME: 1\n").unwrap();
        let src = resolve_genesis_source(cfg.to_str().unwrap(), None);
        assert_eq!(src.label, "local");
        assert_eq!(src.genesis_path.as_deref(), Some(cfg.as_path()));
        let src = resolve_genesis_source("pq-devnet-5", None);
        assert_eq!(src.label, "pq-devnet-5");
        assert!(src.genesis_path.is_none());
        let src = resolve_genesis_source("pq-devnet-5", Some("/x/config.yaml"));
        assert_eq!(src.label, "pq-devnet-5");
        assert_eq!(src.genesis_path, Some(PathBuf::from("/x/config.yaml")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mesh_roles_default_off_unless_is_aggregator() {
        let base = RoleFlags {
            validators: 4,
            ..RoleFlags::default()
        };
        let r = resolve_roles(true, base);
        assert!(!r.is_aggregator && !r.local_finality);
        let r = resolve_roles(
            true,
            RoleFlags {
                is_aggregator: true,
                ..base
            },
        );
        assert!(r.is_aggregator && !r.local_finality);
        let r = resolve_roles(
            true,
            RoleFlags {
                is_aggregator: true,
                no_aggregator: true,
                ..base
            },
        );
        assert!(!r.is_aggregator);
    }

    #[test]
    fn solo_roles_keep_legacy_defaults() {
        let r = resolve_roles(false, RoleFlags::default());
        assert!(r.is_aggregator && r.local_finality);
        assert_eq!(r.validators, 1);
        let r = resolve_roles(
            false,
            RoleFlags {
                no_aggregator: true,
                no_local_finality: true,
                is_aggregator: false,
                validators: 0,
            },
        );
        assert!(!r.is_aggregator && !r.local_finality);
    }

    #[test]
    fn subnet_ids_csv() {
        assert_eq!(parse_subnet_ids(None).unwrap(), Vec::<u64>::new());
        assert_eq!(parse_subnet_ids(Some("2, 0,1,2")).unwrap(), vec![0, 1, 2]);
        assert!(parse_subnet_ids(Some("a")).is_err());
    }

    #[test]
    fn node_key_precedence() {
        let dir = std::env::temp_dir().join(format!("ethean-node-key-src-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("given.key");
        std::fs::write(&file, format!("0x{}\n", "ab".repeat(32))).unwrap();
        let (k, src) = resolve_node_key(file.to_str(), dir.to_str(), false).unwrap();
        assert_eq!(k.to_hex(), "ab".repeat(32));
        assert_eq!(src, NodeKeySource::File(file));
        let (k1, src) = resolve_node_key(None, dir.to_str(), false).unwrap();
        assert!(matches!(src, NodeKeySource::DataDir { created: true, .. }));
        let (k2, src) = resolve_node_key(None, dir.to_str(), false).unwrap();
        assert!(matches!(src, NodeKeySource::DataDir { created: false, .. }));
        assert_eq!(k1, k2);
        let (_, src) = resolve_node_key(None, dir.to_str(), true).unwrap();
        assert_eq!(src, NodeKeySource::Ephemeral);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn flag_env_default_order() {
        std::env::set_var("ETHEAN_TEST_ADDR", "10.0.0.1");
        assert_eq!(
            flag_env_or(Some("1.1.1.1".into()), "ETHEAN_TEST_ADDR", "d"),
            "1.1.1.1"
        );
        assert_eq!(flag_env_or(None, "ETHEAN_TEST_ADDR", "d"), "10.0.0.1");
        std::env::remove_var("ETHEAN_TEST_ADDR");
        assert_eq!(flag_env_or(None, "ETHEAN_TEST_ADDR", "d"), "d");
    }
}
