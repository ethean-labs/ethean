//! Database migration framework
//!
//! Handles database schema migrations and version management.

use crate::storage::database::{Database, DatabaseError};
use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

/// Migration trait for database schema changes
pub trait Migration: Send + Sync {
    /// Migration version (should be unique and incrementing)
    fn version(&self) -> u64;
    
    /// Migration description
    fn description(&self) -> &str;
    
    /// Apply migration
    fn up(&self, database: &Database) -> Result<(), DatabaseError>;
    
    /// Rollback migration (optional)
    fn down(&self, database: &Database) -> Result<(), DatabaseError> {
        // Default implementation does nothing
        let _ = database;
        Err(DatabaseError::InvalidData("Rollback not implemented".to_string()))
    }
}

/// Migration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationMetadata {
    /// Migration version
    pub version: u64,
    /// Migration description
    pub description: String,
    /// Applied timestamp
    pub applied_at: u64,
    /// Applied successfully
    pub success: bool,
}

/// Migration manager
pub struct MigrationManager {
    database: Database,
    migrations: BTreeMap<u64, Box<dyn Migration>>,
}

impl MigrationManager {
    /// Create new migration manager
    pub fn new(database: Database) -> Self {
        Self {
            database,
            migrations: BTreeMap::new(),
        }
    }

    /// Add migration
    pub fn add_migration(&mut self, migration: Box<dyn Migration>) {
        let version = migration.version();
        self.migrations.insert(version, migration);
    }

    /// Get current schema version
    pub fn current_version(&self) -> Result<u64, DatabaseError> {
        let key = Self::version_key();
        
        if let Some(version_bytes) = self.database.get(&key)? {
            if version_bytes.len() == 8 {
                let version = u64::from_be_bytes([
                    version_bytes[0], version_bytes[1], version_bytes[2], version_bytes[3],
                    version_bytes[4], version_bytes[5], version_bytes[6], version_bytes[7],
                ]);
                return Ok(version);
            }
        }
        
        Ok(0) // No version set means version 0
    }

    /// Set schema version
    fn set_version(&self, version: u64) -> Result<(), DatabaseError> {
        let key = Self::version_key();
        self.database.put(&key, &version.to_be_bytes())
    }

    /// Apply all pending migrations
    pub fn migrate(&self) -> Result<Vec<u64>, DatabaseError> {
        let current_version = self.current_version()?;
        let mut applied = Vec::new();
        
        for (&version, migration) in &self.migrations {
            if version > current_version {
                println!("Applying migration {}: {}", version, migration.description());
                
                let metadata = MigrationMetadata {
                    version,
                    description: migration.description().to_string(),
                    applied_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    success: false,
                };
                
                // Store migration attempt
                let meta_key = Self::migration_key(version);
                self.database.put_typed(&meta_key, &metadata)?;
                
                // Apply migration
                migration.up(&self.database)?;
                
                // Mark as successful
                let success_metadata = MigrationMetadata {
                    success: true,
                    ..metadata
                };
                self.database.put_typed(&meta_key, &success_metadata)?;
                
                // Update schema version
                self.set_version(version)?;
                applied.push(version);
                
                println!("Successfully applied migration {}", version);
            }
        }
        
        Ok(applied)
    }

    /// Get migration history
    pub fn migration_history(&self) -> Result<Vec<MigrationMetadata>, DatabaseError> {
        let prefix = b"migration:";
        let keys = self.database.keys_with_prefix(prefix)?;
        
        let mut history = Vec::new();
        for key in keys {
            if let Some(data) = self.database.get(&key)? {
                if let Ok(metadata) = serde_json::from_slice::<MigrationMetadata>(&data) {
                    history.push(metadata);
                }
            }
        }
        
        // Sort by version
        history.sort_by_key(|m| m.version);
        Ok(history)
    }

    /// Check if database needs migration
    pub fn needs_migration(&self) -> Result<bool, DatabaseError> {
        let current_version = self.current_version()?;
        let latest_version = self.migrations.keys().last().copied().unwrap_or(0);
        Ok(current_version < latest_version)
    }

