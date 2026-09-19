//! Network target for Lean pq-devnet joins (bootnodes + label + fork digest).

use std::fs;
use std::path::{Path, PathBuf};

/// Supported network labels for `ethean start --network`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkId {
    /// Operational join target while pq-devnet-5 has no public mesh.
    PqDevnet4,
    /// Next-generation label; code paths stay ready for operator bootnodes.
    PqDevnet5,
    /// Explicit local-only smoke (same profile, never expects mesh).
    Local,
}

impl NetworkId {
    /// Parse CLI / config network name.
    pub fn parse(name: &str) -> Result<Self, String> {
        match name.trim().to_ascii_lowercase().as_str() {
            "pq-devnet-4" | "pq_devnet_4" | "devnet4" | "devnet-4" => Ok(Self::PqDevnet4),
            "pq-devnet-5" | "pq_devnet_5" | "devnet5" | "devnet-5" => Ok(Self::PqDevnet5),
            "local" | "smoke" | "lstar" => Ok(Self::Local),
            other => Err(format!(
                "unknown network '{other}' (expected pq-devnet-4, pq-devnet-5, or local)"
            )),
        }
    }

    /// Stable CLI string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PqDevnet4 => "pq-devnet-4",
            Self::PqDevnet5 => "pq-devnet-5",
            Self::Local => "local",
        }
    }

    /// Config basename under `config/networks/` (None for local smoke).
    pub fn config_stem(self) -> Option<&'static str> {
        match self {
            Self::PqDevnet4 => Some("pq-devnet-4"),
            Self::PqDevnet5 => Some("pq-devnet-5"),
            Self::Local => None,
        }
    }
}

/// Resolved start-time network intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkTarget {
    /// Interop / smoke label.
    pub id: NetworkId,
    /// QUIC multiaddrs to dial after swarm bind.
    pub bootnodes: Vec<String>,
    /// Optional 8-hex operator fork digest (overrides interim name hash).
    pub fork_digest: Option<String>,
}

impl NetworkTarget {
    /// Default operational target: pq-devnet-4 (empty until file/CLI/env supplied).
    pub fn pq_devnet_4() -> Self {
        Self {
            id: NetworkId::PqDevnet4,
            bootnodes: Vec::new(),
            fork_digest: None,
        }
    }

    /// Ready-path target for pq-devnet-5 when an operator mesh opens.
    pub fn pq_devnet_5() -> Self {
        Self {
            id: NetworkId::PqDevnet5,
            bootnodes: Vec::new(),
            fork_digest: None,
        }
    }

    /// Build from CLI network name + optional bootnodes + optional fork digest.
    pub fn from_cli(
        network: &str,
        bootnodes_csv: Option<&str>,
        fork_digest: Option<&str>,
    ) -> Result<Self, String> {
        let id = NetworkId::parse(network)?;
        let mut bootnodes = parse_bootnode_csv(bootnodes_csv.unwrap_or(""));
        if bootnodes.is_empty() {
            if let Ok(env) = std::env::var("ETHEAN_BOOTNODES") {
                bootnodes = parse_bootnode_csv(&env);
            }
        }
        if bootnodes.is_empty() {
            if let Some(path) = bootnodes_path_for(id) {
                bootnodes = load_lines_file(&path)?;
            }
        }

        let mut digest = fork_digest
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        if digest.is_none() {
            if let Ok(env) = std::env::var("ETHEAN_FORK_DIGEST") {
                let t = env.trim().to_string();
                if !t.is_empty() {
                    digest = Some(t);
                }
            }
        }
        if digest.is_none() {
            if let Some(path) = forkdigest_path_for(id) {
                digest = load_single_line_file(&path)?;
            }
        }

        Ok(Self {
            id,
            bootnodes,
            fork_digest: digest,
        })
    }

    /// True when at least one dial target is configured.
    pub fn has_bootnodes(&self) -> bool {
        !self.bootnodes.is_empty()
    }
}

fn parse_bootnode_csv(raw: &str) -> Vec<String> {
    raw.split(|c| c == ',' || c == ';' || c == '\n')
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .map(|s| s.to_string())
        .collect()
}

fn bootnodes_path_for(id: NetworkId) -> Option<PathBuf> {
    id.config_stem()
        .map(|stem| PathBuf::from(format!("config/networks/{stem}.bootnodes")))
}

fn forkdigest_path_for(id: NetworkId) -> Option<PathBuf> {
    id.config_stem()
        .map(|stem| PathBuf::from(format!("config/networks/{stem}.forkdigest")))
}

fn load_lines_file(path: &Path) -> Result<Vec<String>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.to_string())
        .collect())
}

fn load_single_line_file(path: &Path) -> Result<Option<String>, String> {
    let lines = load_lines_file(path)?;
    Ok(lines.into_iter().next())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_devnet_aliases() {
        assert_eq!(NetworkId::parse("pq-devnet-4").unwrap(), NetworkId::PqDevnet4);
        assert_eq!(NetworkId::parse("devnet4").unwrap(), NetworkId::PqDevnet4);
        assert_eq!(NetworkId::parse("pq-devnet-5").unwrap(), NetworkId::PqDevnet5);
        assert_eq!(NetworkId::parse("devnet5").unwrap(), NetworkId::PqDevnet5);
        assert_eq!(NetworkId::parse("local").unwrap(), NetworkId::Local);
    }

    #[test]
    fn csv_bootnodes() {
        let t = NetworkTarget::from_cli(
            "local",
            Some("/ip4/1.2.3.4/udp/9/quic-v1, /ip4/5.6.7.8/udp/9/quic-v1"),
            None,
        )
        .unwrap();
        assert_eq!(t.bootnodes.len(), 2);
    }

    #[test]
    fn fork_digest_cli() {
        let t = NetworkTarget::from_cli("local", None, Some("0xAABBCCDD")).unwrap();
        assert_eq!(t.fork_digest.as_deref(), Some("0xAABBCCDD"));
    }

    #[test]
    fn config_stems_for_devnets() {
        assert_eq!(NetworkId::PqDevnet4.config_stem(), Some("pq-devnet-4"));
        assert_eq!(NetworkId::PqDevnet5.config_stem(), Some("pq-devnet-5"));
        assert_eq!(NetworkId::Local.config_stem(), None);
    }
}
