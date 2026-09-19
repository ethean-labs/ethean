//! RocksDB-backed key/value engine (feature `rocksdb`).

#![cfg(feature = "rocksdb")]

use crate::batch::BatchPut;
use crate::error::{Result, StorageError};
use crate::schema::{SCHEMA_ID, SCHEMA_VERSION};
use ethean_primitives::Hash32;
use rocksdb::{Options, DB};
use sha2::{Digest, Sha256};
use std::path::Path;

const META_SCHEMA_ID: &[u8] = b"__meta__/schema_id";
const META_SCHEMA_VER: &[u8] = b"__meta__/schema_version";
const META_WATERMARK: &[u8] = b"__meta__/signer_watermark";

/// Opened RocksDB handle with Lean schema metadata.
pub struct RocksEngine {
    db: DB,
}

impl std::fmt::Debug for RocksEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RocksEngine")
    }
}

impl RocksEngine {
    /// Open or create at `path`; initialize schema meta if absent.
    pub fn open(path: &Path) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)
            .map_err(|e| StorageError::Corruption(format!("rocksdb open: {e}")))?;
        let engine = Self { db };
        engine.ensure_schema()?;
        Ok(engine)
    }

    fn ensure_schema(&self) -> Result<()> {
        match self.db.get(META_SCHEMA_ID) {
            Ok(None) => {
                self.db
                    .put(META_SCHEMA_ID, SCHEMA_ID.as_bytes())
                    .map_err(|e| StorageError::Corruption(e.to_string()))?;
                self.db
                    .put(META_SCHEMA_VER, SCHEMA_VERSION.to_le_bytes())
                    .map_err(|e| StorageError::Corruption(e.to_string()))?;
                Ok(())
            }
            Ok(Some(id)) => {
                let got = String::from_utf8_lossy(&id).into_owned();
                if got != SCHEMA_ID {
                    return Err(StorageError::SchemaMismatch {
                        expected: SCHEMA_ID,
                        got,
                    });
                }
                let ver = self
                    .db
                    .get(META_SCHEMA_VER)
                    .map_err(|e| StorageError::Corruption(e.to_string()))?
                    .ok_or_else(|| StorageError::MissingKey("schema_version".into()))?;
                if ver.len() != 4 {
                    return Err(StorageError::Corruption("schema_version width".into()));
                }
                let mut buf = [0u8; 4];
                buf.copy_from_slice(&ver);
                let got = u32::from_le_bytes(buf);
                if got != SCHEMA_VERSION {
                    return Err(StorageError::SchemaVersionMismatch {
                        expected: SCHEMA_VERSION,
                        got,
                    });
                }
                Ok(())
            }
            Err(e) => Err(StorageError::Corruption(e.to_string())),
        }
    }

    fn encode_key(table: &str, key: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(2 + table.len() + key.len());
        out.extend_from_slice(&(table.len() as u16).to_le_bytes());
        out.extend_from_slice(table.as_bytes());
        out.extend_from_slice(key);
        out
    }

    fn checksum(value: &[u8]) -> Hash32 {
        Sha256::digest(value).into()
    }

    /// Apply puts then flush.
    pub fn commit_puts(&self, puts: &[BatchPut]) -> Result<()> {
        let mut batch = rocksdb::WriteBatch::default();
        for p in puts {
            let mut payload = Vec::with_capacity(32 + p.value.len());
            payload.extend_from_slice(&p.checksum);
            payload.extend_from_slice(&p.value);
            batch.put(Self::encode_key(p.table, &p.key), payload);
        }
        self.db
            .write(batch)
            .map_err(|e| StorageError::Corruption(e.to_string()))?;
        self.db
            .flush()
            .map_err(|e| StorageError::Corruption(format!("flush: {e}")))?;
        Ok(())
    }

    /// Read value if present and checksum matches.
    pub fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let raw = self
            .db
            .get(Self::encode_key(table, key))
            .map_err(|e| StorageError::Corruption(e.to_string()))?;
        match raw {
            None => Ok(None),
            Some(buf) if buf.len() < 32 => Err(StorageError::ChecksumMismatch(table.into())),
            Some(buf) => {
                let (sum, value) = buf.split_at(32);
                let mut expected = [0u8; 32];
                expected.copy_from_slice(sum);
                let got = Self::checksum(value);
                if got != expected {
                    return Err(StorageError::ChecksumMismatch(table.into()));
                }
                Ok(Some(value.to_vec()))
            }
        }
    }

    pub fn load_watermark(&self) -> Result<u64> {
        match self.db.get(META_WATERMARK) {
            Ok(None) => Ok(0),
            Ok(Some(v)) if v.len() == 8 => {
                let mut b = [0u8; 8];
                b.copy_from_slice(&v);
                Ok(u64::from_le_bytes(b))
            }
            Ok(_) => Err(StorageError::Corruption("watermark width".into())),
            Err(e) => Err(StorageError::Corruption(e.to_string())),
        }
    }

    pub fn store_watermark(&self, leaf: u64) -> Result<()> {
        self.db
            .put(META_WATERMARK, leaf.to_le_bytes())
            .map_err(|e| StorageError::Corruption(e.to_string()))?;
        self.db
            .flush()
            .map_err(|e| StorageError::Corruption(format!("flush watermark: {e}")))?;
        Ok(())
    }
}