    /// Get pending migrations
    pub fn pending_migrations(&self) -> Result<Vec<u64>, DatabaseError> {
        let current_version = self.current_version()?;
        let pending: Vec<u64> = self.migrations
            .keys()
            .filter(|&&v| v > current_version)
            .copied()
            .collect();
        Ok(pending)
    }

    /// Version key for database
    fn version_key() -> Vec<u8> {
        b"schema_version".to_vec()
    }

    /// Migration metadata key
    fn migration_key(version: u64) -> Vec<u8> {
        let mut key = b"migration:".to_vec();
        key.extend_from_slice(&version.to_be_bytes());
        key
    }
}

/// Built-in migrations for Panro
pub mod builtin {
    use super::*;

    /// Initial schema migration
    pub struct InitialSchema;

    impl Migration for InitialSchema {
        fn version(&self) -> u64 {
            1
        }

        fn description(&self) -> &str {
            "Initialize basic schema with state and block storage"
        }

        fn up(&self, _database: &Database) -> Result<(), DatabaseError> {
            // Create initial schema
            // In a real implementation, this would create necessary tables/indices
            println!("Initializing basic schema");
            Ok(())
        }
    }

    /// WOTS signature storage migration
    pub struct WotsSignatureStorage;

    impl Migration for WotsSignatureStorage {
        fn version(&self) -> u64 {
            2
        }

        fn description(&self) -> &str {
            "Add WOTS signature storage support"
        }

        fn up(&self, _database: &Database) -> Result<(), DatabaseError> {
            // Add WOTS signature indices and storage
            println!("Adding WOTS signature storage");
            Ok(())
        }
    }

    /// Enhanced state storage migration
    pub struct EnhancedStateStorage;

    impl Migration for EnhancedStateStorage {
        fn version(&self) -> u64 {
            3
        }

        fn description(&self) -> &str {
            "Enhanced state storage with compression and indexing"
        }

        fn up(&self, _database: &Database) -> Result<(), DatabaseError> {
            // Add enhanced state storage features
            println!("Upgrading state storage");
            Ok(())
        }
    }

    /// Get all builtin migrations
    pub fn get_builtin_migrations() -> Vec<Box<dyn Migration>> {
        vec![
            Box::new(InitialSchema),
            Box::new(WotsSignatureStorage),
            Box::new(EnhancedStateStorage),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestMigration {
        version: u64,
        description: String,
    }

    impl Migration for TestMigration {
        fn version(&self) -> u64 {
            self.version
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn up(&self, _database: &Database) -> Result<(), DatabaseError> {
            Ok(())
        }
    }

    #[test]
    fn test_migration_manager() {
        let database = Database::in_memory();
        let mut manager = MigrationManager::new(database);
        
        // Initially no migrations needed
        assert!(!manager.needs_migration().unwrap());
        assert_eq!(manager.current_version().unwrap(), 0);
        
        // Add test migration
        let migration = TestMigration { 
            version: 1, 
            description: "Test migration".to_string(),
        };
        manager.add_migration(Box::new(migration));
        
        // Now needs migration
        assert!(manager.needs_migration().unwrap());
        
        let pending = manager.pending_migrations().unwrap();
        assert_eq!(pending, vec![1]);
        
        // Apply migrations
        let applied = manager.migrate().unwrap();
        assert_eq!(applied, vec![1]);
        
        // Check version updated
        assert_eq!(manager.current_version().unwrap(), 1);
        assert!(!manager.needs_migration().unwrap());
    }

    #[test]
    fn test_builtin_migrations() {
        let migrations = builtin::get_builtin_migrations();
        assert_eq!(migrations.len(), 3);
        
        // Check versions are sequential
        assert_eq!(migrations[0].version(), 1);
        assert_eq!(migrations[1].version(), 2);
        assert_eq!(migrations[2].version(), 3);
    }

    #[test]
    fn test_migration_metadata() {
        let metadata = MigrationMetadata {
            version: 1,
            description: "Test".to_string(),
            applied_at: 1234567890,
            success: true,
        };
        
        let json = serde_json::to_string(&metadata).unwrap();
        let parsed: MigrationMetadata = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.description, "Test");
        assert!(parsed.success);
    }
}
