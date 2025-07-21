//! Real-time Synchronization Coordinator
//!
//! Manages real-time synchronization between network and storage layers,
//! ensuring data consistency across distributed nodes and efficient
//! conflict resolution.

use super::network_storage::{
    IntegrationError, SyncRequest, SyncOperation, SyncStatus, SyncEventType,
    SyncPriority, SyncStats, IntegrationConfig,
};
use crate::network::NetworkError;
use crate::storage::DatabaseError;
use libp2p::PeerId;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{RwLock, mpsc, oneshot, Semaphore};
use tokio::time::{interval, timeout, sleep};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Advanced synchronization coordinator
pub struct SyncCoordinator {
    /// Active synchronization operations
    active_syncs: HashMap<String, SyncOperation>,
    /// Priority queue for pending sync requests
    sync_queue: BTreeMap<SyncPriority, VecDeque<SyncRequest>>,
    /// Sync operation results
    sync_results: HashMap<String, SyncResult>,
    /// Configuration
    config: IntegrationConfig,
    /// Statistics
    stats: SyncStats,
    /// Concurrency control
    sync_semaphore: Arc<Semaphore>,
    /// Event channels
    sync_events_tx: mpsc::UnboundedSender<SyncEvent>,
    sync_events_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<SyncEvent>>>>,
    /// Background task handles
    task_handles: Vec<tokio::task::JoinHandle<()>>,
}

/// Synchronization event for internal coordination
#[derive(Debug, Clone)]
pub enum SyncEvent {
    SyncStarted {
        sync_id: String,
        sync_type: SyncEventType,
        peer_id: Option<PeerId>,
    },
    SyncProgress {
        sync_id: String,
        progress: f64,
        bytes_transferred: u64,
    },
    SyncCompleted {
        sync_id: String,
        result: SyncResult,
    },
    SyncFailed {
        sync_id: String,
        error: String,
    },
    SyncCancelled {
        sync_id: String,
        reason: String,
    },
}

/// Result of a synchronization operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub sync_id: String,
    pub sync_type: SyncEventType,
    pub status: SyncStatus,
    pub started_at: SystemTime,
    pub completed_at: Option<SystemTime>,
    pub duration: Option<Duration>,
    pub bytes_transferred: u64,
    pub items_synced: u64,
    pub errors_encountered: Vec<String>,
    pub peer_id: Option<String>,
}

/// Sync operation metadata for tracking
#[derive(Debug, Clone)]
pub struct SyncMetadata {
    pub sync_id: String,
    pub sync_type: SyncEventType,
    pub priority: SyncPriority,
    pub estimated_size: Option<u64>,
    pub estimated_duration: Option<Duration>,
    pub dependencies: Vec<String>,
    pub retry_count: u32,
    pub max_retries: u32,
}

/// Data consistency manager
pub struct ConsistencyManager {
    /// Consistency checks configuration
    config: ConsistencyConfig,
    /// Active consistency checks
    active_checks: HashMap<String, ConsistencyCheck>,
    /// Consistency violation history
    violations: Vec<ConsistencyViolation>,
    /// Statistics
    stats: ConsistencyStats,
}

#[derive(Debug, Clone)]
pub struct ConsistencyConfig {
    /// Check interval for automatic consistency validation
    pub check_interval: Duration,
    /// Timeout for consistency checks
    pub check_timeout: Duration,
    /// Maximum acceptable data drift
    pub max_data_drift: Duration,
    /// Enable automatic repair of minor inconsistencies
    pub enable_auto_repair: bool,
    /// Consistency check batch size
    pub batch_size: usize,
}

impl Default for ConsistencyConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(300), // 5 minutes
            check_timeout: Duration::from_secs(60),
            max_data_drift: Duration::from_secs(30),
            enable_auto_repair: true,
            batch_size: 1000,
        }
    }
}

/// Individual consistency check operation
#[derive(Debug, Clone)]
pub struct ConsistencyCheck {
    pub check_id: String,
    pub check_type: ConsistencyCheckType,
    pub started_at: Instant,
    pub scope: ConsistencyScope,
    pub status: CheckStatus,
    pub findings: Vec<ConsistencyFinding>,
}

