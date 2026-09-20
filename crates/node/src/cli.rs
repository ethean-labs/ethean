//! Command-line interface for Ethean

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "ethean")]
#[command(about = "Ethean Lean Consensus Client")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start the lean consensus node
    Start {
        /// Number of duty ticks before drain (ignored with --until-signal).
        #[arg(long, default_value_t = 5)]
        ticks: u32,
        /// Sample the system wall clock (sleeps between intervals).
        #[arg(long, default_value_t = false)]
        wall_clock: bool,
        /// Run until Ctrl-C (implies wall-clock sleeps). Long-run / Grafana path.
        #[arg(long, default_value_t = false)]
        until_signal: bool,
        /// Durable directory: genesis SSZ, ethean.redb, and log/ethean-*-log.
        #[arg(long)]
        data_dir: Option<String>,
        /// Force ephemeral recent-genesis smoke (ignores --data-dir). Restarts start a new chain.
        #[arg(long, default_value_t = false)]
        ephemeral: bool,
        /// Empty --data-dir (chain files, logs, leftovers) before start.
        #[arg(long, default_value_t = false)]
        reset_chain: bool,
        /// More console detail (`-v` Ethean DEBUG, `-vv` include libp2p, `-vvv` TRACE).
        #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
        verbose: u8,
        /// Max log level (error|warn|info|debug|trace|off). Overrides `-v` base level.
        /// `RUST_LOG` still wins when set.
        #[arg(long = "log-level", value_name = "LEVEL")]
        log_level: Option<String>,
        /// Network label (default: pq-devnet-4). Use `pq-devnet-5` when operator mesh is up; `local` for smoke-only.
        #[arg(long, default_value = "pq-devnet-4")]
        network: String,
        /// Comma-separated QUIC multiaddrs (overrides file/env when set).
        #[arg(long)]
        bootnodes: Option<String>,
        /// Optional 8-hex fork digest for gossip topics (overrides interim name hash).
        #[arg(long)]
        fork_digest: Option<String>,
        /// Local registry size (recent genesis; default 4 for finality smoke).
        #[arg(long, default_value_t = 4)]
        validators: usize,
        /// Act as Lean aggregator (collect/inject aggregates). Default on.
        #[arg(long, default_value_t = false)]
        no_aggregator: bool,
        /// Self-apply proposals + full-registry votes so solo head/finality advances.
        /// Default on (disable with --no-local-finality).
        #[arg(long, default_value_t = false)]
        no_local_finality: bool,
        /// Start Prometheus (:9090) + Grafana (:3000) via Docker Compose.
        /// Scrape HTTP on :9100 stays on unless `--no-metrics` is set.
        #[arg(long, default_value_t = false)]
        metrics: bool,
        /// Disable Prometheus scrape HTTP (`/metrics` is on by default at :9100).
        #[arg(long, default_value_t = false)]
        no_metrics: bool,
        /// Metrics listen address (default 127.0.0.1).
        #[arg(long, default_value = "127.0.0.1")]
        metrics_address: String,
        /// Metrics listen port (default 9100; matches deploy/observability scrape).
        #[arg(long, default_value_t = 9100)]
        metrics_port: u16,
    },
    /// Start the validator client
    Validator,
    /// Show version information
    Version,
}
