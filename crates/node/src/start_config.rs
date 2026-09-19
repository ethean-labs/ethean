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

/// Bundle passed into client start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartConfig {
    /// Duty run mode.
    pub mode: RunMode,
    /// Network label + bootnodes (default pq-devnet-4).
    pub network: NetworkTarget,
    /// When `Some`, spawn `/metrics` HTTP on this address.
    pub metrics: Option<MetricsListen>,
    /// Validator registry size for local genesis (default 4).
    pub validators: usize,
    /// Lean aggregator role (collect/prove aggregates).
    pub is_aggregator: bool,
    /// Self-apply proposals + full-registry votes so head/finality move solo.
    pub local_finality: bool,
}

impl Default for StartConfig {
    fn default() -> Self {
        Self {
            mode: RunMode::default(),
            network: NetworkTarget::pq_devnet_4(),
            metrics: Some(MetricsListen::default()),
            validators: 4,
            is_aggregator: true,
            local_finality: true,
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

    /// Set local genesis validator count.
    pub fn with_validators(mut self, validators: usize) -> Self {
        self.validators = validators.max(1);
        self
    }

    /// Aggregator + local finality switches.
    pub fn with_roles(mut self, is_aggregator: bool, local_finality: bool) -> Self {
        self.is_aggregator = is_aggregator;
        self.local_finality = local_finality;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network_target::NetworkId;

    #[test]
    fn defaults_enable_local_finality() {
        let cfg = StartConfig::default();
        assert_eq!(cfg.network.id, NetworkId::PqDevnet4);
        assert!(cfg.local_finality);
        assert!(cfg.is_aggregator);
        assert_eq!(cfg.validators, 4);
    }
}