#[derive(Debug, Clone)]
pub enum ConsistencyCheckType {
    DataIntegrity,
    CrossReplication,
    IndexConsistency,
    NetworkState,
    StorageState,
    FullSystemCheck,
}

#[derive(Debug, Clone)]
pub enum ConsistencyScope {
    Local,
    Peer(PeerId),
    Network,
    Global,
}

#[derive(Debug, Clone)]
pub enum CheckStatus {
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

/// Finding from consistency check
#[derive(Debug, Clone)]
pub struct ConsistencyFinding {
    pub finding_type: FindingType,
    pub severity: FindingSeverity,
    pub description: String,
    pub affected_keys: Vec<String>,
    pub recommended_action: Option<String>,
    pub auto_repairable: bool,
}

#[derive(Debug, Clone)]
pub enum FindingType {
    DataMismatch,
    MissingData,
    ExtraData,
    IndexCorruption,
    TimestampDrift,
    HashMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FindingSeverity {
    Info = 1,
    Warning = 2,
    Error = 3,
    Critical = 4,
}

/// Consistency violation record
#[derive(Debug, Clone)]
pub struct ConsistencyViolation {
    pub violation_id: String,
    pub violation_type: ViolationType,
    pub detected_at: SystemTime,
    pub affected_peers: Vec<PeerId>,
    pub description: String,
    pub resolved: bool,
    pub resolution_action: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ViolationType {
    DataInconsistency,
    NetworkPartition,
    StateCorruption,
    SyncFailure,
    ConflictUnresolved,
}

/// Statistics for consistency management
#[derive(Debug, Clone, Default)]
pub struct ConsistencyStats {
    pub checks_performed: u64,
    pub violations_detected: u64,
    pub violations_resolved: u64,
    pub auto_repairs_performed: u64,
    pub average_check_duration: Duration,
    pub consistency_score: f64,
}

/// Distributed lock manager for coordination
pub struct DistributedLockManager {
    /// Active locks
    locks: HashMap<String, DistributedLock>,
    /// Lock request queue
    lock_queue: VecDeque<LockRequest>,
    /// Configuration
    config: LockConfig,
    /// Statistics
    stats: LockStats,
}

#[derive(Debug, Clone)]
pub struct DistributedLock {
    pub lock_id: String,
    pub resource_key: String,
    pub owner_peer: PeerId,
    pub acquired_at: Instant,
    pub expires_at: Instant,
    pub lock_type: LockType,
}

#[derive(Debug, Clone)]
pub enum LockType {
    Exclusive,
    Shared,
    ReadWrite,
}

#[derive(Debug, Clone)]
pub struct LockRequest {
    pub request_id: String,
    pub resource_key: String,
    pub requester_peer: PeerId,
    pub lock_type: LockType,
    pub timeout: Duration,
    pub created_at: Instant,
}

#[derive(Debug, Clone)]
pub struct LockConfig {
    pub default_timeout: Duration,
    pub max_lock_duration: Duration,
    pub enable_lock_stealing: bool,
    pub lock_renewal_interval: Duration,
}

impl Default for LockConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            max_lock_duration: Duration::from_secs(300),
            enable_lock_stealing: false,
            lock_renewal_interval: Duration::from_secs(10),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LockStats {
    pub locks_acquired: u64,
    pub locks_released: u64,
    pub lock_timeouts: u64,
    pub lock_conflicts: u64,
    pub average_lock_duration: Duration,
}

impl SyncCoordinator {
    /// Create a new synchronization coordinator
    pub fn new(config: IntegrationConfig) -> Self {
        let (sync_events_tx, sync_events_rx) = mpsc::unbounded_channel();
        let sync_semaphore = Arc::new(Semaphore::new(config.max_concurrent_syncs));

        Self {
            active_syncs: HashMap::new(),
            sync_queue: BTreeMap::new(),
            sync_results: HashMap::new(),
            config,
            stats: SyncStats::default(),
            sync_semaphore,
            sync_events_tx,
            sync_events_rx: Arc::new(RwLock::new(Some(sync_events_rx))),
            task_handles: Vec::new(),
        }
    }

