//! Network-Storage Integration Module
//!
//! Provides seamless integration between network and storage layers,
//! enabling real-time synchronization, conflict resolution, and
//! distributed coordination for Ethereum Beacon Chain operations.

pub mod network_storage;
pub mod sync_coordinator;
pub mod conflict_resolver;

pub use network_storage::{
    NetworkStorageBridge, IntegrationConfig, IntegrationError, IntegrationEvent,
    IntegrationStats, EventBus, ConsistencyReport, NetworkEventType, StorageEventType,
    SyncEventType, ConflictInfo, ConflictResolution, ConflictStrategy,
};

pub use sync_coordinator::{
    SyncCoordinator, ConsistencyManager, DistributedLockManager,
    SyncEvent, SyncResult, SyncMetadata, ConsistencyConfig, ConsistencyCheck,
    LockConfig, DistributedLock, SyncStats, ConsistencyStats, LockStats,
};

pub use conflict_resolver::{
    ConflictResolver, ConflictEvent, ConflictStats, TimestampStrategy,
    VectorClockStrategy, HashStrategy, ConsensusStrategy, ContentMergeStrategy,
    ConflictDetector, VectorClock, HashAlgorithm, MergeAlgorithm,
};

use crate::network::orchestrator::NetworkOrchestrator;
use crate::storage::StorageManager;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

/// Integration builder for easy setup
pub struct IntegrationBuilder {
    config: Option<IntegrationConfig>,
    network_orchestrator: Option<Arc<RwLock<NetworkOrchestrator>>>,
    storage_manager: Option<Arc<RwLock<StorageManager>>>,
}

impl IntegrationBuilder {
    /// Create a new integration builder
    pub fn new() -> Self {
        Self {
            config: None,
            network_orchestrator: None,
            storage_manager: None,
        }
    }

