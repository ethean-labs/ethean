//! Start / run mode configuration for the node client.

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartConfig {
    /// Duty run mode.
    pub mode: RunMode,
}

impl Default for StartConfig {
    fn default() -> Self {
        Self {
            mode: RunMode::default(),
        }
    }
}

impl StartConfig {
    /// Smoke elapsed loop with `ticks` intervals.
    pub fn smoke(ticks: u32) -> Self {
        Self {
            mode: RunMode::SmokeElapsed { ticks },
        }
    }

    /// Wall-clock loop; `enable_sleep` should be true for the binary.
    pub fn wall(ticks: u32, enable_sleep: bool) -> Self {
        Self {
            mode: RunMode::WallClock {
                ticks,
                enable_sleep,
            },
        }
    }

    /// Run until Ctrl-C with wall-clock sleeps between intervals.
    pub fn until_signal(enable_sleep: bool) -> Self {
        Self {
            mode: RunMode::UntilSignal { enable_sleep },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_smoke_five() {
        assert_eq!(
            StartConfig::default().mode,
            RunMode::SmokeElapsed { ticks: 5 }
        );
    }

    #[test]
    fn until_signal_mode() {
        assert!(matches!(
            StartConfig::until_signal(true).mode,
            RunMode::UntilSignal { enable_sleep: true }
        ));
    }
}
