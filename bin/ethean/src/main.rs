//! Ethean Lean Consensus Client main binary

mod banner;
mod banner_art;
mod console_fmt;
mod file_log;
mod lean_assets;
mod log_filter;
mod observability;
mod start_run;

use clap::Parser;
use ethean_node::crypto_status::CryptoStatus;
use ethean_node::{
    cli::{Cli, Command},
    StartArgs,
};
use tracing::info;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let (verbose, log_level, show_banner) = match &cli.command {
        Command::Start(a) => (a.verbose, a.log_level.as_deref(), !a.no_banner),
        Command::Version => (0, None, true),
        _ => (0, None, false),
    };
    let filter = log_filter::build_filter(verbose, log_level)?;

    if show_banner {
        banner::print_identity();
    }

    match &cli.command {
        Command::Start(a) => {
            if a.reset_chain {
                if let (Some(path), false) = (a.data_dir.as_deref(), a.ephemeral) {
                    ethean_node::chain_persist::reset_chain_files(std::path::Path::new(path))?;
                }
            }
            file_log::install(a.data_dir.as_deref(), a.ephemeral, filter)?;
        }
        _ => file_log::install_stdout_only(filter),
    }

    info!(
        "Starting Ethean Lean Consensus Client v{}",
        env!("CARGO_PKG_VERSION")
    );

    match cli.command {
        Command::Start(args) => run_start(args).await?,
        Command::Validator => run_validator(),
        Command::Version => {
            println!(
                "Ethean Lean Consensus Client v{}",
                env!("CARGO_PKG_VERSION")
            );
        }
    }

    Ok(())
}

async fn run_start(args: StartArgs) -> Result<()> {
    let no_banner = args.no_banner;
    let data_dir = args.data_dir.clone();
    let ephemeral = args.ephemeral;
    let verbose = args.verbose;
    let (client, plan) = start_run::prepare(args).await?;
    if !no_banner {
        banner::print_start_card(
            &client,
            &banner::StartCard {
                network: &plan.network,
                roles: plan.roles,
                cfg: &plan.cfg,
                data_dir: data_dir.as_deref(),
                ephemeral,
                metrics_stack: plan.metrics_stack,
                scrape: plan.scrape,
                metrics_bind: plan.scrape.then_some(plan.metrics_bind.as_str()),
                verbose,
            },
        );
    }
    let _ = (&plan.http_address, plan.http_port, &plan.node_key_source);
    client.start_with(plan.cfg).await?;
    Ok(())
}

fn run_validator() {
    let status = CryptoStatus::probe();
    info!(?status, "Validator client status");
    println!("XMSS signing and verification: native (leanSpec PROD).");
    println!(
        "Aggregate proof verification: leanMultisig {}.",
        status.verifier_rev
    );
    match &status.prover {
        Some(path) => println!("Prover: {}", path.display()),
        None => println!(
            "Prover: not found. Build `ethean-prover` or set ETHEAN_PROVER_BIN; \
             aggregators and proposers need it."
        ),
    }
}
