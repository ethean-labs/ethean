//! Database backup and recovery module
//!
//! Provides comprehensive backup strategies, incremental backups,
//! and automated recovery mechanisms for data integrity.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use tokio::fs;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::collections::HashMap;
use crate::storage::database::{Database, DatabaseError};

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Backup directory path
    pub backup_dir: PathBuf,
    /// Maximum number of backups to retain
    pub max_backups: usize,
    /// Compression enabled
    pub compress: bool,
    /// Incremental backup enabled
    pub incremental: bool,
    /// Backup interval in seconds
    pub interval_seconds: u64,
    /// Include indexes in backup
    pub include_indexes: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            backup_dir: PathBuf::from("./backups"),
            max_backups: 10,
            compress: true,
            incremental: true,
            interval_seconds: 3600, // 1 hour
            include_indexes: true,
        }
    }
}

/// Backup metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub timestamp: u64,
    pub backup_type: BackupType,
    pub size_bytes: u64,
    pub checksum: String,
    pub database_version: String,
    pub slot_range: Option<(u64, u64)>,
    pub epoch_range: Option<(u64, u64)>,
    pub compressed: bool,
    pub incremental_base: Option<String>,
}

/// Backup type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BackupType {
    Full,
    Incremental { base_backup: String },
    Differential { base_backup: String },
}

/// Recovery options
#[derive(Debug, Clone)]
pub struct RecoveryOptions {
    pub target_slot: Option<u64>,
    pub target_epoch: Option<u64>,
    pub verify_integrity: bool,
    pub restore_indexes: bool,
    pub force_overwrite: bool,
}

impl Default for RecoveryOptions {
    fn default() -> Self {
        Self {
            target_slot: None,
            target_epoch: None,
            verify_integrity: true,
            restore_indexes: true,
            force_overwrite: false,
        }
    }
}

/// Backup manager
pub struct BackupManager {
    config: BackupConfig,
    database: Database,
    last_backup_time: Option<SystemTime>,
}

impl BackupManager {
    /// Create new backup manager
    pub fn new(config: BackupConfig, database: Database) -> Self {
        Self {
            config,
            database,
            last_backup_time: None,
        }
    }
    
    /// Create full backup
    pub async fn create_full_backup(&mut self) -> Result<BackupMetadata, DatabaseError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let backup_name = format!("full_backup_{}", timestamp);
        let backup_path = self.config.backup_dir.join(&backup_name);
        
        // Create backup directory
        fs::create_dir_all(&backup_path).await
            .map_err(|e| DatabaseError::IoError(e.to_string()))?;
        
        // Get database snapshot
        let snapshot = self.database.create_snapshot()?;
        let mut total_size = 0u64;
        let mut file_count = 0usize;
        
        // Backup all key-value pairs
        let keys = self.database.all_keys()?;
        
        for (i, key) in keys.iter().enumerate() {
            if let Some(value) = self.database.get_from_snapshot(&snapshot, key)? {
                let file_name = format!("data_{:08}.dat", i);
                let file_path = backup_path.join(&file_name);
                
                let mut file = fs::File::create(&file_path).await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))?;
                
