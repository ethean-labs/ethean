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
        /// Path-backed store (requires `ethean-storage/rocksdb` feature on the build).
        #[arg(long)]
        data_dir: Option<String>,
        /// Network label (default: pq-devnet-4). Use `pq-devnet-5` when operator mesh is up; `local` for smoke-only.
        #[arg(long, default_value = "pq-devnet-4")]
        network: String,
        /// Comma-separated QUIC multiaddrs (overrides file/env when set).
        #[arg(long)]
        bootnodes: Option<String>,
        /// Optional 8-hex fork digest for gossip topics (overrides interim name hash).
        #[arg(long)]
        fork_digest: Option<String>,
        /// Local genesis validator count (default 4). Recent genesis time so slots start near zero.
        #[arg(long, default_value_t = 4)]
        validators: usize,
        /// Disable aggregator role (on by default for local finality smoke).
        #[arg(long, default_value_t = false)]
        no_aggregator: bool,
        /// Disable local finality / self-apply (on by default so solo runs advance head).
        #[arg(long, default_value_t = false)]
        no_local_finality: bool,
        /// Start Prometheus (:9090) + Grafana (:3000) via Docker Compose.
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
