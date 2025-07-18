//! Command-line interface for Panro

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "panro")]
#[command(about = "Panro Beacon Chain Client")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start the beacon node
    Start,
    /// Start the validator client
    Validator,
    /// Show version information
    Version,
}
