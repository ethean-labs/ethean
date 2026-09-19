//! Ethean Lean Consensus Client main binary

mod observability;

use clap::Parser;
use ethean_crypto::FfiStatus;
use ethean_node::{
    cli::{Cli, Command},
    EtheanClient, MetricsListen, NetworkTarget, StartConfig,
};
use tracing::{info, warn};

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
            network,
            bootnodes,
            fork_digest,
            validators,
            no_aggregator,
            no_local_finality,
            metrics,
            no_metrics,
            metrics_address,
            metrics_port,
        } => {
            let network = NetworkTarget::from_cli(
                &network,
                bootnodes.as_deref(),
                fork_digest.as_deref(),
            )?;
            let scrape = !no_metrics;
            let is_aggregator = !no_aggregator;
            let finality = !no_local_finality;
            if metrics {
                if let Err(e) = observability::ensure_stack() {
                    warn!(error = %e, "observability stack not started; node continues");
                }
            }
            let metrics_listen = if scrape {
                let addr = format!("{metrics_address}:{metrics_port}").parse()?;
                Some(MetricsListen { addr })
            } else {
                None
            };
            info!(
                ticks,
                wall_clock,
                until_signal,
                ?data_dir,
                network = network.id.as_str(),
                bootnodes = network.bootnodes.len(),
                validators,
                is_aggregator,
                local_finality = finality,
                metrics_stack = metrics,
                scrape,
                "Starting lean consensus node"
            );
            if network.bootnodes.is_empty() && !finality {
                warn!(
                    "no bootnodes and local finality disabled — head will stay at genesis \
                     (pass --local-finality or --bootnodes / use scripts/local-pq-mesh.*)"
                );
            }
            let client = match data_dir.as_deref() {
                Some(path) => EtheanClient::open_data_dir_validators(path, validators).await?,
                None => EtheanClient::with_local_devnet(validators).await?,
            };
            let cfg = if until_signal {
                StartConfig::until_signal(true)
            } else if wall_clock {
                StartConfig::wall(ticks, true)
            } else {
                StartConfig::smoke(ticks)
            }
            .with_network(network)
            .with_metrics(metrics_listen)
            .with_validators(validators)
            .with_roles(is_aggregator, finality);
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
