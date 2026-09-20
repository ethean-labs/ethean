//! Start / run mode configuration for the node client.

use crate::network_target::NetworkTarget;
use std::net::SocketAddr;

/// How `EtheanClient::start` drives duty ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    /// Advance elapsed time in-process (no wall sleep); default smoke path.
    SmokeElapsed {
        /// Number of interval ticks to process.
        ticks: u32,
    },
    /// Sample the wall clock (optional sleep between intervals).
    WallClock {
        /// Number of wall samples / ticks to process.
        ticks: u32,
        /// When false, do not sleep (tests / dry runs).
        enable_sleep: bool,
    },
    /// Wall-clock loop until Ctrl-C / process signal (binary long-run).
    UntilSignal {
        /// Sleep between interval boundaries.
        enable_sleep: bool,
    },
}

impl Default for RunMode {
    fn default() -> Self {
        Self::SmokeElapsed { ticks: 5 }
    }
}

/// Optional Prometheus scrape listener.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricsListen {
    /// Bind address (default `127.0.0.1:9100`).
    pub addr: SocketAddr,
}

impl Default for MetricsListen {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:9100".parse().expect("static addr"),
        }
    }
}

/// Optional Lean `/lean/v1` HTTP listener.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpcListen {
    /// Bind address (default `127.0.0.1:5052`).
    pub addr: SocketAddr,
    /// Bearer token for admin routes on non-loopback binds.
    pub admin_token: String,
}

impl Default for RpcListen {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:5052".parse().expect("static addr"),
            admin_token: String::new(),
        }
    }
}

/// Local validator / finality roles for long-run smoke (no public mesh required).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalRoles {
    /// Registry size for recent local genesis.
    pub validators: usize,
    /// Lean aggregator role.
    pub is_aggregator: bool,
    /// Self-apply proposals + inject full-registry votes.
    pub local_finality: bool,
}

impl Default for LocalRoles {
    fn default() -> Self {
        Self {
            validators: 4,
            is_aggregator: true,
            local_finality: true,
        }
    }
}

/// Bundle passed into client start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartConfig {
    /// Duty run mode.
    pub mode: RunMode,
    /// Network label + bootnodes (default pq-devnet-4).
    pub network: NetworkTarget,
    /// When `Some`, spawn `/metrics` HTTP on this address.
    pub metrics: Option<MetricsListen>,
    /// When `Some`, spawn Lean `/lean/v1` HTTP on this address.
    pub http: Option<RpcListen>,
    /// UDP/QUIC listen port (`0` = OS ephemeral; default `9000` for Hive/mesh).
    pub listen_port: u16,
    /// Local genesis size and aggregator/finality flags.
    pub roles: LocalRoles,
}

impl Default for StartConfig {
    fn default() -> Self {
        Self {
            mode: RunMode::default(),
            network: NetworkTarget::pq_devnet_4(),
            metrics: Some(MetricsListen::default()),
            http: Some(RpcListen::default()),
            listen_port: 9000,
            roles: LocalRoles::default(),
        }
    }
}

impl StartConfig {
    /// Smoke elapsed loop with `ticks` intervals (pq-devnet-4 label).
    pub fn smoke(ticks: u32) -> Self {
        Self {
            mode: RunMode::SmokeElapsed { ticks },
            ..Self::default()
        }
    }

    /// Wall-clock loop; `enable_sleep` should be true for the binary.
    pub fn wall(ticks: u32, enable_sleep: bool) -> Self {
        Self {
            mode: RunMode::WallClock {
                ticks,
                enable_sleep,
            },
            ..Self::default()
        }
    }

    /// Run until Ctrl-C with wall-clock sleeps between intervals.
    pub fn until_signal(enable_sleep: bool) -> Self {
        Self {
            mode: RunMode::UntilSignal { enable_sleep },
            ..Self::default()
        }
    }

    /// Attach a resolved network target.
    pub fn with_network(mut self, network: NetworkTarget) -> Self {
        self.network = network;
        self
    }

    /// Enable or disable the metrics scrape listener.
    pub fn with_metrics(mut self, metrics: Option<MetricsListen>) -> Self {
        self.metrics = metrics;
        self
    }

    /// Enable or disable the Lean HTTP API listener.
    pub fn with_http(mut self, http: Option<RpcListen>) -> Self {
        self.http = http;
        self
    }

    /// Set UDP/QUIC listen port (`0` = ephemeral).
    pub fn with_listen_port(mut self, listen_port: u16) -> Self {
        self.listen_port = listen_port;
        self
    }

    /// Set local validator count and aggregator/finality roles.
    pub fn with_roles(mut self, roles: LocalRoles) -> Self {
        self.roles = roles;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network_target::NetworkId;

    #[test]
    fn defaults_to_smoke_five_on_pq_devnet_4() {
        let cfg = StartConfig::default();
        assert_eq!(cfg.mode, RunMode::SmokeElapsed { ticks: 5 });
        assert_eq!(cfg.network.id, NetworkId::PqDevnet4);
        assert!(cfg.metrics.is_some());
        assert!(cfg.http.is_some());
        assert_eq!(cfg.listen_port, 9000);
        assert!(cfg.roles.local_finality);
        assert_eq!(cfg.roles.validators, 4);
    }

    #[test]
    fn until_signal_mode() {
        assert!(matches!(
            StartConfig::until_signal(true).mode,
            RunMode::UntilSignal { enable_sleep: true }
        ));
    }

    #[test]
    fn with_network_keeps_pq_devnet_5_ready() {
        let cfg = StartConfig::smoke(1).with_network(NetworkTarget::pq_devnet_5());
        assert_eq!(cfg.network.id, NetworkId::PqDevnet5);
    }
}
