//! Ethean Lean Consensus Client main binary

mod banner;
mod banner_art;
mod console_fmt;
mod file_log;
mod lean_assets;
mod log_filter;
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
    let (verbose, log_level, show_banner) = match &cli.command {
        Command::Start {
            verbose,
            log_level,
            no_banner,
            ..
        } => (*verbose, log_level.as_deref(), !*no_banner),
        Command::Version => (0, None, true),
        _ => (0, None, false),
    };
    let filter = log_filter::build_filter(verbose, log_level)?;

    if show_banner {
        banner::print_identity();
    }

    match &cli.command {
        Command::Start {
            data_dir,
            ephemeral,
            reset_chain,
            ..
        } => {
            if *reset_chain {
                if let Some(path) = data_dir.as_deref() {
                    if !*ephemeral {
                        ethean_node::chain_persist::reset_chain_files(std::path::Path::new(
                            path,
                        ))?;
                    }
                }
            }
            file_log::install(data_dir.as_deref(), *ephemeral, filter)?;
        }
        _ => file_log::install_stdout_only(filter),
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
            verbose,
            log_level,
            no_banner,
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
            no_http,
            http_address,
            http_port,
            http_admin_token,
            listen_port,
            prune_keep_slots,
            lean_config,
            validator_registry,
            node_id,
        } => {
            let network = NetworkTarget::from_cli(
                &network,
                bootnodes.as_deref(),
                fork_digest.as_deref(),
            )?;
            let mut roles = LocalRoles {
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
            let http_listen = if no_http {
                None
            } else {
                let addr = format!("{http_address}:{http_port}").parse()?;
                Some(ethean_node::RpcListen {
                    addr,
                    admin_token: http_admin_token,
                })
            };
            let registry_keys =
                lean_assets::load_optional_registry(validator_registry.as_deref(), &node_id)?;
            let prune_keep = ethean_node::block_prune::resolve_prune_keep_slots(prune_keep_slots);
            info!(
                ticks,
                wall_clock,
                until_signal,
                ?data_dir,
                ephemeral,
                reset_chain,
                verbose,
                ?log_level,
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
                http = http_listen.is_some(),
                http_address = http_address.as_str(),
                http_port,
                listen_port,
                prune_keep_slots = prune_keep,
                lean_config = lean_config.as_deref().unwrap_or(""),
                node_id = node_id.as_str(),
                "Starting lean consensus node"
            );
            if ephemeral && data_dir.is_some() {
                warn!("--ephemeral set; ignoring --data-dir (recent genesis smoke)");
            }
            if reset_chain {
                if data_dir.is_none() {
                    warn!("--reset-chain ignored without --data-dir");
                } else if ephemeral {
                    warn!("--reset-chain ignored with --ephemeral");
                } else {
                    info!("data-dir emptied before this start (--reset-chain)");
                }
            }
            let client = if let Some(ref cfg_path) = lean_config {
                let lean = ethean_node::load_lean_network_config(std::path::Path::new(cfg_path))?;
                roles.validators = lean.validators.len().max(1);
                info!(
                    path = %cfg_path,
                    genesis_time = lean.genesis_time,
                    validators = lean.validators.len(),
                    attestation_committee_count = lean.attestation_committee_count,
                    "loaded Lean network config.yaml genesis"
                );
                let built = ethean_node::GenesisBuilder::new(lean.genesis_time)
                    .with_validator_keys(lean.validators)
                    .build()?;
                let profile = ethean_node::lstar_devnet()?.with_attestation_committee_count(
                    lean.attestation_committee_count.max(1),
                )?;
                info!(
                    attestation_committee_count = profile.attestation_committee_count,
                    attestation_subnets = profile.attestation_subnet_count(),
                    "applied config.yaml ATTESTATION_COMMITTEE_COUNT onto chain profile"
                );
                let mut client = EtheanClient::with_genesis(profile, built.state).await?;
                client.apply_local_roles(roles);
                client
            } else if ephemeral || data_dir.is_none() {
                EtheanClient::with_local_roles(roles).await?
            } else {
                EtheanClient::open_data_dir_with_roles(data_dir.as_deref().unwrap(), roles).await?
            };
            let mut client = client;
            lean_assets::apply_registry_keys(&mut client, registry_keys.as_ref());
            let cfg = if until_signal {
                StartConfig::until_signal(true)
            } else if wall_clock {
                StartConfig::wall(ticks, true)
            } else {
                StartConfig::smoke(ticks)
            }
            .with_network(network.clone())
            .with_metrics(metrics_listen.clone())
            .with_http(http_listen)
            .with_listen_port(listen_port)
            .with_roles(roles)
            .with_prune_keep_slots(prune_keep);
            if !no_banner {
                let bind = format!("{metrics_address}:{metrics_port}");
                banner::print_start_card(
                    &client,
                    &banner::StartCard {
                        network: &network,
                        roles,
                        cfg: &cfg,
                        data_dir: data_dir.as_deref(),
                        ephemeral,
                        metrics_stack: metrics,
                        scrape,
                        metrics_bind: scrape.then_some(bind.as_str()),
                        verbose,
                    },
                );
            }
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
