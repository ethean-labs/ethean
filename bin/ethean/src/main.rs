//! Ethean Lean Consensus Client main binary

use clap::Parser;
use ethean_node::{
    cli::{Cli, Command},
    EtheanClient, StartConfig,
};
use tracing::info;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!(
        "Starting Ethean Lean Consensus Client v{}",
        env!("CARGO_PKG_VERSION")
    );

    let cli = Cli::parse();

    match cli.command {
        Command::Start { ticks, wall_clock } => {
            info!(ticks, wall_clock, "Starting lean consensus node");
            let client = EtheanClient::new().await?;
            let cfg = if wall_clock {
                StartConfig::wall(ticks, true)
            } else {
                StartConfig::smoke(ticks)
            };
            client.start_with(cfg).await?;
        }
        Command::Validator => {
            info!("Starting validator client");
            println!("Validator client not yet implemented");
        }
        Command::Version => {
            println!(
                "Ethean Lean Consensus Client v{}",
                env!("CARGO_PKG_VERSION")
            );
        }
    }

    Ok(())
}
