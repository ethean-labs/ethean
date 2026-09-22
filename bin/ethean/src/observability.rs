//! Bring up local Prometheus + Grafana via Docker Compose when `--metrics` is set.

use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

/// Start `deploy/observability` (Prometheus :9090, Grafana :3000).
///
/// Best-effort: missing or unhealthy Docker prints a warning and returns Ok so the node still runs.
pub fn ensure_stack() -> Result<(), String> {
    let Some(dir) = find_observability_dir() else {
        warn!(
            "deploy/observability not found; start Grafana/Prometheus manually \
             (./scripts/run-observability.sh)"
        );
        return Ok(());
    };

    info!(
        path = %dir.display(),
        "starting Prometheus + Grafana via docker compose (--metrics)"
    );

    let output = match Command::new("docker")
        .args(["compose", "up", "-d"])
        .current_dir(&dir)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            warn!(
                error = %e,
                "Docker CLI not found — Grafana :3000 and Prometheus :9090 will stay blank. \
                 Install Docker Engine with the compose plugin, then re-run with --metrics \
                 (or ./scripts/run-observability.sh). Port change is not needed; \
                 only :9100 is from ethean itself."
            );
            return Ok(());
        }
    };

    if output.status.success() {
        info!("Grafana http://localhost:3000  Prometheus http://localhost:9090");
        info!("Scrape expects ethean metrics on 127.0.0.1:9100");
        return Ok(());
    }

    let combined = combine_output(&output.stdout, &output.stderr);
    warn!(
        exit = ?output.status.code(),
        "docker compose failed — {}. Ethean keeps running; scrape \
         http://127.0.0.1:9100/metrics without Grafana.",
        compose_failure_hint(&combined)
    );
    Ok(())
}

fn combine_output(stdout: &[u8], stderr: &[u8]) -> String {
    let mut s = String::from_utf8_lossy(stdout).into_owned();
    let err = String::from_utf8_lossy(stderr);
    if !err.is_empty() {
        if !s.is_empty() {
            s.push('\n');
        }
        s.push_str(&err);
    }
    s
}

/// Operator-facing hint from `docker compose` stdout/stderr.
fn compose_failure_hint(output: &str) -> &'static str {
    let t = output.to_ascii_lowercase();
    if t.contains("permission denied") && t.contains("docker.sock") {
        "the current user cannot reach /var/run/docker.sock. Add it to the docker \
         group (`sudo usermod -aG docker $USER`, then log in again) and re-run --metrics"
    } else if t.contains("cannot connect to the docker daemon")
        || t.contains("is the docker daemon running")
    {
        "Docker CLI is present but the daemon is not running. Start it with \
         `sudo systemctl start docker`, then re-run --metrics"
    } else if t.contains("unknown command") || t.contains("'compose' is not a docker command") {
        "the Docker compose plugin is missing. Install docker-compose-plugin, then re-run --metrics"
    } else {
        "check `docker compose up -d` in deploy/observability, then re-run --metrics \
         (or ./scripts/run-observability.sh). Or skip --metrics and scrape :9100 only"
    }
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

    #[test]
    fn hint_for_daemon_down() {
        let s = "Cannot connect to the Docker daemon at unix:///var/run/docker.sock. \
                 Is the docker daemon running?";
        assert!(compose_failure_hint(s).contains("systemctl start docker"));
    }

    #[test]
    fn hint_for_socket_permission() {
        let s = "permission denied while trying to connect to the Docker daemon socket \
                 at unix:///var/run/docker.sock";
        assert!(compose_failure_hint(s).contains("docker group"));
    }
}