    /// Start the synchronization coordinator
    pub async fn start(&mut self) -> Result<(), IntegrationError> {
        info!("Starting Synchronization Coordinator");

        // Start sync processor
        self.start_sync_processor().await?;
        
        // Start performance monitor
        self.start_performance_monitor().await?;
        
        // Start cleanup task
        self.start_cleanup_task().await?;

        info!("Synchronization Coordinator started successfully");
        Ok(())
    }

    /// Stop the synchronization coordinator
    pub async fn stop(&mut self) -> Result<(), IntegrationError> {
        info!("Stopping Synchronization Coordinator");

        // Cancel all background tasks
        for handle in self.task_handles.drain(..) {
            handle.abort();
        }

        // Cancel active syncs
        for (sync_id, _) in self.active_syncs.drain() {
            let _ = self.sync_events_tx.send(SyncEvent::SyncCancelled {
                sync_id,
                reason: "Coordinator shutdown".to_string(),
            });
        }

        info!("Synchronization Coordinator stopped");
        Ok(())
    }

    /// Add a synchronization request
    pub async fn add_sync_request(&mut self, request: SyncRequest) -> Result<String, IntegrationError> {
        let sync_id = Uuid::new_v4().to_string();
        
        debug!("Adding sync request: {} (type: {:?}, priority: {:?})", 
               sync_id, request.sync_type, request.priority);

        // Add to priority queue
        self.sync_queue
            .entry(request.priority.clone())
            .or_insert_with(VecDeque::new)
            .push_back(request);

        self.stats.total_sync_requests += 1;
        self.stats.sync_queue_size = self.get_total_queue_size();

        // Trigger sync processing
        self.process_sync_queue().await?;

        Ok(sync_id)
    }

    /// Process sync queue
    async fn process_sync_queue(&mut self) -> Result<(), IntegrationError> {
        while let Some((priority, mut queue)) = self.sync_queue.iter_mut()
            .rev() // Start with highest priority
            .find(|(_, queue)| !queue.is_empty())
            .map(|(p, q)| (p.clone(), q))
        {
            if self.active_syncs.len() >= self.config.max_concurrent_syncs {
                debug!("Max concurrent syncs reached, waiting");
                break;
            }

            if let Some(request) = queue.pop_front() {
                self.start_sync_operation(request).await?;
            }

            if queue.is_empty() {
                self.sync_queue.remove(&priority);
            }
        }

        self.stats.sync_queue_size = self.get_total_queue_size();
        Ok(())
    }

    /// Start a sync operation
    async fn start_sync_operation(&mut self, request: SyncRequest) -> Result<(), IntegrationError> {
        let sync_id = Uuid::new_v4().to_string();
        
        // Acquire semaphore permit
        let _permit = self.sync_semaphore.acquire().await
            .map_err(|e| IntegrationError::SyncError(format!("Failed to acquire sync permit: {}", e)))?;

        let sync_operation = SyncOperation {
            id: sync_id.clone(),
            sync_type: request.sync_type.clone(),
            started_at: Instant::now(),
            source_peer: request.source_peer,
            target_peer: request.target_peer,
            progress: 0.0,
            status: SyncStatus::InProgress,
        };

        self.active_syncs.insert(sync_id.clone(), sync_operation);
        self.stats.active_sync_count = self.active_syncs.len();

        // Send start event
        let _ = self.sync_events_tx.send(SyncEvent::SyncStarted {
            sync_id: sync_id.clone(),
            sync_type: request.sync_type,
            peer_id: request.target_peer,
        });

        debug!("Started sync operation: {}", sync_id);
        Ok(())
    }

    /// Update sync progress
    pub async fn update_sync_progress(
        &mut self,
        sync_id: &str,
        progress: f64,
        bytes_transferred: u64,
    ) -> Result<(), IntegrationError> {
        if let Some(sync_op) = self.active_syncs.get_mut(sync_id) {
            sync_op.progress = progress.clamp(0.0, 1.0);
            
            let _ = self.sync_events_tx.send(SyncEvent::SyncProgress {
                sync_id: sync_id.to_string(),
                progress,
                bytes_transferred,
            });
        }

        Ok(())
    }

