//! Build a client + `StartConfig` from `ethean start` flags.

use ethean_node::{
    block_prune, flag_env_or, load_lean_network_config, parse_subnet_ids, resolve_genesis_source,
    resolve_node_key, resolve_roles, EtheanClient, GenesisBuilder, LocalRoles, MetricsListen,
    NetworkTarget, NodeKeySource, RoleFlags, RpcListen, StartArgs, StartConfig,
};
use std::net::IpAddr;
use tracing::{info, warn};

use crate::lean_assets;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Resolved listen / scrape / HTTP / identity bits for one start.
pub struct StartPlan {
    pub cfg: StartConfig,
    pub roles: LocalRoles,
    pub network: NetworkTarget,
    pub scrape: bool,
    pub metrics_stack: bool,
    pub metrics_bind: String,
    pub http_address: String,
    pub http_port: u16,
    pub node_key_source: NodeKeySource,
}

/// Load genesis, roles, identity, and listeners from `StartArgs`.
pub async fn prepare(args: StartArgs) -> Result<(EtheanClient, StartPlan)> {
    let genesis = resolve_genesis_source(&args.network, args.lean_config.as_deref());
    let mesh = genesis.genesis_path.is_some();
    let mut roles = resolve_roles(
        mesh,
        RoleFlags {
            validators: args.validators,
            no_aggregator: args.no_aggregator,
            is_aggregator: args.is_aggregator,
            no_local_finality: args.no_local_finality,
        },
    );
    let network = NetworkTarget::from_cli(
        &genesis.label,
        args.bootnodes.as_deref(),
        args.fork_digest.as_deref(),
    )?;
    let subnet_ids = parse_subnet_ids(args.aggregate_subnet_ids.as_deref())?;
    let (node_key, node_key_source) = resolve_node_key(
        args.node_key.as_deref(),
        args.data_dir.as_deref(),
        args.ephemeral,
    )?;
    log_node_key(&node_key_source);

    let http_address = flag_env_or(
        args.http_address.clone(),
        "ETHEAN_HTTP_ADDRESS",
        "127.0.0.1",
    );
    let metrics_address = flag_env_or(
        args.metrics_address.clone(),
        "ETHEAN_METRICS_ADDRESS",
        "127.0.0.1",
    );
    let scrape = !args.no_metrics;
    let metrics_stack = args.observability_stack;
    if args.metrics && !metrics_stack {
        info!("--metrics keeps scrape HTTP on (already the default); pass --observability-stack for Grafana");
    }
    if metrics_stack {
        if let Err(e) = crate::observability::ensure_stack() {
            warn!(error = %e, "observability stack not started; node continues");
        }
    } else {
        info!(
            "Prometheus/Grafana UI not started (pass --observability-stack to docker-compose \
             deploy/observability, or run scripts/run-observability.*)"
        );
    }

    let metrics_listen = if scrape {
        Some(MetricsListen {
            addr: format!("{metrics_address}:{}", args.metrics_port).parse()?,
        })
    } else {
        None
    };
    let http_listen = if args.no_http {
        None
    } else {
        Some(RpcListen {
            addr: format!("{http_address}:{}", args.http_port).parse()?,
            admin_token: args.http_admin_token.clone(),
        })
    };
    let listen_ip: IpAddr = args
        .socket_address
        .parse()
        .map_err(|e| format!("--socket-address '{}': {e}", args.socket_address))?;
    let registry_keys =
        lean_assets::load_optional_registry(args.validator_registry.as_deref(), &args.node_id)?;
    let prune_keep = block_prune::resolve_prune_keep_slots(args.prune_keep_slots);

    let mut client = open_client(&args, &genesis, &mut roles).await?;
    lean_assets::apply_registry_keys(&mut client, registry_keys.as_ref());
    log_start(
        &args,
        &network,
        roles,
        scrape,
        metrics_stack,
        prune_keep,
        &http_address,
        &metrics_address,
    );

    let cfg = run_mode(&args)
        .with_network(network.clone())
        .with_metrics(metrics_listen)
        .with_http(http_listen)
        .with_listen_port(args.listen_port)
        .with_listen_ip(listen_ip)
        .with_node_key(Some(node_key))
        .with_roles(roles)
        .with_aggregate_subnet_ids(subnet_ids)
        .with_checkpoint_sync_url(args.checkpoint_sync_url.clone())
        .with_prune_keep_slots(prune_keep);

    Ok((
        client,
        StartPlan {
            cfg,
            roles,
            network,
            scrape,
            metrics_stack,
            metrics_bind: format!("{metrics_address}:{}", args.metrics_port),
            http_address,
            http_port: args.http_port,
            node_key_source,
        },
    ))
}

