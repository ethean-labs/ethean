//! Console / file log verbosity for the ethean binary.

use tracing_subscriber::EnvFilter;

/// Quiet default: INFO for Ethean, WARN for noisy libp2p crates.
pub const DEFAULT_FILTER: &str = "info,\
libp2p=warn,\
libp2p_gossipsub=warn,\
libp2p_quic=warn,\
libp2p_swarm=warn,\
libp2p_core=warn,\
libp2p_identify=warn,\
libp2p_ping=warn,\
multistream_select=warn";

/// `-v`: Ethean DEBUG, still quiet libp2p heartbeats.
pub const VERBOSE_FILTER: &str = "debug,\
libp2p=warn,\
libp2p_gossipsub=warn,\
libp2p_quic=warn,\
libp2p_swarm=warn,\
libp2p_core=warn,\
libp2p_identify=warn,\
libp2p_ping=warn,\
multistream_select=warn";

/// Build an [`EnvFilter`]. `RUST_LOG` wins when set and non-empty.
pub fn build_filter(verbose: u8, log_level: Option<&str>) -> Result<EnvFilter, String> {
    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        if !rust_log.trim().is_empty() {
            return EnvFilter::try_new(rust_log)
                .map_err(|e| format!("invalid RUST_LOG: {e}"));
        }
    }
    let directive = match log_level.map(|s| s.trim().to_ascii_lowercase()) {
        Some(ref level) if !level.is_empty() => {
            validate_level(level)?;
            if matches!(level.as_str(), "trace") || verbose >= 2 {
                level.clone()
            } else {
                format!("{level},libp2p=warn,libp2p_gossipsub=warn,libp2p_quic=warn,libp2p_swarm=warn,libp2p_core=warn")
            }
        }
        _ => match verbose {
            0 => DEFAULT_FILTER.to_string(),
            1 => VERBOSE_FILTER.to_string(),
            2 => "debug".to_string(),
            _ => "trace".to_string(),
        },
    };
    EnvFilter::try_new(directive).map_err(|e| format!("invalid log filter: {e}"))
}

fn validate_level(level: &str) -> Result<(), String> {
    match level {
        "error" | "warn" | "info" | "debug" | "trace" | "off" => Ok(()),
        _ => Err(format!(
            "unknown --log-level {level:?} (use error|warn|info|debug|trace|off)"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_parses() {
        assert!(build_filter(0, None).is_ok());
        assert!(build_filter(1, None).is_ok());
        assert!(build_filter(2, None).is_ok());
        assert!(build_filter(3, None).is_ok());
    }

    #[test]
    fn rejects_bad_level() {
        assert!(build_filter(0, Some("loud")).is_err());
    }
}
