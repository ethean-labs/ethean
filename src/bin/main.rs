//! Ethean Lean Consensus Client Main Binary

use ethean::{
    cli::{Cli, Command},
    EtheanClient,
};
use tracing::{info, error};
use clap::Parser;

/// Main result type for the application
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Ethean Lean Consensus Client v{}", env!("CARGO_PKG_VERSION"));

    // Parse command line arguments
    let cli = Cli::parse();

    match cli.command {
        Command::Start => {
            info!("Starting lean consensus node");
            let client = EtheanClient::new().await?;
            client.start().await?;
        }
        Command::Validator => {
            info!("Starting validator client");
            // Validator specific logic would go here
            println!("Validator client not yet implemented");
        }
        Command::Version => {
            println!("Ethean Lean Consensus Client v{}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}
