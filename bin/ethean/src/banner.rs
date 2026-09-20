//! Geth-style startup welcome: logo, slogan, versions, start snapshot.

use crate::banner_art::{LOGO, PRODUCT, SLOGAN};
use ethean_node::{
    EtheanClient, LocalRoles, NetworkTarget, RunMode, StartConfig, SystemTimeSource,
};
use nu_ansi_term::{Color, Style};
use std::io::{self, Write};

/// Snapshot printed once before duty-loop log dumps.
#[derive(Debug, Clone)]
pub struct StartCard<'a> {
    pub network: &'a NetworkTarget,
    pub roles: LocalRoles,
    pub cfg: &'a StartConfig,
    pub data_dir: Option<&'a str>,
    pub ephemeral: bool,
    pub metrics_stack: bool,
    pub scrape: bool,
    pub metrics_bind: Option<&'a str>,
    pub verbose: u8,
}

/// Early identity (logo + slogan + version) before tracing floods the console.
pub fn print_identity() {
    let green = Color::LightGreen.bold();
    let dim = Style::new().dimmed();
    let mut out = io::stdout();
    let _ = writeln!(out, "{}", green.paint(LOGO.trim_start_matches('\n')));
    let _ = writeln!(out, "  {}", green.paint(PRODUCT));
    let _ = writeln!(out, "  {}", dim.paint(SLOGAN));
    let _ = writeln!(
        out,
        "  {} {}",
        dim.paint("version"),
        green.paint(format!("v{}", env!("CARGO_PKG_VERSION")))
    );
    let _ = writeln!(out);
    let _ = out.flush();
}

/// Detailed monitoring-style card after the client has loaded chain state.
pub fn print_start_card(client: &EtheanClient, card: &StartCard<'_>) {
    let green = Color::LightGreen.bold();
    let cyan = Color::LightCyan.normal();
    let dim = Style::new().dimmed();
    let mut out = io::stdout();

    let head = client
        .owner()
        .head_state
        .as_ref()
        .map(|s| s.slot.get())
        .unwrap_or(0);
    let justified = client
        .owner()
        .head_state
        .as_ref()
        .map(|s| s.latest_justified.slot.get())
        .unwrap_or(0);
    let finalized = client
        .owner()
        .head_state
        .as_ref()
        .map(|s| s.latest_finalized.slot.get())
        .unwrap_or(0);
    let current = client
        .clock()
        .slot_now(&SystemTimeSource)
        .map(|s| s.get())
        .unwrap_or(head);
    let genesis_time = client.genesis().genesis_time();
    let validators = client
        .owner()
        .head_state
        .as_ref()
        .map(|s| s.validators.len())
        .unwrap_or(client.genesis().validators.len());
    let sps = client.profile().seconds_per_slot;
    let fork = client.profile().fork_name;

    let network_kind = network_kind_label(card.network.id.as_str());
    let run_mode = run_mode_label(card.cfg.mode);
    let chain_mode = if card.ephemeral || card.data_dir.is_none() {
        "ephemeral (new genesis each start)"
    } else {
        "durable (--data-dir resume)"
    };

    let _ = writeln!(out, "{}", dim.paint("────────────────────────────────────────────────────────"));
    let _ = writeln!(out, "  {}", green.paint("start snapshot"));
    let _ = writeln!(out, "{}", dim.paint("────────────────────────────────────────────────────────"));
    row(&mut out, "client", &format!("{PRODUCT} v{}", env!("CARGO_PKG_VERSION")), &cyan, &dim);
    row(&mut out, "network", &format!("{} ({})", card.network.id.as_str(), network_kind), &cyan, &dim);
    row(&mut out, "run mode", run_mode, &cyan, &dim);
    row(&mut out, "chain", chain_mode, &cyan, &dim);
    if let Some(dir) = card.data_dir {
        if !card.ephemeral {
            row(&mut out, "data-dir", dir, &cyan, &dim);
        }
    }
    row(&mut out, "fork", fork, &cyan, &dim);
    row(&mut out, "slot time", &format!("{sps}s"), &cyan, &dim);
    row(&mut out, "validators", &validators.to_string(), &green, &dim);
    row(
        &mut out,
        "roles",
        &format!(
            "aggregator={} local_finality={}",
            on_off(card.roles.is_aggregator),
            on_off(card.roles.local_finality)
        ),
        &cyan,
        &dim,
    );
    row(
        &mut out,
        "bootnodes",
        &card.network.bootnodes.len().to_string(),
        &cyan,
        &dim,
    );
    if let Some(fd) = card.network.fork_digest.as_deref() {
        row(&mut out, "fork digest", fd, &cyan, &dim);
    }
    row(&mut out, "genesis time", &genesis_time.to_string(), &cyan, &dim);
    row(
        &mut out,
        "slots",
        &format!("head={head}  justified={justified}  finalized={finalized}  wall={current}"),
        &green,
        &dim,
    );
    row(
        &mut out,
        "metrics",
        &metrics_line(card.metrics_stack, card.scrape, card.metrics_bind),
        &cyan,
        &dim,
    );
    row(&mut out, "log verbosity", &verbose_label(card.verbose), &cyan, &dim);
    let _ = writeln!(out, "{}", dim.paint("────────────────────────────────────────────────────────"));
    let _ = writeln!(
        out,
        "  {}",
        dim.paint("duty loop starting — standard log dump follows")
    );
    let _ = writeln!(out);
    let _ = out.flush();
}

fn row(out: &mut impl Write, key: &str, value: &str, value_style: &Style, key_style: &Style) {
    let _ = writeln!(
        out,
        "  {:14} {}",
        key_style.paint(key),
        value_style.paint(value)
    );
}

fn on_off(v: bool) -> &'static str {
    if v {
        "on"
    } else {
        "off"
    }
}

fn network_kind_label(id: &str) -> &'static str {
    match id {
        "pq-devnet-4" | "pq-devnet-5" => "Lean PQ devnet (not mainnet)",
        "local" => "local smoke",
        _ => "custom / operator",
    }
}

fn run_mode_label(mode: RunMode) -> &'static str {
    match mode {
        RunMode::UntilSignal { .. } => "until-signal (Ctrl-C)",
        RunMode::WallClock { .. } => "wall-clock ticks",
        RunMode::SmokeElapsed { .. } => "smoke elapsed ticks",
    }
}

fn metrics_line(stack: bool, scrape: bool, bind: Option<&str>) -> String {
    let scrape_s = if scrape {
        bind.unwrap_or("127.0.0.1:9100")
    } else {
        "off"
    };
    format!(
        "scrape={scrape_s}  grafana/prometheus={}",
        if stack { "starting" } else { "not started" }
    )
}

fn verbose_label(v: u8) -> String {
    match v {
        0 => "info (default)".into(),
        1 => "-v debug (libp2p quiet)".into(),
        2 => "-vv debug + libp2p".into(),
        _ => "-vvv trace".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_kind_marks_devnet() {
        assert!(network_kind_label("pq-devnet-4").contains("devnet"));
        assert!(network_kind_label("local").contains("smoke"));
    }
}
