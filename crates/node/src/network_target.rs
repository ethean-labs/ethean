//! Network target for Lean pq-devnet joins (bootnodes + label).

use std::fs;
use std::path::{Path, PathBuf};

/// Supported network labels for `ethean start --network`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkId {
    /// Interop generation aimed at leanroadmap pq-devnet-5 (in progress).
    PqDevnet5,
    /// Explicit local-only smoke (same profile, never expects mesh).
    Local,
}

impl NetworkId {
    /// Parse CLI / config network name.
    pub fn parse(name: &str) -> Result<Self, String> {
        match name.trim().to_ascii_lowercase().as_str() {
            "pq-devnet-5" | "pq_devnet_5" | "devnet5" | "devnet-5" => Ok(Self::PqDevnet5),
            "local" | "smoke" | "lstar" => Ok(Self::Local),
            other => Err(format!(
                "unknown network '{other}' (expected pq-devnet-5 or local)"
            )),
        }
    }

    /// Stable CLI string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PqDevnet5 => "pq-devnet-5",
            Self::Local => "local",
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
}

impl NetworkTarget {
    /// Default: pq-devnet-5 label with empty bootnodes (offline until supplied).
    pub fn pq_devnet_5() -> Self {
        Self {
            id: NetworkId::PqDevnet5,
            bootnodes: Vec::new(),
        }
    }

    /// Build from CLI network name + optional comma-separated bootnodes.
    pub fn from_cli(network: &str, bootnodes_csv: Option<&str>) -> Result<Self, String> {
        let id = NetworkId::parse(network)?;
        let mut bootnodes = parse_bootnode_csv(bootnodes_csv.unwrap_or(""));
        if bootnodes.is_empty() {
            if let Ok(env) = std::env::var("ETHEAN_BOOTNODES") {
                bootnodes = parse_bootnode_csv(&env);
            }
        }
        if bootnodes.is_empty() && id == NetworkId::PqDevnet5 {
            bootnodes = load_bootnodes_file(&default_bootnodes_path())?;
        }
        Ok(Self { id, bootnodes })
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

fn default_bootnodes_path() -> PathBuf {
    PathBuf::from("config/networks/pq-devnet-5.bootnodes")
}

fn load_bootnodes_file(path: &Path) -> Result<Vec<String>, String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_devnet5_aliases() {
        assert_eq!(NetworkId::parse("pq-devnet-5").unwrap(), NetworkId::PqDevnet5);
        assert_eq!(NetworkId::parse("devnet5").unwrap(), NetworkId::PqDevnet5);
        assert_eq!(NetworkId::parse("local").unwrap(), NetworkId::Local);
    }

    #[test]
    fn csv_bootnodes() {
        let t = NetworkTarget::from_cli(
            "local",
            Some("/ip4/1.2.3.4/udp/9/quic-v1, /ip4/5.6.7.8/udp/9/quic-v1"),
        )
        .unwrap();
        assert_eq!(t.bootnodes.len(), 2);
    }
}
