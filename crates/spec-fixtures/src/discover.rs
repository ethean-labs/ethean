//! Discover filled leanSpec JSON fixtures under a cache root.

use std::path::{Path, PathBuf};

/// Env var pointing at an extracted leanSpec fixtures tree (contains `fixtures/`).
pub const FIXTURES_ENV: &str = "ETHEAN_LEANSPEC_FIXTURES";

/// Resolve fixtures root from [`FIXTURES_ENV`] when set and present on disk.
pub fn fixtures_root_from_env() -> Option<PathBuf> {
    let raw = std::env::var_os(FIXTURES_ENV)?;
    let path = PathBuf::from(raw);
    if path.is_dir() {
        Some(path)
    } else {
        None
    }
}

/// Walk `root` for `*.json` under `fixtures/consensus` (or `consensus` / root).
pub fn discover_json_fixtures(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut candidates = Vec::new();
    let consensus = [
        root.join("fixtures").join("consensus"),
        root.join("consensus"),
        root.to_path_buf(),
    ];
    for base in &consensus {
        if base.is_dir() {
            walk_json(base, &mut candidates)?;
            break;
        }
    }
    candidates.sort();
    Ok(candidates)
}

fn walk_json(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_json(&path, out)?;
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("json"))
        {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_absent_is_none() {
        std::env::remove_var(FIXTURES_ENV);
        assert!(fixtures_root_from_env().is_none());
    }

    #[test]
    fn discovers_under_env_cache_if_present() {
        let Some(root) = fixtures_root_from_env() else {
            eprintln!("skip: set {FIXTURES_ENV} after fetch-leanspec-fixtures.ps1");
            return;
        };
        let files = discover_json_fixtures(&root).expect("walk");
        assert!(
            !files.is_empty(),
            "expected json under {}",
            root.display()
        );
    }
}
