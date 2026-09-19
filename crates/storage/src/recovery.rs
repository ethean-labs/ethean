//! Startup recovery checks (fail closed on corruption).

use crate::db::Database;
use crate::error::{Result, StorageError};
use crate::tables::{META_FINALIZED, META_HEAD, META_JUSTIFIED, TABLE_METADATA};

/// Verify schema and that metadata roots (if present) have matching block rows.
pub fn recover_on_open(db: &Database) -> Result<()> {
    db.verify_schema()?;
    for key in [META_HEAD, META_JUSTIFIED, META_FINALIZED] {
        if let Some(root) = db.get(TABLE_METADATA, key)? {
            if root.len() != 32 {
                return Err(StorageError::Corruption(
                    "metadata root length".into(),
                ));
            }
            // Presence of metadata without a block is quarantined.
            if db.get(crate::tables::TABLE_BLOCKS, &root)?.is_none() {
                return Err(StorageError::Corruption(
                    "metadata points at missing block".into(),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_db_recovers() {
        let db = Database::open().unwrap();
        assert!(recover_on_open(&db).is_ok());
    }

    #[test]
    fn orphan_metadata_fails() {
        let mut db = Database::open().unwrap();
        db.put_ssz(TABLE_METADATA, META_HEAD, &[9u8; 32]).unwrap();
        assert!(recover_on_open(&db).is_err());
    }
}