    /// Complete a sync operation
    pub async fn complete_sync(
        &mut self,
        sync_id: &str,
        result: SyncResult,
    ) -> Result<(), IntegrationError> {
        if let Some(mut sync_op) = self.active_syncs.remove(sync_id) {
            sync_op.status = SyncStatus::Completed;
            sync_op.progress = 1.0;

            self.sync_results.insert(sync_id.to_string(), result.clone());
            self.stats.completed_syncs += 1;
            self.stats.active_sync_count = self.active_syncs.len();

            let _ = self.sync_events_tx.send(SyncEvent::SyncCompleted {
                sync_id: sync_id.to_string(),
                result,
            });

            debug!("Completed sync operation: {}", sync_id);
        }

        Ok(())
    }

    /// Fail a sync operation
    pub async fn fail_sync(
        &mut self,
        sync_id: &str,
        error: String,
    ) -> Result<(), IntegrationError> {
        if let Some(mut sync_op) = self.active_syncs.remove(sync_id) {
            sync_op.status = SyncStatus::Failed(error.clone());

            self.stats.failed_syncs += 1;
            self.stats.active_sync_count = self.active_syncs.len();

            let _ = self.sync_events_tx.send(SyncEvent::SyncFailed {
                sync_id: sync_id.to_string(),
                error,
            });

            warn!("Failed sync operation: {} - {}", sync_id, error);
        }

        Ok(())
    }

    /// Get synchronization statistics
    pub fn get_stats(&self) -> SyncStats {
        self.stats.clone()
    }

    /// Get active sync operations
    pub fn get_active_syncs(&self) -> &HashMap<String, SyncOperation> {
        &self.active_syncs
    }

    /// Get sync result
    pub fn get_sync_result(&self, sync_id: &str) -> Option<&SyncResult> {
        self.sync_results.get(sync_id)
    }

    // Private helper methods

    fn get_total_queue_size(&self) -> usize {
        self.sync_queue.values().map(|queue| queue.len()).sum()
    }

    async fn start_sync_processor(&mut self) -> Result<(), IntegrationError> {
        let events_rx = self.sync_events_rx.write().await.take()
            .ok_or_else(|| IntegrationError::Configuration("Sync events receiver already taken".to_string()))?;

        let handle = tokio::spawn(async move {
            Self::sync_event_processor(events_rx).await;
        });

        self.task_handles.push(handle);
        Ok(())
    }

    async fn sync_event_processor(mut events_rx: mpsc::UnboundedReceiver<SyncEvent>) {
        while let Some(event) = events_rx.recv().await {
            match event {
                SyncEvent::SyncStarted { sync_id, sync_type, peer_id } => {
                    debug!("Sync started: {} ({:?}) with peer {:?}", sync_id, sync_type, peer_id);
                }
                SyncEvent::SyncProgress { sync_id, progress, bytes_transferred } => {
                    debug!("Sync progress: {} - {:.1}% ({} bytes)", sync_id, progress * 100.0, bytes_transferred);
                }
                SyncEvent::SyncCompleted { sync_id, result } => {
                    info!("Sync completed: {} - {} items synced", sync_id, result.items_synced);
                }
                SyncEvent::SyncFailed { sync_id, error } => {
                    error!("Sync failed: {} - {}", sync_id, error);
                }
                SyncEvent::SyncCancelled { sync_id, reason } => {
                    warn!("Sync cancelled: {} - {}", sync_id, reason);
                }
            }
        }
    }

    async fn start_performance_monitor(&mut self) -> Result<(), IntegrationError> {
        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                // Performance monitoring logic would go here
                debug!("Sync coordinator performance check");
            }
        });

        self.task_handles.push(handle);
        Ok(())
    }

    async fn start_cleanup_task(&mut self) -> Result<(), IntegrationError> {
        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300)); // 5 minutes
            
            loop {
                interval.tick().await;
                // Cleanup logic would go here
                debug!("Sync coordinator cleanup");
            }
        });

        self.task_handles.push(handle);
        Ok(())
    }
}

