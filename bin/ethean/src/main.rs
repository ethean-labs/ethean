//! Ethean Lean Consensus Client main binary

mod file_log;
mod observability;

use clap::Parser;
use ethean_crypto::FfiStatus;
use ethean_node::{
    cli::{Cli, Command},
    EtheanClient, LocalRoles, MetricsListen, NetworkTarget, StartConfig,
};
use tracing::{info, warn};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Command::Start {
            data_dir,
            ephemeral,
            ..
        } => file_log::install(data_dir.as_deref(), *ephemeral)?,
        _ => tracing_subscriber::fmt().init(),
    }

    info!(
        "Starting Ethean Lean Consensus Client v{}",
        env!("CARGO_PKG_VERSION")
    );

    match cli.command {
        Command::Start {
            ticks,
            wall_clock,
            until_signal,
            data_dir,
            ephemeral,
            reset_chain,
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
            let roles = LocalRoles {
                validators: validators.max(1),
                is_aggregator: !no_aggregator,
                local_finality: !no_local_finality,
            };
            let scrape = !no_metrics;
            if metrics {
                if let Err(e) = observability::ensure_stack() {
                    warn!(error = %e, "observability stack not started; node continues");
                }
            } else {
                info!(
                    "Prometheus/Grafana UI not started (pass --metrics to docker-compose \
                     deploy/observability, or run scripts/run-observability.*)"
                );
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
                ephemeral,
                reset_chain,
                network = network.id.as_str(),
                bootnodes = network.bootnodes.len(),
                fork_digest = network.fork_digest.as_deref().unwrap_or(""),
                validators = roles.validators,
                is_aggregator = roles.is_aggregator,
                local_finality = roles.local_finality,
                metrics_stack = metrics,
                scrape,
                metrics_address = metrics_address.as_str(),
                metrics_port,
                "Starting lean consensus node"
            );
            if ephemeral && data_dir.is_some() {
                warn!("--ephemeral set; ignoring --data-dir (recent genesis smoke)");
            }
            if reset_chain {
                if let Some(ref path) = data_dir {
                    if !ephemeral {
                        ethean_node::chain_persist::reset_chain_files(std::path::Path::new(path))?;
                    }
                } else {
                    warn!("--reset-chain ignored without --data-dir");
                }
            }
            let client = if ephemeral || data_dir.is_none() {
                EtheanClient::with_local_roles(roles).await?
            } else {
                EtheanClient::open_data_dir_with_roles(data_dir.as_deref().unwrap(), roles).await?
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
            .with_roles(roles);
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
