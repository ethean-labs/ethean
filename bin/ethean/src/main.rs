//! Ethean Lean Consensus Client main binary

use clap::Parser;
use ethean_crypto::FfiStatus;
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
        Command::Start {
            ticks,
            wall_clock,
            until_signal,
            data_dir,
        } => {
            info!(
                ticks,
                wall_clock,
                until_signal,
                ?data_dir,
                "Starting lean consensus node"
            );
            let client = match data_dir.as_deref() {
                Some(path) => EtheanClient::open_data_dir(path).await?,
                None => EtheanClient::new().await?,
            };
            let cfg = if until_signal {
                StartConfig::until_signal(true)
            } else if wall_clock {
                StartConfig::wall(ticks, true)
            } else {
                StartConfig::smoke(ticks)
            };
            client.start_with(cfg).await?;
        }
        Command::Validator => {
            let ffi = FfiStatus::probe();
            info!(?ffi, "Validator client status");
            println!(
                "Validator duties require production crypto backends (leansig={}, leanvm={}).",
                ffi.leansig, ffi.leanvm
            );
            if !ffi.both_selected() {
                println!("Gaps: {:?}", ffi.gaps());
                println!("Fail-closed: enable leansig-backend / leanvm-backend when pins link.");
            }
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