impl ConsistencyManager {
    /// Create a new consistency manager
    pub fn new(config: ConsistencyConfig) -> Self {
        Self {
            config,
            active_checks: HashMap::new(),
            violations: Vec::new(),
            stats: ConsistencyStats::default(),
        }
    }

    /// Start a consistency check
    pub async fn start_consistency_check(
        &mut self,
        check_type: ConsistencyCheckType,
        scope: ConsistencyScope,
    ) -> Result<String, IntegrationError> {
        let check_id = Uuid::new_v4().to_string();
        
        let check = ConsistencyCheck {
            check_id: check_id.clone(),
            check_type,
            started_at: Instant::now(),
            scope,
            status: CheckStatus::Running,
            findings: Vec::new(),
        };

        self.active_checks.insert(check_id.clone(), check);
        self.stats.checks_performed += 1;

        debug!("Started consistency check: {} ({:?})", check_id, check_type);
        Ok(check_id)
    }

    /// Complete a consistency check
    pub async fn complete_consistency_check(
        &mut self,
        check_id: &str,
        findings: Vec<ConsistencyFinding>,
    ) -> Result<(), IntegrationError> {
        if let Some(check) = self.active_checks.get_mut(check_id) {
            check.status = CheckStatus::Completed;
            check.findings = findings.clone();

            // Check for violations
            for finding in &findings {
                if finding.severity >= FindingSeverity::Error {
                    self.record_violation(finding).await?;
                }
            }

            debug!("Completed consistency check: {} with {} findings", check_id, check.findings.len());
        }

        Ok(())
    }

    /// Record a consistency violation
    async fn record_violation(&mut self, finding: &ConsistencyFinding) -> Result<(), IntegrationError> {
        let violation = ConsistencyViolation {
            violation_id: Uuid::new_v4().to_string(),
            violation_type: match finding.finding_type {
                FindingType::DataMismatch => ViolationType::DataInconsistency,
                FindingType::HashMismatch => ViolationType::StateCorruption,
                _ => ViolationType::DataInconsistency,
            },
            detected_at: SystemTime::now(),
            affected_peers: Vec::new(), // Would be populated with actual peer data
            description: finding.description.clone(),
            resolved: false,
            resolution_action: finding.recommended_action.clone(),
        };

        self.violations.push(violation);
        self.stats.violations_detected += 1;

        warn!("Consistency violation recorded: {}", finding.description);
        Ok(())
    }

    /// Get consistency statistics
    pub fn get_stats(&self) -> &ConsistencyStats {
        &self.stats
    }
}

impl DistributedLockManager {
    /// Create a new distributed lock manager
    pub fn new(config: LockConfig) -> Self {
        Self {
            locks: HashMap::new(),
            lock_queue: VecDeque::new(),
            config,
            stats: LockStats::default(),
        }
    }

    /// Acquire a distributed lock
    pub async fn acquire_lock(
        &mut self,
        resource_key: String,
        requester_peer: PeerId,
        lock_type: LockType,
        timeout: Option<Duration>,
    ) -> Result<String, IntegrationError> {
        let lock_id = Uuid::new_v4().to_string();
        let timeout = timeout.unwrap_or(self.config.default_timeout);

        // Check if resource is already locked
        if self.is_resource_locked(&resource_key, &lock_type) {
            // Add to queue
            let request = LockRequest {
                request_id: lock_id.clone(),
                resource_key,
                requester_peer,
                lock_type,
                timeout,
                created_at: Instant::now(),
            };
            
            self.lock_queue.push_back(request);
            self.stats.lock_conflicts += 1;
            
            return Err(IntegrationError::SyncError("Resource locked".to_string()));
        }

        // Grant lock immediately
        let lock = DistributedLock {
            lock_id: lock_id.clone(),
            resource_key,
            owner_peer: requester_peer,
            acquired_at: Instant::now(),
            expires_at: Instant::now() + timeout,
            lock_type,
        };

        self.locks.insert(lock_id.clone(), lock);
        self.stats.locks_acquired += 1;

        debug!("Acquired distributed lock: {}", lock_id);
        Ok(lock_id)
    }