                // Write key length, key, value length, value
                file.write_u32_le(key.len() as u32).await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))?;
                file.write_all(key).await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))?;
                file.write_u32_le(value.len() as u32).await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))?;
                file.write_all(&value).await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))?;
                
                total_size += key.len() as u64 + value.len() as u64 + 8; // +8 for length fields
                file_count += 1;
            }
        }
        
        // Calculate checksum
        let checksum = self.calculate_backup_checksum(&backup_path).await?;
        
        // Get slot and epoch ranges
        let slot_range = self.get_slot_range().await?;
        let epoch_range = self.get_epoch_range().await?;
        
        // Create metadata
        let metadata = BackupMetadata {
            timestamp,
            backup_type: BackupType::Full,
            size_bytes: total_size,
            checksum,
            database_version: "1.0.0".to_string(),
            slot_range,
            epoch_range,
            compressed: false, // TODO: Implement compression
            incremental_base: None,
        };
        
        // Save metadata
        let metadata_path = backup_path.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;
        
        fs::write(&metadata_path, metadata_json).await
            .map_err(|e| DatabaseError::IoError(e.to_string()))?;
        
        self.last_backup_time = Some(SystemTime::now());
        
        // Cleanup old backups
        self.cleanup_old_backups().await?;
        
        println!("Full backup completed: {} files, {} bytes", file_count, total_size);
        
        Ok(metadata)
    }
    
    /// Create incremental backup
    pub async fn create_incremental_backup(&mut self, base_backup: &str) -> Result<BackupMetadata, DatabaseError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let backup_name = format!("incremental_backup_{}", timestamp);
        let backup_path = self.config.backup_dir.join(&backup_name);
        
        // Create backup directory
        fs::create_dir_all(&backup_path).await
            .map_err(|e| DatabaseError::IoError(e.to_string()))?;
        
        // Load base backup metadata
        let base_backup_path = self.config.backup_dir.join(base_backup);
        let base_metadata = self.load_backup_metadata(&base_backup_path).await?;
        
        // Get changed keys since base backup
        let changed_keys = self.get_changed_keys_since(base_metadata.timestamp).await?;
        
        let mut total_size = 0u64;
        let mut file_count = 0usize;
        
        // Backup only changed keys
        for (i, key) in changed_keys.iter().enumerate() {
            if let Some(value) = self.database.get(key)? {
                let file_name = format!("delta_{:08}.dat", i);
                let file_path = backup_path.join(&file_name);
                
                let mut file = fs::File::create(&file_path).await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))?;
                
                // Write key length, key, value length, value
                file.write_u32_le(key.len() as u32).await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))?;
                file.write_all(key).await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))?;
                file.write_u32_le(value.len() as u32).await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))?;
                file.write_all(&value).await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))?;
                
                total_size += key.len() as u64 + value.len() as u64 + 8;
                file_count += 1;
            }
        }
        
        // Calculate checksum
        let checksum = self.calculate_backup_checksum(&backup_path).await?;
        
        // Get current ranges
        let slot_range = self.get_slot_range().await?;
        let epoch_range = self.get_epoch_range().await?;
        
        // Create metadata
        let metadata = BackupMetadata {
            timestamp,
            backup_type: BackupType::Incremental {
                base_backup: base_backup.to_string(),
            },
            size_bytes: total_size,
            checksum,
            database_version: "1.0.0".to_string(),
            slot_range,
            epoch_range,
            compressed: false,
            incremental_base: Some(base_backup.to_string()),
        };
        
        // Save metadata
        let metadata_path = backup_path.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;
        
        fs::write(&metadata_path, metadata_json).await
            .map_err(|e| DatabaseError::IoError(e.to_string()))?;
        
        self.last_backup_time = Some(SystemTime::now());
        
        println!("Incremental backup completed: {} files, {} bytes", file_count, total_size);
        
        Ok(metadata)
    }
    
    /// Restore from backup
    pub async fn restore_backup(&self, backup_name: &str, options: RecoveryOptions) -> Result<(), DatabaseError> {
        let backup_path = self.config.backup_dir.join(backup_name);
        
        if !backup_path.exists() {
            return Err(DatabaseError::NotFound(format!("Backup not found: {}", backup_name)));
        }
        
        // Load metadata
        let metadata = self.load_backup_metadata(&backup_path).await?;
        
        // Verify integrity if requested
        if options.verify_integrity {
            self.verify_backup_integrity(&backup_path, &metadata).await?;
        }
        
        // Clear database if force overwrite
        if options.force_overwrite {
            self.database.clear_all()?;
        }
        
        match metadata.backup_type {
            BackupType::Full => {
                self.restore_full_backup(&backup_path, &options).await?;
            }
            BackupType::Incremental { ref base_backup } => {
                // First restore base backup
                self.restore_backup(base_backup, options.clone()).await?;
                // Then apply incremental changes
                self.restore_incremental_backup(&backup_path, &options).await?;
            }
            BackupType::Differential { ref base_backup } => {
                // Restore base backup first
                self.restore_backup(base_backup, options.clone()).await?;
                // Then apply differential changes
                self.restore_differential_backup(&backup_path, &options).await?;
            }
        }
        
        println!("Backup restored successfully: {}", backup_name);
        Ok(())
    }
    
    /// List available backups
    pub async fn list_backups(&self) -> Result<Vec<BackupMetadata>, DatabaseError> {
        let mut backups = Vec::new();
        
        if !self.config.backup_dir.exists() {
            return Ok(backups);
        }
        
        let mut entries = fs::read_dir(&self.config.backup_dir).await
            .map_err(|e| DatabaseError::IoError(e.to_string()))?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| DatabaseError::IoError(e.to_string()))? {
            
            if entry.file_type().await
                .map_err(|e| DatabaseError::IoError(e.to_string()))?
                .is_dir() {
                
                let backup_path = entry.path();
                if let Ok(metadata) = self.load_backup_metadata(&backup_path).await {
                    backups.push(metadata);
                }
            }
        }
        
        // Sort by timestamp (newest first)
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(backups)
    }
    
    /// Auto backup if needed
    pub async fn auto_backup_if_needed(&mut self) -> Result<Option<BackupMetadata>, DatabaseError> {
        if let Some(last_backup) = self.last_backup_time {
            let elapsed = last_backup.elapsed().unwrap_or_default();
            if elapsed.as_secs() < self.config.interval_seconds {
                return Ok(None);
            }
        }
        
        // Determine backup type
        let backups = self.list_backups().await?;
        
        if backups.is_empty() || !self.config.incremental {
            // Create full backup
            let metadata = self.create_full_backup().await?;
            Ok(Some(metadata))
        } else {
            // Create incremental backup based on latest full backup
            let base_backup = backups.iter()
                .find(|b| matches!(b.backup_type, BackupType::Full))
                .ok_or_else(|| DatabaseError::NotFound("No full backup found".to_string()))?;
            
            let base_name = format!("full_backup_{}", base_backup.timestamp);
            let metadata = self.create_incremental_backup(&base_name).await?;
            Ok(Some(metadata))
        }
    }
    
    // Helper methods
    
    async fn load_backup_metadata(&self, backup_path: &Path) -> Result<BackupMetadata, DatabaseError> {
        let metadata_path = backup_path.join("metadata.json");
        let metadata_json = fs::read_to_string(&metadata_path).await
            .map_err(|e| DatabaseError::IoError(e.to_string()))?;
        
        serde_json::from_str(&metadata_json)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))
    }
    
    async fn calculate_backup_checksum(&self, backup_path: &Path) -> Result<String, DatabaseError> {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        let mut entries = fs::read_dir(backup_path).await
            .map_err(|e| DatabaseError::IoError(e.to_string()))?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| DatabaseError::IoError(e.to_string()))? {
            
            if entry.file_name() != "metadata.json" {
                let file_path = entry.path();
                let content = fs::read(&file_path).await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))?;
                hasher.update(&content);
            }
        }
        
        Ok(format!("{:x}", hasher.finalize()))
    }
    
    async fn verify_backup_integrity(&self, backup_path: &Path, metadata: &BackupMetadata) -> Result<(), DatabaseError> {
        let calculated_checksum = self.calculate_backup_checksum(backup_path).await?;
        
        if calculated_checksum != metadata.checksum {
            return Err(DatabaseError::CorruptedData(
                "Backup checksum verification failed".to_string()
            ));
        }
        
        Ok(())
    }
    
    async fn get_slot_range(&self) -> Result<Option<(u64, u64)>, DatabaseError> {
        // Implementation would scan for min/max slot values
        // For now, return None
        Ok(None)
    }
    
    async fn get_epoch_range(&self) -> Result<Option<(u64, u64)>, DatabaseError> {
        // Implementation would scan for min/max epoch values
        // For now, return None
        Ok(None)
    }
    
    async fn get_changed_keys_since(&self, timestamp: u64) -> Result<Vec<Vec<u8>>, DatabaseError> {
        // Implementation would track changes with timestamps
        // For now, return all keys
        self.database.all_keys()
    }
    
    async fn restore_full_backup(&self, backup_path: &Path, _options: &RecoveryOptions) -> Result<(), DatabaseError> {
        let mut entries = fs::read_dir(backup_path).await
            .map_err(|e| DatabaseError::IoError(e.to_string()))?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| DatabaseError::IoError(e.to_string()))? {
            
            let file_name = entry.file_name();
            if file_name.to_string_lossy().starts_with("data_") {
                let file_path = entry.path();
                let mut file = fs::File::open(&file_path).await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))?;
                
                // Read key
                let key_len = file.read_u32_le().await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))? as usize;
                let mut key = vec![0u8; key_len];
                file.read_exact(&mut key).await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))?;
                
                // Read value
                let value_len = file.read_u32_le().await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))? as usize;
                let mut value = vec![0u8; value_len];
                file.read_exact(&mut value).await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))?;
                
                // Store in database
                self.database.put(&key, &value)?;
            }
        }
        
        Ok(())
    }
    
    async fn restore_incremental_backup(&self, backup_path: &Path, _options: &RecoveryOptions) -> Result<(), DatabaseError> {
        let mut entries = fs::read_dir(backup_path).await
            .map_err(|e| DatabaseError::IoError(e.to_string()))?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| DatabaseError::IoError(e.to_string()))? {
            
            let file_name = entry.file_name();
            if file_name.to_string_lossy().starts_with("delta_") {
                let file_path = entry.path();
                let mut file = fs::File::open(&file_path).await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))?;
                
                // Read and apply delta changes
                let key_len = file.read_u32_le().await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))? as usize;
                let mut key = vec![0u8; key_len];
                file.read_exact(&mut key).await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))?;
                
                let value_len = file.read_u32_le().await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))? as usize;
                let mut value = vec![0u8; value_len];
                file.read_exact(&mut value).await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))?;
                
                self.database.put(&key, &value)?;
            }
        }
        
        Ok(())
    }
    
    async fn restore_differential_backup(&self, backup_path: &Path, options: &RecoveryOptions) -> Result<(), DatabaseError> {
        // For now, differential backup works same as incremental
        self.restore_incremental_backup(backup_path, options).await
    }
    
    async fn cleanup_old_backups(&self) -> Result<(), DatabaseError> {
        let mut backups = self.list_backups().await?;
        
        if backups.len() <= self.config.max_backups {
            return Ok(());
        }
        
        // Sort by timestamp (oldest first for removal)
        backups.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        // Remove oldest backups beyond the limit
        let to_remove = backups.len() - self.config.max_backups;
        
        for i in 0..to_remove {
            let backup_name = match &backups[i].backup_type {
                BackupType::Full => format!("full_backup_{}", backups[i].timestamp),
                BackupType::Incremental { .. } => format!("incremental_backup_{}", backups[i].timestamp),
                BackupType::Differential { .. } => format!("differential_backup_{}", backups[i].timestamp),
            };
            
            let backup_path = self.config.backup_dir.join(&backup_name);
            if backup_path.exists() {
                fs::remove_dir_all(&backup_path).await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))?;
                println!("Removed old backup: {}", backup_name);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::Database;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_full_backup_and_restore() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let backup_path = temp_dir.path().join("backups");
        
        // Create database and add some data
        let db = Database::open(&db_path).unwrap();
        db.put(b"key1", b"value1").unwrap();
        db.put(b"key2", b"value2").unwrap();
        
        // Create backup manager and perform backup
        let config = BackupConfig {
            backup_dir: backup_path.clone(),
            ..Default::default()
        };
        
        let mut backup_manager = BackupManager::new(config, db);
        let metadata = backup_manager.create_full_backup().await.unwrap();
        
        assert_eq!(metadata.backup_type, BackupType::Full);
        assert!(metadata.size_bytes > 0);
        
        // Create new database and restore
        let db2_path = temp_dir.path().join("test_db2");
        let db2 = Database::open(&db2_path).unwrap();
        let backup_manager2 = BackupManager::new(
            BackupConfig {
                backup_dir: backup_path,
                ..Default::default()
            },
            db2,
        );
        
        let backup_name = format!("full_backup_{}", metadata.timestamp);
        backup_manager2.restore_backup(
            &backup_name,
            RecoveryOptions::default(),
        ).await.unwrap();
        
        // Verify restored data
        assert_eq!(backup_manager2.database.get(b"key1").unwrap().unwrap(), b"value1");
        assert_eq!(backup_manager2.database.get(b"key2").unwrap().unwrap(), b"value2");
    }
}
