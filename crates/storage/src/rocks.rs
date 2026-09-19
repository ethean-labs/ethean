//! Path-backed store open (RocksDB when feature enabled).

use crate::db::Database;
use crate::error::Result;
#[cfg(any(test, not(feature = "rocksdb")))]
use crate::error::StorageError;
use std::path::Path;

/// Attempt to open a path-backed store.
///
/// Always refuses legacy markers first. Without the `rocksdb` feature, returns
/// [`StorageError::BackendPending`]. With the feature, opens RocksDB at `path`.
pub fn open_path(path: &str) -> Result<Database> {
    Database::refuse_legacy_path(path)?;
    open_rocks_inner(Path::new(path))
}

#[cfg(feature = "rocksdb")]
fn open_rocks_inner(path: &Path) -> Result<Database> {
    Database::open_rocks(path)
}

#[cfg(not(feature = "rocksdb"))]
fn open_rocks_inner(_path: &Path) -> Result<Database> {
    Err(StorageError::BackendPending(
        "enable ethean-storage/rocksdb to open a path-backed store",
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

    #[cfg(not(feature = "rocksdb"))]
    #[test]
    fn pending_without_feature() {
        assert!(matches!(
            open_path("/tmp/ethean-lc.db"),
            Err(StorageError::BackendPending(_))
        ));
    }

    #[cfg(feature = "rocksdb")]
    #[test]
    fn rocks_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("ethean-lc");
        let mut db = open_path(path.to_str().unwrap()).expect("open rocks");
        assert!(db.is_rocks());
        db.put_ssz("blocks", b"r1", b"payload").unwrap();
        assert_eq!(db.get("blocks", b"r1").unwrap().unwrap(), b"payload");
        db.advance_signer_watermark(3).unwrap();
        drop(db);
        let db2 = open_path(path.to_str().unwrap()).expect("reopen");
        assert_eq!(db2.get("blocks", b"r1").unwrap().unwrap(), b"payload");
        assert_eq!(db2.signer_watermark(), 3);
    }
}
