//! In-memory durable store with schema gate and flush-before-publish.

use crate::batch::{apply_puts, BatchPut, WriteBatch};
use crate::error::{Result, StorageError};
use crate::schema::{assert_schema, SCHEMA_ID, SCHEMA_VERSION};
use ethean_primitives::Hash32;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Opened database handle (process-local; RocksDB backend deferred).
#[derive(Debug)]
pub struct Database {
    schema_id: String,
    schema_version: u32,
    data: HashMap<(String, Vec<u8>), (Vec<u8>, Hash32)>,
    /// Highest signer watermark leaf (never rewind on chain rollback).
    signer_watermark: u64,
}

impl Database {
    /// Open or create with pinned schema.
    pub fn open() -> Result<Self> {
        Ok(Self {
            schema_id: SCHEMA_ID.to_string(),
            schema_version: SCHEMA_VERSION,
            data: HashMap::new(),
            signer_watermark: 0,
        })
    }

    /// Refuse legacy directory markers.
    pub fn refuse_legacy_path(path: &str) -> Result<()> {
        let lower = path.to_ascii_lowercase();
        if lower.contains("panro") || lower.ends_with(".json") {
            return Err(StorageError::LegacyRefused(path.to_string()));
        }
        Ok(())
    }

    /// Verify schema on reopen.
    pub fn verify_schema(&self) -> Result<()> {
        assert_schema(&self.schema_id, self.schema_version)
    }

    /// Checksum helper for callers.
    pub fn checksum(value: &[u8]) -> Hash32 {
        Sha256::digest(value).into()
    }

    /// Apply a batch and mark it flushed (simulates fsync success).
    pub fn commit(&mut self, batch: &mut WriteBatch) -> Result<()> {
        self.verify_schema()?;
        let puts = batch.take_puts();
        apply_puts(&mut self.data, &puts);
        batch.mark_flushed();
        Ok(())
    }

    /// Read a value if present and checksum matches.
    pub fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>> {
        match self.data.get(&(table.to_string(), key.to_vec())) {
            None => Ok(None),
            Some((value, sum)) => {
                let got = Self::checksum(value);
                if &got != sum {
                    return Err(StorageError::ChecksumMismatch(table.to_string()));
                }
                Ok(Some(value.clone()))
            }
        }
    }

    /// Put helper building a single-item batch.
    pub fn put_ssz(&mut self, table: &'static str, key: &[u8], value: &[u8]) -> Result<()> {
        let mut batch = WriteBatch::default();
        batch.put(BatchPut {
            table,
            key: key.to_vec(),
            value: value.to_vec(),
            checksum: Self::checksum(value),
        });
        self.commit(&mut batch)?;
        if !batch.is_flushed() {
            return Err(StorageError::NotDurable);
        }
        Ok(())
    }

    /// Advance signer watermark monotonically.
    pub fn advance_signer_watermark(&mut self, leaf: u64) -> Result<()> {
        if leaf < self.signer_watermark {
            return Err(StorageError::Corruption(
                "signer watermark rewind refused".into(),
            ));
        }
        self.signer_watermark = leaf;
        Ok(())
    }

    /// Current signer watermark.
    pub fn signer_watermark(&self) -> u64 {
        self.signer_watermark
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get_roundtrip() {
        let mut db = Database::open().unwrap();
        db.put_ssz("blocks", b"root", b"ssz-bytes").unwrap();
        assert_eq!(db.get("blocks", b"root").unwrap().unwrap(), b"ssz-bytes");
    }

    #[test]
    fn refuses_legacy_json_path() {
        assert!(Database::refuse_legacy_path("/data/panro-state.json").is_err());
    }

    #[test]
    fn watermark_never_rewinds() {
        let mut db = Database::open().unwrap();
        db.advance_signer_watermark(10).unwrap();
        assert!(db.advance_signer_watermark(9).is_err());
    }
}
