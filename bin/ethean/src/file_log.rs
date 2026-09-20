//! Process logs: colored stdout + optional plain file under `--data-dir/log`.

use crate::console_fmt::EtheanConsole;
use ethean_node::persist_paths::{utc_run_stamp, PersistPaths};
use std::fs::OpenOptions;
use std::io;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Install tracing: green-forward console; also a dated plain file when durable.
pub fn install(data_dir: Option<&str>, ephemeral: bool) -> Result<(), String> {
    match data_dir {
        Some(dir) if !ephemeral => install_stdout_and_file(dir),
        _ => {
            install_stdout_only();
            Ok(())
        }
    }
}

/// Console-only (no data-dir), same Ethean palette.
pub fn install_stdout_only() {
    let stdout = fmt::layer()
        .event_format(EtheanConsole)
        .with_ansi(true)
        .with_writer(io::stdout);
    tracing_subscriber::registry().with(stdout).init();
}

fn install_stdout_and_file(data_dir: &str) -> Result<(), String> {
    let paths = PersistPaths::new(data_dir);
    paths
        .ensure_dir()
        .map_err(|e| format!("create data-dir log folder: {e}"))?;
    let stamp = utc_run_stamp(now_unix_secs());
    let path = paths.run_log_path(&stamp);
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("open {}: {e}", path.display()))?;

    let stdout = fmt::layer()
        .event_format(EtheanConsole)
        .with_ansi(true)
        .with_writer(io::stdout);
    let file_layer = fmt::layer()
        .event_format(EtheanConsole)
        .with_ansi(false)
        .with_writer(Mutex::new(file));

    tracing_subscriber::registry()
        .with(stdout)
        .with(file_layer)
        .init();
    info!(path = %path.display(), "writing process log under data-dir/log");
    Ok(())
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