    /// Set integration configuration
    pub fn with_config(mut self, config: IntegrationConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set network orchestrator
    pub fn with_network_orchestrator(
        mut self,
        orchestrator: Arc<RwLock<NetworkOrchestrator>>,
    ) -> Self {
        self.network_orchestrator = Some(orchestrator);
        self
    }

    /// Set storage manager
    pub fn with_storage_manager(
        mut self,
        storage: Arc<RwLock<StorageManager>>,
    ) -> Self {
        self.storage_manager = Some(storage);
        self
    }

    /// Build the network-storage bridge
    pub async fn build(self) -> Result<NetworkStorageBridge, IntegrationError> {
        let config = self.config.unwrap_or_default();
        let network_orchestrator = self.network_orchestrator
            .ok_or_else(|| IntegrationError::Configuration("Network orchestrator not provided".to_string()))?;
        let storage_manager = self.storage_manager
            .ok_or_else(|| IntegrationError::Configuration("Storage manager not provided".to_string()))?;

        NetworkStorageBridge::new(network_orchestrator, storage_manager, config).await
    }
}

impl Default for IntegrationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize complete integration system
pub async fn init_integration(
    network_orchestrator: Arc<RwLock<NetworkOrchestrator>>,
    storage_manager: Arc<RwLock<StorageManager>>,
) -> Result<NetworkStorageBridge, IntegrationError> {
    info!("Initializing Network-Storage Integration");

    let config = IntegrationConfig::default();
    
    let bridge = IntegrationBuilder::new()
        .with_config(config)
        .with_network_orchestrator(network_orchestrator)
        .with_storage_manager(storage_manager)
        .build()
        .await?;

    info!("Network-Storage Integration initialized successfully");
    Ok(bridge)
}

/// Integration health check
pub async fn health_check(
    bridge: &NetworkStorageBridge,
) -> Result<IntegrationHealthReport, IntegrationError> {
    let stats = bridge.get_stats().await;
    let sync_stats = bridge.get_sync_stats().await;
    
    let consistency_report = bridge.perform_consistency_check().await?;
    
    let health_report = IntegrationHealthReport {
        overall_health: calculate_overall_health(&stats, &sync_stats, &consistency_report),
        integration_stats: stats,
        sync_stats,
        consistency_report,
        recommendations: generate_health_recommendations(&stats, &sync_stats),
    };

    Ok(health_report)
}

/// Integration health report
#[derive(Debug, Clone)]
pub struct IntegrationHealthReport {
    pub overall_health: HealthStatus,
    pub integration_stats: IntegrationStats,
    pub sync_stats: SyncStats,
    pub consistency_report: ConsistencyReport,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Offline,
}

fn calculate_overall_health(
    integration_stats: &IntegrationStats,
    sync_stats: &SyncStats,
    consistency_report: &ConsistencyReport,
) -> HealthStatus {
    let mut score = 100.0;

    // Penalize for failed operations
    if sync_stats.total_sync_requests > 0 {
        let failure_rate = sync_stats.failed_syncs as f64 / sync_stats.total_sync_requests as f64;
        score -= failure_rate * 30.0;
    }

    // Penalize for consistency issues
    if !consistency_report.overall_consistent {
        score -= 40.0;
    }

    // Penalize for low data consistency rate
    if integration_stats.data_consistency_rate < 0.95 {
        score -= (0.95 - integration_stats.data_consistency_rate) * 20.0;
    }

    match score {
        s if s >= 90.0 => HealthStatus::Healthy,
        s if s >= 70.0 => HealthStatus::Warning,
        s if s >= 30.0 => HealthStatus::Critical,
        _ => HealthStatus::Offline,
    }
}

fn generate_health_recommendations(
    integration_stats: &IntegrationStats,
    sync_stats: &SyncStats,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    if sync_stats.total_sync_requests > 0 {
        let failure_rate = sync_stats.failed_syncs as f64 / sync_stats.total_sync_requests as f64;
        if failure_rate > 0.1 {
            recommendations.push("High sync failure rate detected. Check network connectivity and storage health.".to_string());
        }
    }

    if integration_stats.data_consistency_rate < 0.95 {
        recommendations.push("Data consistency rate is below optimal. Consider increasing consistency check frequency.".to_string());
    }

    if sync_stats.sync_queue_size > 100 {
        recommendations.push("Large sync queue detected. Consider increasing concurrent sync limit or optimizing sync performance.".to_string());
    }

    if integration_stats.conflicts_detected > 0 && integration_stats.conflicts_resolved == 0 {
        recommendations.push("Unresolved conflicts detected. Review conflict resolution strategies.".to_string());
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_builder() {
        let builder = IntegrationBuilder::new();
        assert!(builder.config.is_none());
        assert!(builder.network_orchestrator.is_none());
        assert!(builder.storage_manager.is_none());
    }

    #[test]
    fn test_health_status_calculation() {
        let integration_stats = IntegrationStats {
            data_consistency_rate: 0.98,
            ..Default::default()
        };
        
        let sync_stats = SyncStats {
            total_sync_requests: 100,
            failed_syncs: 5,
            ..Default::default()
        };
        
        let consistency_report = ConsistencyReport {
            overall_consistent: true,
            network_consistent: true,
            storage_consistent: true,
            check_duration: std::time::Duration::from_secs(1),
            checked_at: std::time::SystemTime::now(),
            issues_found: Vec::new(),
            recommendations: Vec::new(),
        };
        
        let health = calculate_overall_health(&integration_stats, &sync_stats, &consistency_report);
        assert_eq!(health, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_recommendations() {
        let integration_stats = IntegrationStats {
            data_consistency_rate: 0.85,
            conflicts_detected: 10,
            conflicts_resolved: 8,
            ..Default::default()
        };
        
        let sync_stats = SyncStats {
            total_sync_requests: 100,
            failed_syncs: 20,
            sync_queue_size: 150,
            ..Default::default()
        };
        
        let recommendations = generate_health_recommendations(&integration_stats, &sync_stats);
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("High sync failure rate")));
        assert!(recommendations.iter().any(|r| r.contains("Data consistency rate")));
        assert!(recommendations.iter().any(|r| r.contains("Large sync queue")));
    }

    #[test]
    fn test_integration_config_defaults() {
        let config = IntegrationConfig::default();
        assert!(config.enable_realtime_sync);
        assert!(config.enable_distributed_backup);
        assert_eq!(config.max_sync_batch_size, 1000);
        assert_eq!(config.max_concurrent_syncs, 10);
    }
}
