//! `ethean devnet-init`: write a local multi-node devnet bundle and print the
//! start command of every node.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ethean_node::cli::DevnetInitArgs;
use ethean_node::devnet_bundle::{production_keygen, write_bundle, DevnetSpec, KeyRole};
use tracing::info;

use crate::Result;

/// HTTP API port of node `k`.
pub fn http_port(k: usize) -> u16 {
    5052 + k as u16
}

/// Prometheus port of node `k`.
pub fn metrics_port(k: usize) -> u16 {
    9200 + k as u16
}

pub fn run(args: &DevnetInitArgs) -> Result<()> {
    let dir = PathBuf::from(&args.out);
    let spec = |genesis_time| DevnetSpec {
        nodes: args.nodes,
        validators_per_node: args.validators_per_node,
        genesis_time,
        base_port: args.base_port,
        attestation_committee_count: args.attestation_committee_count,
        aggregators: args.aggregators,
    };
    let mut keygen = |index: u64, role: KeyRole| {
        info!(index, ?role, "generating PROD XMSS key (about 20-40 s)");
        production_keygen(index, role)
    };
    // Keys first: generation can take minutes, and genesis must still be ahead.
    write_bundle(&dir, spec(0), &mut keygen)?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let bundle = write_bundle(&dir, spec(now + args.genesis_delay), &mut |_, _| {
        Err("key files vanished between passes".into())
    })?;

    println!("devnet bundle: {}", dir.display());
    println!("  config     : {}", bundle.config.display());
    println!("  registry   : {}", bundle.registry.display());
    println!("  nodes      : {}", bundle.nodes_file.display());
    println!("  genesis in : {} s", args.genesis_delay);
    println!();
    for (k, node) in bundle.nodes.iter().enumerate() {
        let others: Vec<&str> = bundle
            .nodes
            .iter()
            .filter(|n| n.node_id != node.node_id)
            .filter_map(|n| n.multiaddr.as_deref())
            .collect();
        let bootnodes = if others.is_empty() {
            "none".to_string()
        } else {
            others.join(",")
        };
        let aggregator = if node.is_aggregator {
            " --is-aggregator"
        } else {
            ""
        };
        println!(
            "ethean start --until-signal --network {} --validator-registry {} --node-id {} \
             --node-key {} --listen-port {} --http-port {} --metrics-port {} \
             --data-dir {} --reset-chain --bootnodes {bootnodes}{aggregator}",
            bundle.config.display(),
            bundle.registry.display(),
            node.node_id,
            node.node_key.display(),
            node.quic_port,
            http_port(k),
            metrics_port(k),
            dir.join("data").join(&node.node_id).display(),
        );
    }
    Ok(())
}
