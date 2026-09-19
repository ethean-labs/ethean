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
             (./scripts/run-observability.sh or .\\scripts\\run-observability.ps1)"
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
                 Install and start Docker Desktop, then re-run with --metrics \
                 (or .\\scripts\\run-observability.ps1). Port change is not needed; \
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
    if t.contains("dockerdesktoplinuxengine")
        || t.contains("500 internal server error")
        || t.contains("/_ping")
        || t.contains("requested api version")
    {
        "Docker Desktop UI is up but the Linux engine is down (HTTP 500 on \
         dockerDesktopLinuxEngine/_ping). On Windows this is usually Hyper-V / \
         Virtual Machine Platform off (HCS_E_HYPERV_NOT_INSTALLED). Admin PowerShell: \
         wsl.exe --install --no-distribution, enable Virtual Machine Platform, reboot, \
         then start Docker Desktop and wait until Engine is running"
    } else if t.contains("wsl") || t.contains("sistem dosyaya") {
        "often Docker Desktop/WSL (wsl.exe exit 1, 'Sistem dosyaya erişemiyor'). \
         Fix: restart Docker Desktop, or `wsl --update` / enable Virtual Machine \
         Platform; or skip --metrics and use curl on :9100 only"
    } else if t.contains("npipe") || t.contains("cannot connect to the docker") {
        "Docker CLI is present but the daemon is not ready. Start Docker Desktop, \
         wait until Engine is running, then re-run --metrics"
    } else {
        "start Docker Desktop, wait until Engine is running, then re-run --metrics \
         (or .\\scripts\\run-observability.ps1). Or skip --metrics and scrape :9100 only"
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
    fn hint_for_engine_500_ping() {
        let s = "request returned 500 Internal Server Error for API route and version \
                 http://%2F%2F.%2Fpipe%2FdockerDesktopLinuxEngine/_ping, check if the \
                 server supports the requested API version";
        assert!(compose_failure_hint(s).contains("Hyper-V"));
    }

    #[test]
    fn hint_for_npipe_daemon_down() {
        let s = "failed to connect to the docker API at npipe://./pipe/docker_engine";
        assert!(compose_failure_hint(s).contains("daemon is not ready"));
    }
}
