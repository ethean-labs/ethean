//! Command-line interface for Ethean

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "ethean")]
#[command(about = "Ethean Lean Consensus Client")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start the lean consensus node
    Start(StartArgs),
    /// Start the validator client
    Validator,
    /// Show version information
    Version,
}

/// Flags for `ethean start` (Hive / lean-quickstart command lines mirror ream's).
#[derive(Args, Debug, Clone)]
pub struct StartArgs {
    /// Number of duty ticks before drain (ignored with --until-signal).
    #[arg(long, default_value_t = 5)]
    pub ticks: u32,
    /// Sample the system wall clock (sleeps between intervals).
    #[arg(long, default_value_t = false)]
    pub wall_clock: bool,
    /// Run until Ctrl-C (implies wall-clock sleeps). Long-run / Grafana path.
    #[arg(long, default_value_t = false)]
    pub until_signal: bool,
    /// Durable directory: genesis SSZ, ethean.redb, node.key, and log/ethean-*-log.
    #[arg(long)]
    pub data_dir: Option<String>,
    /// Force ephemeral recent-genesis smoke (ignores --data-dir). Restarts start a new chain.
    #[arg(long, default_value_t = false)]
    pub ephemeral: bool,
    /// Empty --data-dir (chain files, logs, leftovers) before start.
    #[arg(long, default_value_t = false)]
    pub reset_chain: bool,
    /// More console detail (`-v` Ethean DEBUG, `-vv` include libp2p, `-vvv` TRACE).
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,
    /// Max log level (error|warn|info|debug|trace|off). Overrides `-v` base level.
    /// `RUST_LOG` still wins when set.
    #[arg(long = "log-level", value_name = "LEVEL")]
    pub log_level: Option<String>,
    /// Skip the ASCII startup banner and start snapshot card.
    #[arg(long, default_value_t = false)]
    pub no_banner: bool,
    /// Network label (pq-devnet-4 default, pq-devnet-5, local) or a path to a
    /// Lean `config.yaml`; a path loads that genesis under the `local` profile.
    #[arg(long, default_value = "pq-devnet-4", value_name = "LABEL|PATH")]
    pub network: String,
    /// Bootnodes: `none`, CSV of QUIC multiaddrs and/or `enr:` records, or a
    /// YAML file (`nodes.yaml`). Overrides `ETHEAN_BOOTNODES` and the label file.
    #[arg(long, value_name = "NONE|CSV|PATH")]
    pub bootnodes: Option<String>,
    /// Optional 8-hex fork digest for gossip topics (overrides interim name hash).
    #[arg(long)]
    pub fork_digest: Option<String>,
    /// Local registry size (recent genesis; default 4 for finality smoke).
    #[arg(long, default_value_t = 4)]
    pub validators: usize,
    /// Disable the Lean aggregator role (solo default on; mesh default off).
    #[arg(long, default_value_t = false)]
    pub no_aggregator: bool,
    /// Enable the Lean aggregator role (needed with a genesis file; ream-compatible).
    #[arg(long, default_value_t = false)]
    pub is_aggregator: bool,
    /// Subnet ids an aggregator covers (CSV of u64). Ethean subscribes to every
    /// attestation subnet already; the list is recorded for parity with ream.
    #[arg(long = "aggregate-subnet-ids", value_name = "CSV")]
    pub aggregate_subnet_ids: Option<String>,
    /// Override ATTESTATION_COMMITTEE_COUNT from config.yaml.
    #[arg(long = "attestation-committee-count", value_name = "N")]
    pub attestation_committee_count: Option<u64>,
    /// Checkpoint sync: `<url>/lean/v0/states/finalized` (blocks URL is derived).
    /// Anchors an empty data dir on the fetched finalized state + block.
    #[arg(long = "checkpoint-sync-url", value_name = "URL")]
    pub checkpoint_sync_url: Option<String>,
    /// Self-apply proposals + full-registry votes so solo head/finality advances.
    /// Solo default on; off whenever a genesis file is given.
    #[arg(long, default_value_t = false)]
    pub no_local_finality: bool,
    /// Keep the Prometheus scrape HTTP on (already the default; ream-compatible).
    #[arg(long, default_value_t = false, conflicts_with = "no_metrics")]
    pub metrics: bool,
    /// Disable Prometheus scrape HTTP (`/metrics` is on by default at :9100).
    #[arg(long, default_value_t = false)]
    pub no_metrics: bool,
    /// Start Prometheus (:9090) + Grafana (:3000) via Docker Compose.
    #[arg(long = "observability-stack", default_value_t = false)]
    pub observability_stack: bool,
    /// Metrics listen address (default 127.0.0.1; env `ETHEAN_METRICS_ADDRESS`).
    #[arg(long, value_name = "IP")]
    pub metrics_address: Option<String>,
    /// Metrics listen port (default 9100; matches deploy/observability scrape).
    #[arg(long, default_value_t = 9100)]
    pub metrics_port: u16,
    /// Disable Lean HTTP API (`/lean/v1` is on by default at :5052).
    #[arg(long, default_value_t = false)]
    pub no_http: bool,
    /// Lean HTTP listen address (default 127.0.0.1; env `ETHEAN_HTTP_ADDRESS`).
    #[arg(long, value_name = "IP")]
    pub http_address: Option<String>,
    /// Lean HTTP listen port (default 5052).
    #[arg(long, default_value_t = 5052)]
    pub http_port: u16,
    /// Bearer token for `/lean/v1/admin/*` on non-loopback HTTP binds.
    #[arg(long, default_value = "")]
    pub http_admin_token: String,
    /// QUIC listen interface (default 0.0.0.0).
    #[arg(long = "socket-address", default_value = "0.0.0.0", value_name = "IP")]
    pub socket_address: String,
    /// UDP/QUIC listen port (default 9000; `0` = OS ephemeral). Alias `--socket-port`.
    #[arg(long, alias = "socket-port", default_value_t = 9000)]
    pub listen_port: u16,
    /// secp256k1 node key file (64 hex chars) for the libp2p PeerId.
    /// Alias `--private-key-path`. Default: `<data-dir>/node.key`, else per run.
    #[arg(long = "node-key", alias = "private-key-path", value_name = "PATH")]
    pub node_key: Option<String>,
    /// Slots kept below finalized before pruning `--data-dir` block blobs (default 256).
    /// Override with `ETHEAN_PRUNE_KEEP_SLOTS` when the flag is omitted.
    #[arg(long = "prune-keep-slots", value_name = "SLOTS")]
    pub prune_keep_slots: Option<u64>,
    /// Path to Lean Hive / quickstart `config.yaml` (GENESIS_TIME + GENESIS_VALIDATORS).
    #[arg(long = "lean-config", value_name = "PATH")]
    pub lean_config: Option<String>,
    /// Path to `validators.yaml` / `annotated_validators.yaml` (node id → validator rows).
    /// Alias `--validator-registry-path`.
    #[arg(
        long = "validator-registry",
        alias = "validator-registry-path",
        value_name = "PATH"
    )]
    pub validator_registry: Option<String>,
    /// Node id used with `--validator-registry` (e.g. `ethean_0`).
    #[arg(long = "node-id", default_value = "ethean_0")]
    pub node_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> StartArgs {
        let mut full = vec!["ethean", "start"];
        full.extend_from_slice(args);
        match Cli::parse_from(full).command {
            Command::Start(a) => a,
            _ => panic!("expected start"),
        }
    }

    #[test]
    fn ream_style_aliases_parse() {
        let a = parse(&[
            "--network",
            "/tmp/config.yaml",
            "--validator-registry-path",
            "/tmp/validators.yaml",
            "--node-id",
            "ethean_0",
            "--socket-address",
            "0.0.0.0",
            "--socket-port",
            "9001",
            "--bootnodes",
            "none",
            "--private-key-path",
            "/tmp/node.key",
            "--is-aggregator",
            "--aggregate-subnet-ids",
            "0,1",
            "--attestation-committee-count",
            "2",
            "--checkpoint-sync-url",
            "http://h/lean/v0/states/finalized",
            "--metrics",
        ]);
        assert_eq!(a.listen_port, 9001);
        assert_eq!(
            a.validator_registry.as_deref(),
            Some("/tmp/validators.yaml")
        );
        assert_eq!(a.node_key.as_deref(), Some("/tmp/node.key"));
        assert!(a.is_aggregator && a.metrics && !a.observability_stack);
        assert_eq!(a.attestation_committee_count, Some(2));
    }

    #[test]
    fn legacy_flags_still_parse() {
        let a = parse(&[
            "--until-signal",
            "--ephemeral",
            "--listen-port",
            "9000",
            "--validator-registry",
            "/tmp/v.yaml",
            "--no-aggregator",
            "--no-local-finality",
            "--no-metrics",
            "--observability-stack",
        ]);
        assert!(a.until_signal && a.no_aggregator && a.no_metrics && a.observability_stack);
        assert_eq!(a.socket_address, "0.0.0.0");
    }
}
