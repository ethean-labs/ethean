//! On-disk layout under `--data-dir`.

use crate::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};

const GENESIS_JSON: &str = "genesis.json";
const GENESIS_SSZ: &str = "genesis.ssz";
const STATE_SSZ: &str = "state.ssz";
const HEAD_ROOT: &str = "head.root";
const REDB_FILE: &str = "ethean.redb";
const BLOCKS_DIR: &str = "blocks";
const LOG_DIR: &str = "log";
const LEGACY_PIN: &str = "genesis_pin.json";
const LEGACY_HEAD: &str = "head_snap.json";

/// Paths under a durable data directory.
#[derive(Debug, Clone)]
pub struct PersistPaths {
    pub root: PathBuf,
}

impl PersistPaths {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn genesis_json(&self) -> PathBuf {
        self.root.join(GENESIS_JSON)
    }

    pub fn genesis_ssz(&self) -> PathBuf {
        self.root.join(GENESIS_SSZ)
    }

    pub fn state_ssz(&self) -> PathBuf {
        self.root.join(STATE_SSZ)
    }

    pub fn head_root(&self) -> PathBuf {
        self.root.join(HEAD_ROOT)
    }

    pub fn redb(&self) -> PathBuf {
        self.root.join(REDB_FILE)
    }

    pub fn blocks_dir(&self) -> PathBuf {
        self.root.join(BLOCKS_DIR)
    }

    pub fn log_dir(&self) -> PathBuf {
        self.root.join(LOG_DIR)
    }

    /// `log/ethean-{YYYY-MM-DD-HHMMSS}-log` for one process start.
    pub fn run_log_path(&self, stamp: &str) -> PathBuf {
        self.log_dir().join(format!("ethean-{stamp}-log"))
    }

    pub fn block_ssz(&self, root_hex: &str) -> PathBuf {
        self.blocks_dir().join(format!("{root_hex}.ssz"))
    }

    pub fn legacy_pin(&self) -> PathBuf {
        self.root.join(LEGACY_PIN)
    }

    pub fn legacy_head(&self) -> PathBuf {
        self.root.join(LEGACY_HEAD)
    }

    pub fn ensure_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.root)
            .map_err(|e| Error::Config(format!("create data-dir {}: {e}", self.root.display())))?;
        fs::create_dir_all(self.blocks_dir()).map_err(|e| {
            Error::Config(format!(
                "create blocks dir {}: {e}",
                self.blocks_dir().display()
            ))
        })?;
        fs::create_dir_all(self.log_dir())
            .map_err(|e| Error::Config(format!("create log dir {}: {e}", self.log_dir().display())))
    }
}

/// UTC `YYYY-MM-DD-HHMMSS` used in `ethean-{stamp}-log` names.
pub fn utc_run_stamp(unix_secs: u64) -> String {
    let sod = (unix_secs % 86_400) as u32;
    let hh = sod / 3600;
    let mm = (sod % 3600) / 60;
    let ss = sod % 60;
    let (year, month, day) = civil_from_unix_days((unix_secs / 86_400) as i64);
    format!("{year:04}-{month:02}-{day:02}-{hh:02}{mm:02}{ss:02}")
}

fn civil_from_unix_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };
    (year as i32, month, day)
}

/// Empty `--data-dir` (chain files, logs, leftovers) so the next start is fresh.
/// Keeps the directory itself; deletes every child.
pub fn reset_chain_files(root: &Path) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    if root.is_file() {
        return remove_path(root);
    }
    let entries = fs::read_dir(root)
        .map_err(|e| Error::Config(format!("read data-dir {}: {e}", root.display())))?;
    for entry in entries {
        let entry = entry
            .map_err(|e| Error::Config(format!("read data-dir entry {}: {e}", root.display())))?;
        remove_path(&entry.path())?;
    }
    Ok(())
}

fn remove_path(path: &Path) -> Result<()> {
    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => {
            return Err(Error::Config(format!("stat {}: {e}", path.display())));
        }
    };
    if meta.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|e| Error::Config(format!("remove {}: {e}", path.display())))?;
    } else {
        fs::remove_file(path)
            .map_err(|e| Error::Config(format!("remove {}: {e}", path.display())))?;
    }
    Ok(())
}

/// Write `data` to `path` via a sibling `.tmp` then rename.
pub fn write_atomic(path: &Path, data: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, data).map_err(|e| Error::Config(format!("write {}: {e}", tmp.display())))?;
    if path.exists() {
        fs::remove_file(path)
            .map_err(|e| Error::Config(format!("replace {}: {e}", path.display())))?;
    }
    fs::rename(&tmp, path).map_err(|e| {
        Error::Config(format!(
            "rename {} -> {}: {e}",
            tmp.display(),
            path.display()
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_ethean_layout() {
        let p = PersistPaths::new("/tmp/ethean-data");
        assert!(p.redb().ends_with("ethean.redb"));
        assert!(p.genesis_json().ends_with("genesis.json"));
        assert!(p.state_ssz().ends_with("state.ssz"));
        assert!(
            p.block_ssz("ab").ends_with("blocks/ab.ssz")
                || p.block_ssz("ab").ends_with("blocks\\ab.ssz")
        );
        assert!(p.log_dir().ends_with("log"));
        let log = p.run_log_path("2026-09-20-040512");
        let name = log.file_name().unwrap().to_string_lossy();
        assert_eq!(name, "ethean-2026-09-20-040512-log");
    }

    #[test]
    fn utc_stamp_known_unix() {
        assert_eq!(utc_run_stamp(0), "1970-01-01-000000");
        assert_eq!(utc_run_stamp(1_000_000_000), "2001-09-09-014640");
    }

    #[test]
    fn reset_empties_data_dir_including_logs() {
        let dir = tempfile::tempdir().unwrap();
        let paths = PersistPaths::new(dir.path());
        paths.ensure_dir().unwrap();
        fs::write(paths.genesis_json(), b"{}").unwrap();
        fs::write(paths.redb(), b"x").unwrap();
        fs::write(paths.run_log_path("2026-09-20-040512"), b"log").unwrap();
        fs::write(dir.path().join("stray.txt"), b"x").unwrap();
        reset_chain_files(dir.path()).unwrap();
        assert!(dir.path().is_dir());
        assert!(!paths.genesis_json().exists());
        assert!(!paths.redb().exists());
        assert!(!paths.blocks_dir().exists());
        assert!(!paths.log_dir().exists());
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    }
}