fn run_mode(args: &StartArgs) -> StartConfig {
    if args.until_signal {
        StartConfig::until_signal(true)
    } else if args.wall_clock {
        StartConfig::wall(args.ticks, true)
    } else {
        StartConfig::smoke(args.ticks)
    }
}

fn log_node_key(src: &NodeKeySource) {
    match src {
        NodeKeySource::File(path) => {
            info!(path = %path.display(), "loaded secp256k1 node key from --node-key");
        }
        NodeKeySource::DataDir { path, created } => {
            if *created {
                info!(path = %path.display(), "generated secp256k1 node key under data-dir");
            } else {
                info!(path = %path.display(), "reused secp256k1 node key from data-dir");
            }
        }
        NodeKeySource::Ephemeral => {
            info!("generated ephemeral secp256k1 node key for this process");
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn log_start(
    args: &StartArgs,
    network: &NetworkTarget,
    roles: LocalRoles,
    scrape: bool,
    metrics_stack: bool,
    prune_keep: u64,
    http_address: &str,
    metrics_address: &str,
) {
    info!(
        ticks = args.ticks,
        wall_clock = args.wall_clock,
        until_signal = args.until_signal,
        ?args.data_dir,
        ephemeral = args.ephemeral,
        reset_chain = args.reset_chain,
        verbose = args.verbose,
        ?args.log_level,
        network = network.id.as_str(),
        bootnodes = network.bootnodes.len(),
        fork_digest = network.fork_digest.as_deref().unwrap_or(""),
        validators = roles.validators,
        is_aggregator = roles.is_aggregator,
        local_finality = roles.local_finality,
        metrics_stack,
        scrape,
        metrics_address,
        metrics_port = args.metrics_port,
        http = !args.no_http,
        http_address,
        http_port = args.http_port,
        listen_port = args.listen_port,
        socket_address = args.socket_address.as_str(),
        prune_keep_slots = prune_keep,
        lean_config = args.lean_config.as_deref().unwrap_or(""),
        node_id = args.node_id.as_str(),
        "Starting lean consensus node"
    );
    if args.ephemeral && args.data_dir.is_some() {
        warn!("--ephemeral set; ignoring --data-dir (recent genesis smoke)");
    }
    if args.reset_chain {
        if args.data_dir.is_none() {
            warn!("--reset-chain ignored without --data-dir");
        } else if args.ephemeral {
            warn!("--reset-chain ignored with --ephemeral");
        } else {
            info!("data-dir emptied before this start (--reset-chain)");
        }
    }
}

async fn open_client(
    args: &StartArgs,
    genesis: &ethean_node::GenesisSource,
    roles: &mut LocalRoles,
) -> Result<EtheanClient> {
    if let Some(ref cfg_path) = genesis.genesis_path {
        let lean = load_lean_network_config(cfg_path)?;
        roles.validators = lean.validators.len().max(1);
        let acc = args
            .attestation_committee_count
            .unwrap_or(lean.attestation_committee_count)
            .max(1);
        info!(
            path = %cfg_path.display(),
            genesis_time = lean.genesis_time,
            validators = lean.validators.len(),
            attestation_committee_count = acc,
            "loaded Lean network config.yaml genesis"
        );
        let built = GenesisBuilder::new(lean.genesis_time)
            .with_validator_keys(lean.validators)
            .build()?;
        let profile = ethean_node::lstar_devnet()?.with_attestation_committee_count(acc)?;
        info!(
            attestation_committee_count = profile.attestation_committee_count,
            attestation_subnets = profile.attestation_subnet_count(),
            "applied ATTESTATION_COMMITTEE_COUNT onto chain profile"
        );
        let mut client = EtheanClient::with_genesis(profile, built.state).await?;
        client.apply_local_roles(*roles);
        return Ok(client);
    }
    if let Some(acc) = args.attestation_committee_count {
        warn!(
            acc,
            "--attestation-committee-count without a genesis file is ignored (needs config.yaml)"
        );
    }
    if args.ephemeral || args.data_dir.is_none() {
        Ok(EtheanClient::with_local_roles(*roles).await?)
    } else {
        Ok(
            EtheanClient::open_data_dir_with_roles(args.data_dir.as_deref().unwrap(), *roles)
                .await?,
        )
    }
}