    /// Release a distributed lock
    pub async fn release_lock(&mut self, lock_id: &str) -> Result<(), IntegrationError> {
        if let Some(lock) = self.locks.remove(lock_id) {
            self.stats.locks_released += 1;
            
            // Process queue for this resource
            self.process_lock_queue(&lock.resource_key).await?;
            
            debug!("Released distributed lock: {}", lock_id);
        }

        Ok(())
    }

    /// Check if resource is locked
    fn is_resource_locked(&self, resource_key: &str, lock_type: &LockType) -> bool {
        self.locks.values().any(|lock| {
            lock.resource_key == resource_key && 
            self.conflicts_with_lock_type(&lock.lock_type, lock_type)
        })
    }

    /// Check if lock types conflict
    fn conflicts_with_lock_type(&self, existing: &LockType, requested: &LockType) -> bool {
        match (existing, requested) {
            (LockType::Exclusive, _) => true,
            (_, LockType::Exclusive) => true,
            (LockType::ReadWrite, LockType::ReadWrite) => true,
            _ => false,
        }
    }

    /// Process lock queue for a resource
    async fn process_lock_queue(&mut self, resource_key: &str) -> Result<(), IntegrationError> {
        // Find and grant next compatible lock in queue
        let mut i = 0;
        while i < self.lock_queue.len() {
            if self.lock_queue[i].resource_key == resource_key {
                let request = self.lock_queue.remove(i).unwrap();
                
                // Try to grant the lock
                if !self.is_resource_locked(&request.resource_key, &request.lock_type) {
                    let lock = DistributedLock {
                        lock_id: request.request_id.clone(),
                        resource_key: request.resource_key,
                        owner_peer: request.requester_peer,
                        acquired_at: Instant::now(),
                        expires_at: Instant::now() + request.timeout,
                        lock_type: request.lock_type,
                    };

                    self.locks.insert(request.request_id, lock);
                    self.stats.locks_acquired += 1;
                    break;
                }
            } else {
                i += 1;
            }
        }

        Ok(())
    }

    /// Get lock statistics
    pub fn get_stats(&self) -> &LockStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_coordinator_creation() {
        let config = IntegrationConfig::default();
        let coordinator = SyncCoordinator::new(config);
        
        assert_eq!(coordinator.active_syncs.len(), 0);
        assert_eq!(coordinator.sync_queue.len(), 0);
    }

    #[tokio::test]
    async fn test_consistency_manager_creation() {
        let config = ConsistencyConfig::default();
        let manager = ConsistencyManager::new(config);
        
        assert_eq!(manager.active_checks.len(), 0);
        assert_eq!(manager.violations.len(), 0);
    }

    #[tokio::test]
    async fn test_distributed_lock_manager() {
        let config = LockConfig::default();
        let mut manager = DistributedLockManager::new(config);
        
        let peer_id = PeerId::random();
        let resource = "test_resource".to_string();
        
        let lock_id = manager.acquire_lock(
            resource.clone(),
            peer_id,
            LockType::Exclusive,
            None,
        ).await;
        
        assert!(lock_id.is_ok());
        
        // Try to acquire same resource again
        let result = manager.acquire_lock(
            resource,
            peer_id,
            LockType::Shared,
            None,
        ).await;
        
        assert!(result.is_err());
    }

    #[test]
    fn test_finding_severity_ordering() {
        let mut severities = vec![
            FindingSeverity::Critical,
            FindingSeverity::Info,
            FindingSeverity::Error,
            FindingSeverity::Warning,
        ];
        
        severities.sort();
        
        assert_eq!(severities, vec![
            FindingSeverity::Info,
            FindingSeverity::Warning,
            FindingSeverity::Error,
            FindingSeverity::Critical,
        ]);
    }

    #[test]
    fn test_consistency_config_defaults() {
        let config = ConsistencyConfig::default();
        assert_eq!(config.check_interval, Duration::from_secs(300));
        assert_eq!(config.batch_size, 1000);
        assert!(config.enable_auto_repair);
    }
}
