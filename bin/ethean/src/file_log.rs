//! File logs under `--data-dir/log` (stdout stays on).

use ethean_node::persist_paths::{utc_run_stamp, PersistPaths};
use std::fs::OpenOptions;
use std::io;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;
use tracing_subscriber::fmt::writer::MakeWriterExt;

/// Install tracing: stdout always; also a dated file when durable data-dir is on.
pub fn install(data_dir: Option<&str>, ephemeral: bool) -> Result<(), String> {
    match data_dir {
        Some(dir) if !ephemeral => install_stdout_and_file(dir),
        _ => {
            tracing_subscriber::fmt().init();
            Ok(())
        }
    }
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
    let writer = io::stdout.and(Mutex::new(file));
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(writer)
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
