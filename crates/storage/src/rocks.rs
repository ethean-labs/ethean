//! RocksDB backend gate (Phase 11 open).
//!
//! Default builds stay on the in-memory [`crate::db::Database`]. Enabling the
//! `rocksdb` feature still fails closed until the real open path is wired.

use crate::db::Database;
use crate::error::{Result, StorageError};

/// Attempt to open a path-backed store.
///
/// Always refuses legacy markers first. Without a wired RocksDB open, returns
/// [`StorageError::BackendPending`].
pub fn open_path(path: &str) -> Result<Database> {
    Database::refuse_legacy_path(path)?;
    open_rocks_inner(path)
}

#[cfg(feature = "rocksdb")]
fn open_rocks_inner(_path: &str) -> Result<Database> {
    // Feature flag acknowledges the dependency; binding is still an open gate.
    Err(StorageError::BackendPending(
        "rocksdb feature enabled but open/bind not wired; refuse silent memory fallback",
    ))
}

#[cfg(not(feature = "rocksdb"))]
fn open_rocks_inner(_path: &str) -> Result<Database> {
    Err(StorageError::BackendPending(
        "enable ethean-storage/rocksdb after RocksDB bind lands",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_legacy_before_backend() {
        assert!(matches!(
            open_path("/tmp/panro.db"),
            Err(StorageError::LegacyRefused(_))
        ));
    }

    #[test]
    fn pending_without_silent_fallback() {
        assert!(matches!(
            open_path("/tmp/ethean-lc.db"),
            Err(StorageError::BackendPending(_))
        ));
    }
}
