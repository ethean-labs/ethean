//! Panro Beacon Chain Client Main Binary

use panro::{
    cli::{Cli, Command},
    PanroClient,
    Result,
};
use tracing::{info, error};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Panro Beacon Chain Client v{}", env!("CARGO_PKG_VERSION"));

    // Parse command line arguments
    let cli = Cli::parse();

    match cli.command {
        Command::Start => {
            info!("Starting beacon node");
            let client = PanroClient::new().await?;
            client.start().await?;
        }
        Command::Validator => {
            info!("Starting validator client");
            // Validator specific logic would go here
            println!("Validator client not yet implemented");
        }
        Command::Version => {
            println!("Panro Beacon Chain Client v{}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}
