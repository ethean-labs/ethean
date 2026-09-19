//! Network target for Lean pq-devnet joins (bootnodes + label + fork digest).

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
    /// Optional 8-hex operator fork digest (overrides interim name hash).
    pub fork_digest: Option<String>,
}

impl NetworkTarget {
    /// Default: pq-devnet-5 label with empty bootnodes (offline until supplied).
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
        if bootnodes.is_empty() && id == NetworkId::PqDevnet5 {
            bootnodes = load_lines_file(&default_bootnodes_path())?;
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
        if digest.is_none() && id == NetworkId::PqDevnet5 {
            digest = load_single_line_file(&default_forkdigest_path())?;
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

fn default_bootnodes_path() -> PathBuf {
    PathBuf::from("config/networks/pq-devnet-5.bootnodes")
}

fn default_forkdigest_path() -> PathBuf {
    PathBuf::from("config/networks/pq-devnet-5.forkdigest")
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
}
