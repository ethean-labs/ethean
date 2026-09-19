//! Bring up local Prometheus + Grafana via Docker Compose when `--metrics` is set.

use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

/// Start `deploy/observability` (Prometheus :9090, Grafana :3000).
///
/// Best-effort: missing Docker prints a warning and returns Ok so the node still runs.
pub fn ensure_stack() -> Result<(), String> {
    let Some(dir) = find_observability_dir() else {
        warn!(
            "deploy/observability not found; start Grafana/Prometheus manually \
             (./scripts/run-observability.sh or .\\scripts\\run-observability.ps1)"
        );
        return Ok(());
    };

    info!(
        path = %dir.display(),
        "starting Prometheus + Grafana via docker compose (--metrics)"
    );

    let status = match Command::new("docker")
        .args(["compose", "up", "-d"])
        .current_dir(&dir)
        .status()
    {
        Ok(s) => s,
        Err(e) => {
            warn!(
                error = %e,
                "Docker CLI not found — Grafana :3000 and Prometheus :9090 will stay blank. \
                 Install and start Docker Desktop, then re-run with --metrics \
                 (or .\\scripts\\run-observability.ps1). Port change is not needed; \
                 only :9100 is from ethean itself."
            );
            return Ok(());
        }
    };

    if !status.success() {
        return Err(format!(
            "`docker compose up -d` failed in {} (exit {:?})",
            dir.display(),
            status.code()
        ));
    }

    info!("Grafana http://localhost:3000  Prometheus http://localhost:9090");
    info!("Scrape expects ethean metrics on 127.0.0.1:9100");
    Ok(())
}

fn find_observability_dir() -> Option<PathBuf> {
    if let Ok(root) = std::env::var("ETHEAN_ROOT") {
        let p = PathBuf::from(root).join("deploy/observability");
        if compose_exists(&p) {
            return Some(p);
        }
    }

    let mut candidates = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.to_path_buf());
        }
    }

    for start in candidates {
        let mut cur = start;
        for _ in 0..8 {
            let p = cur.join("deploy/observability");
            if compose_exists(&p) {
                return Some(p);
            }
            if !cur.pop() {
                break;
            }
        }
    }
    None
}

fn compose_exists(dir: &Path) -> bool {
    dir.join("docker-compose.yml").is_file() || dir.join("compose.yml").is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_exists_false_for_missing() {
        assert!(!compose_exists(Path::new("/no/such/observability")));
    }
}
