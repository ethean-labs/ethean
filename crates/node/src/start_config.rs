//! Start / run mode configuration for the node client.

use crate::network_target::NetworkTarget;

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

/// Bundle passed into client start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartConfig {
    /// Duty run mode.
    pub mode: RunMode,
    /// Network label + bootnodes (default pq-devnet-4).
    pub network: NetworkTarget,
}

impl Default for StartConfig {
    fn default() -> Self {
        Self {
            mode: RunMode::default(),
            network: NetworkTarget::pq_devnet_4(),
        }
    }
}

impl StartConfig {
    /// Smoke elapsed loop with `ticks` intervals (pq-devnet-4 label).
    pub fn smoke(ticks: u32) -> Self {
        Self {
            mode: RunMode::SmokeElapsed { ticks },
            network: NetworkTarget::pq_devnet_4(),
        }
    }

    /// Wall-clock loop; `enable_sleep` should be true for the binary.
    pub fn wall(ticks: u32, enable_sleep: bool) -> Self {
        Self {
            mode: RunMode::WallClock {
                ticks,
                enable_sleep,
            },
            network: NetworkTarget::pq_devnet_4(),
        }
    }

    /// Run until Ctrl-C with wall-clock sleeps between intervals.
    pub fn until_signal(enable_sleep: bool) -> Self {
        Self {
            mode: RunMode::UntilSignal { enable_sleep },
            network: NetworkTarget::pq_devnet_4(),
        }
    }

    /// Attach a resolved network target.
    pub fn with_network(mut self, network: NetworkTarget) -> Self {
        self.network = network;
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
