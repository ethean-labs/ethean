//! Storage schema identity and version gates.

/// Frozen schema id for this migration snapshot.
pub const SCHEMA_ID: &str = "ethean-lc-d5-v1";

/// Monotonic schema version for forward-only migrations.
pub const SCHEMA_VERSION: u32 = 1;

/// Reject legacy / unknown schema labels at open.
pub fn assert_schema(id: &str, version: u32) -> Result<(), crate::error::StorageError> {
    if id != SCHEMA_ID {
        return Err(crate::error::StorageError::SchemaMismatch {
            expected: SCHEMA_ID,
            got: id.to_string(),
        });
    }
    if version != SCHEMA_VERSION {
        return Err(crate::error::StorageError::SchemaVersionMismatch {
            expected: SCHEMA_VERSION,
            got: version,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_pinned_schema() {
        assert!(assert_schema(SCHEMA_ID, SCHEMA_VERSION).is_ok());
    }

    #[test]
    fn rejects_foreign_schema() {
        assert!(assert_schema("panro-legacy", 1).is_err());
    }
}
