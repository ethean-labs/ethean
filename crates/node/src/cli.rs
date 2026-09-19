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
        /// Run until Ctrl-C (implies wall-clock sleeps).
        #[arg(long, default_value_t = false)]
        until_signal: bool,
        /// Path-backed store (requires `ethean-storage/rocksdb` feature on the build).
        #[arg(long)]
        data_dir: Option<String>,
        /// Network label (default: pq-devnet-5). Use `local` for explicit smoke-only.
        #[arg(long, default_value = "pq-devnet-5")]
        network: String,
        /// Comma-separated QUIC multiaddrs (overrides file/env when set).
        #[arg(long)]
        bootnodes: Option<String>,
        /// Optional 8-hex fork digest for gossip topics (overrides interim name hash).
        #[arg(long)]
        fork_digest: Option<String>,
    },
    /// Start the validator client
    Validator,
    /// Show version information
    Version,
}
