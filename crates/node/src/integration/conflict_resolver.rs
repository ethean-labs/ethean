//! Advanced Conflict Resolution System
//!
//! Provides sophisticated conflict detection and resolution strategies
//! for distributed data consistency in network-storage integration.

use super::network_storage::{
    IntegrationError, ConflictInfo, ConflictResolution, ConflictStrategy,
    ConflictType, ConflictVersion, ConflictPriority, ResolutionType,
    IntegrationConfig,
};
use libp2p::PeerId;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, mpsc, oneshot};
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use sha2::Sha256;

/// Wrapper for PeerId to enable serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializablePeerId {
    #[serde(with = "peer_id_serde")]
    pub peer_id: PeerId,
}

mod peer_id_serde {
    use super::*;
    use serde::{Serializer, Deserializer};

    pub fn serialize<S>(peer_id: &PeerId, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&peer_id.to_bytes())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PeerId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        PeerId::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}

/// Advanced conflict resolver with multiple resolution strategies
pub struct ConflictResolver {
    /// Available resolution strategies
    strategies: HashMap<String, Box<dyn ConflictStrategy + Send + Sync>>,
    /// Active conflicts being processed
    active_conflicts: HashMap<String, ConflictInfo>,
    /// Conflict resolution queue
    conflict_queue: BTreeMap<ConflictPriority, VecDeque<ConflictInfo>>,
    /// Resolution history for learning
    resolution_history: VecDeque<ConflictResolution>,
    /// Configuration
    config: IntegrationConfig,
    /// Statistics
    stats: ConflictStats,
    /// Event channels
    conflict_events_tx: mpsc::UnboundedSender<ConflictEvent>,
    conflict_events_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<ConflictEvent>>>>,
    /// Background task handles
    task_handles: Vec<tokio::task::JoinHandle<()>>,
}

/// Events related to conflict resolution
#[derive(Debug, Clone)]
pub enum ConflictEvent {
    ConflictDetected {
        conflict_id: String,
        conflict_type: ConflictType,
        priority: ConflictPriority,
    },
    ConflictResolutionStarted {
        conflict_id: String,
        strategy: String,
    },
    ConflictResolved {
        conflict_id: String,
        resolution: ConflictResolution,
    },
    ConflictResolutionFailed {
        conflict_id: String,
        error: String,
        retry_count: u32,
    },
    ConflictEscalated {
        conflict_id: String,
        original_priority: ConflictPriority,
        new_priority: ConflictPriority,
    },
}

/// Statistics for conflict resolution
#[derive(Debug, Clone, Default)]
pub struct ConflictStats {
    pub conflicts_detected: u64,
    pub conflicts_resolved: u64,
    pub conflicts_failed: u64,
    pub average_resolution_time: Duration,
    pub strategy_success_rates: HashMap<String, f64>,
    pub conflict_types_distribution: HashMap<String, u64>,
    pub escalated_conflicts: u64,
    pub manual_interventions: u64,
}

/// Timestamp-based conflict resolution strategy
pub struct TimestampStrategy {
    pub name: String,
    pub prefer_latest: bool,
    pub max_timestamp_drift: Duration,
}

/// Vector clock-based conflict resolution strategy
pub struct VectorClockStrategy {
    pub name: String,
    pub vector_clocks: HashMap<PeerId, VectorClock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorClock {
    pub clock: HashMap<String, u64>,
    pub peer_id: SerializablePeerId,
    pub last_updated: SystemTime,
}

/// Hash-based conflict resolution strategy
pub struct HashStrategy {
    pub name: String,
    pub hash_algorithm: HashAlgorithm,
    pub verification_threshold: f64,
}

#[derive(Debug, Clone)]
pub enum HashAlgorithm {
    Sha256,
    Sha512,
    Blake3,
}

/// Consensus-based conflict resolution strategy
pub struct ConsensusStrategy {
    pub name: String,
    pub min_consensus_ratio: f64,
    pub timeout: Duration,
    pub Byzantine_tolerance: bool,
}

/// Content-aware merge strategy
pub struct ContentMergeStrategy {
    pub name: String,
    pub merge_algorithms: HashMap<String, MergeAlgorithm>,
}

#[derive(Debug, Clone)]
pub enum MergeAlgorithm {
    ThreeWayMerge,
    OperationalTransform,
    ConflictFreeReplicated,
    LastWriterWins,
    FirstWriterWins,
}

/// Advanced conflict detection system
pub struct ConflictDetector {
    /// Detection rules
    detection_rules: Vec<DetectionRule>,
    /// Monitoring active operations
    monitored_operations: HashMap<String, MonitoredOperation>,
    /// Detection statistics
    stats: DetectionStats,
}

#[derive(Debug, Clone)]
pub struct DetectionRule {
    pub rule_id: String,
    pub rule_type: DetectionRuleType,
    pub condition: DetectionCondition,
    pub priority: ConflictPriority,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum DetectionRuleType {
    TimestampDrift,
    HashMismatch,
    VersionConflict,
    ConcurrentWrite,
    NetworkPartition,
    StateInconsistency,
}

#[derive(Debug, Clone)]
pub struct DetectionCondition {
    pub threshold: f64,
    pub time_window: Duration,
    pub peer_count_threshold: usize,
    pub data_size_threshold: usize,
}

#[derive(Debug, Clone)]
pub struct MonitoredOperation {
    pub operation_id: String,
    pub operation_type: String,
    pub started_at: Instant,
    pub peers_involved: Vec<PeerId>,
    pub data_keys: Vec<String>,
    pub checkpoints: Vec<OperationCheckpoint>,
}

#[derive(Debug, Clone)]
pub struct OperationCheckpoint {
    pub timestamp: Instant,
    pub data_hash: String,
    pub peer_id: PeerId,
    pub operation_state: String,
}

#[derive(Debug, Clone, Default)]
pub struct DetectionStats {
    pub rules_evaluated: u64,
    pub conflicts_detected: u64,
    pub false_positives: u64,
    pub detection_latency: Duration,
    pub rule_performance: HashMap<String, RulePerformance>,
}

#[derive(Debug, Clone, Default)]
pub struct RulePerformance {
    pub evaluations: u64,
    pub detections: u64,
    pub false_positives: u64,
    pub average_evaluation_time: Duration,
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new(config: IntegrationConfig) -> Self {
        let (conflict_events_tx, conflict_events_rx) = mpsc::unbounded_channel();
        
        let mut resolver = Self {
            strategies: HashMap::new(),
            active_conflicts: HashMap::new(),
            conflict_queue: BTreeMap::new(),
            resolution_history: VecDeque::new(),
            config,
            stats: ConflictStats::default(),
            conflict_events_tx,
            conflict_events_rx: Arc::new(RwLock::new(Some(conflict_events_rx))),
            task_handles: Vec::new(),
        };

        // Register default strategies
        resolver.register_default_strategies();
        
        resolver
    }

    /// Start the conflict resolver
    pub async fn start(&mut self) -> Result<(), IntegrationError> {
        info!("Starting Conflict Resolver");

        // Start conflict processor
        self.start_conflict_processor().await?;
        
        // Start resolution monitor
        self.start_resolution_monitor().await?;
        
        // Start strategy optimization
        self.start_strategy_optimization().await?;

        info!("Conflict Resolver started successfully");
        Ok(())
    }

    /// Stop the conflict resolver
    pub async fn stop(&mut self) -> Result<(), IntegrationError> {
        info!("Stopping Conflict Resolver");

        // Cancel all background tasks
        for handle in self.task_handles.drain(..) {
            handle.abort();
        }

        info!("Conflict Resolver stopped");
        Ok(())
    }

    /// Detect and register a conflict
    pub async fn detect_conflict(
        &mut self,
        key: String,
        versions: Vec<ConflictVersion>,
        conflict_type: ConflictType,
        priority: ConflictPriority,
    ) -> Result<String, IntegrationError> {
        let conflict_id = Uuid::new_v4().to_string();
        
        let conflict = ConflictInfo {
            id: conflict_id.clone(),
            conflict_type: conflict_type.clone(),
            key,
            versions,
            detected_at: Instant::now(),
            priority: priority.clone(),
        };

        debug!("Detected conflict: {} (type: {:?}, priority: {:?})", 
               conflict_id, conflict_type, priority);

        // Add to queue
        self.conflict_queue
            .entry(priority.clone())
            .or_insert_with(VecDeque::new)
            .push_back(conflict);

        self.stats.conflicts_detected += 1;
        self.stats.conflict_types_distribution
            .entry(format!("{:?}", conflict_type))
            .and_modify(|e| *e += 1)
            .or_insert(1);

        // Send event
        let _ = self.conflict_events_tx.send(ConflictEvent::ConflictDetected {
            conflict_id: conflict_id.clone(),
            conflict_type,
            priority,
        });

        // Process conflicts
        self.process_conflict_queue().await?;

        Ok(conflict_id)
    }

    /// Process conflict queue
    async fn process_conflict_queue(&mut self) -> Result<(), IntegrationError> {
        while let Some((priority, mut queue)) = self.conflict_queue.iter_mut()
            .rev() // Start with highest priority
            .find(|(_, queue)| !queue.is_empty())
            .map(|(p, q)| (p.clone(), q))
        {
            if let Some(conflict) = queue.pop_front() {
                self.resolve_conflict(conflict).await?;
            }

            if queue.is_empty() {
                self.conflict_queue.remove(&priority);
            }
        }

        Ok(())
    }

    /// Resolve a specific conflict
    async fn resolve_conflict(&mut self, conflict: ConflictInfo) -> Result<(), IntegrationError> {
        let conflict_id = conflict.id.clone();
        let strategy_name = self.select_strategy(&conflict)?;
        
        debug!("Resolving conflict {} with strategy: {}", conflict_id, strategy_name);

        // Add to active conflicts
        self.active_conflicts.insert(conflict_id.clone(), conflict.clone());

        // Send start event
        let _ = self.conflict_events_tx.send(ConflictEvent::ConflictResolutionStarted {
            conflict_id: conflict_id.clone(),
            strategy: strategy_name.clone(),
        });

        // Execute resolution with timeout
        let resolution_timeout = self.config.conflict_resolution_timeout;
        let resolution_result = timeout(
            resolution_timeout,
            self.execute_resolution(&conflict, &strategy_name),
        ).await;

        match resolution_result {
            Ok(Ok(resolution)) => {
                self.complete_resolution(conflict_id, resolution).await?;
            }
            Ok(Err(e)) => {
                self.fail_resolution(conflict_id, e.to_string()).await?;
            }
            Err(_) => {
                self.fail_resolution(conflict_id, "Resolution timeout".to_string()).await?;
            }
        }

        Ok(())
    }

    /// Select the best strategy for a conflict
    fn select_strategy(&self, conflict: &ConflictInfo) -> Result<String, IntegrationError> {
        // Strategy selection based on conflict type and history
        let strategy_name = match &conflict.conflict_type {
            ConflictType::DataInconsistency => "timestamp",
            ConflictType::ConcurrentUpdate => "vector_clock",
            ConflictType::NetworkPartition => "consensus",
            ConflictType::VersionMismatch => "hash",
            ConflictType::StateCorruption => "content_merge",
        };

        if !self.strategies.contains_key(strategy_name) {
            return Err(IntegrationError::ConflictResolution(
                format!("Strategy not found: {}", strategy_name)
            ));
        }

        Ok(strategy_name.to_string())
    }

    /// Execute conflict resolution with selected strategy
    async fn execute_resolution(
        &self,
        conflict: &ConflictInfo,
        strategy_name: &str,
    ) -> Result<ConflictResolution, IntegrationError> {
        let strategy = self.strategies.get(strategy_name)
            .ok_or_else(|| IntegrationError::ConflictResolution(
                format!("Strategy not found: {}", strategy_name)
            ))?;

        strategy.resolve_conflict(conflict)
    }

    /// Complete a successful resolution
    async fn complete_resolution(
        &mut self,
        conflict_id: String,
        resolution: ConflictResolution,
    ) -> Result<(), IntegrationError> {
        // Remove from active conflicts
        self.active_conflicts.remove(&conflict_id);
        
        // Add to history
        self.resolution_history.push_back(resolution.clone());
        if self.resolution_history.len() > 1000 {
            self.resolution_history.pop_front();
        }

        // Update statistics
        self.stats.conflicts_resolved += 1;
        self.update_strategy_success_rate(&resolution.strategy_used, true);

        // Send event
        let _ = self.conflict_events_tx.send(ConflictEvent::ConflictResolved {
            conflict_id: conflict_id.clone(),
            resolution,
        });

        debug!("Successfully resolved conflict: {}", conflict_id);
        Ok(())
    }

    /// Handle failed resolution
    async fn fail_resolution(
        &mut self,
        conflict_id: String,
        error: String,
    ) -> Result<(), IntegrationError> {
        // Remove from active conflicts
        if let Some(conflict) = self.active_conflicts.remove(&conflict_id) {
            // Check if we should escalate or retry
            if conflict.priority < ConflictPriority::Critical {
                // Escalate priority and retry
                let new_priority = self.escalate_priority(conflict.priority);
                let escalated_conflict = ConflictInfo {
                    priority: new_priority.clone(),
                    ..conflict
                };

                self.conflict_queue
                    .entry(new_priority.clone())
                    .or_insert_with(VecDeque::new)
                    .push_back(escalated_conflict);

                self.stats.escalated_conflicts += 1;

                let _ = self.conflict_events_tx.send(ConflictEvent::ConflictEscalated {
                    conflict_id: conflict_id.clone(),
                    original_priority: conflict.priority,
                    new_priority,
                });
            } else {
                // Mark as failed, might need manual intervention
                self.stats.conflicts_failed += 1;
                self.stats.manual_interventions += 1;
            }
        }

        // Send event
        let _ = self.conflict_events_tx.send(ConflictEvent::ConflictResolutionFailed {
            conflict_id,
            error,
            retry_count: 0,
        });

        Ok(())
    }

    /// Escalate conflict priority
    fn escalate_priority(&self, current: ConflictPriority) -> ConflictPriority {
        match current {
            ConflictPriority::Low => ConflictPriority::Normal,
            ConflictPriority::Normal => ConflictPriority::High,
            ConflictPriority::High => ConflictPriority::Critical,
            ConflictPriority::Critical => ConflictPriority::Critical,
        }
    }

    /// Update strategy success rate
    fn update_strategy_success_rate(&mut self, strategy_name: &str, success: bool) {
        let current_rate = self.stats.strategy_success_rates
            .get(strategy_name)
            .copied()
            .unwrap_or(0.0);
        
        // Simple exponential moving average
        let alpha = 0.1;
        let new_rate = if success {
            current_rate * (1.0 - alpha) + alpha
        } else {
            current_rate * (1.0 - alpha)
        };

        self.stats.strategy_success_rates.insert(strategy_name.to_string(), new_rate);
    }

    /// Register default resolution strategies
    fn register_default_strategies(&mut self) {
        // Timestamp strategy
        let timestamp_strategy = TimestampStrategy {
            name: "timestamp".to_string(),
            prefer_latest: true,
            max_timestamp_drift: Duration::from_secs(30),
        };
        self.strategies.insert("timestamp".to_string(), Box::new(timestamp_strategy));

        // Vector clock strategy
        let vector_clock_strategy = VectorClockStrategy {
            name: "vector_clock".to_string(),
            vector_clocks: HashMap::new(),
        };
        self.strategies.insert("vector_clock".to_string(), Box::new(vector_clock_strategy));

        // Hash strategy
        let hash_strategy = HashStrategy {
            name: "hash".to_string(),
            hash_algorithm: HashAlgorithm::Sha256,
            verification_threshold: 0.67,
        };
        self.strategies.insert("hash".to_string(), Box::new(hash_strategy));

        // Consensus strategy
        let consensus_strategy = ConsensusStrategy {
            name: "consensus".to_string(),
            min_consensus_ratio: 0.67,
            timeout: Duration::from_secs(30),
            Byzantine_tolerance: true,
        };
        self.strategies.insert("consensus".to_string(), Box::new(consensus_strategy));

        // Content merge strategy
        let content_merge_strategy = ContentMergeStrategy {
            name: "content_merge".to_string(),
            merge_algorithms: HashMap::new(),
        };
        self.strategies.insert("content_merge".to_string(), Box::new(content_merge_strategy));
    }

    /// Get conflict resolution statistics
    pub fn get_stats(&self) -> &ConflictStats {
        &self.stats
    }

    /// Get active conflicts
    pub fn get_active_conflicts(&self) -> &HashMap<String, ConflictInfo> {
        &self.active_conflicts
    }

    /// Get resolution history
    pub fn get_resolution_history(&self) -> &VecDeque<ConflictResolution> {
        &self.resolution_history
    }

    // Private helper methods

    async fn start_conflict_processor(&mut self) -> Result<(), IntegrationError> {
        let events_rx = self.conflict_events_rx.write().await.take()
            .ok_or_else(|| IntegrationError::Configuration("Conflict events receiver already taken".to_string()))?;

        let handle = tokio::spawn(async move {
            Self::conflict_event_processor(events_rx).await;
        });

        self.task_handles.push(handle);
        Ok(())
    }

    async fn conflict_event_processor(mut events_rx: mpsc::UnboundedReceiver<ConflictEvent>) {
        while let Some(event) = events_rx.recv().await {
            match event {
                ConflictEvent::ConflictDetected { conflict_id, conflict_type, priority } => {
                    debug!("Conflict detected: {} ({:?}, priority: {:?})", conflict_id, conflict_type, priority);
                }
                ConflictEvent::ConflictResolutionStarted { conflict_id, strategy } => {
                    debug!("Conflict resolution started: {} with strategy: {}", conflict_id, strategy);
                }
                ConflictEvent::ConflictResolved { conflict_id, resolution } => {
                    info!("Conflict resolved: {} (strategy: {})", conflict_id, resolution.strategy_used);
                }
                ConflictEvent::ConflictResolutionFailed { conflict_id, error, retry_count } => {
                    warn!("Conflict resolution failed: {} - {} (retry: {})", conflict_id, error, retry_count);
                }
                ConflictEvent::ConflictEscalated { conflict_id, original_priority, new_priority } => {
                    warn!("Conflict escalated: {} ({:?} -> {:?})", conflict_id, original_priority, new_priority);
                }
            }
        }
    }

    async fn start_resolution_monitor(&mut self) -> Result<(), IntegrationError> {
        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                debug!("Conflict resolution monitor check");
            }
        });

        self.task_handles.push(handle);
        Ok(())
    }

    async fn start_strategy_optimization(&mut self) -> Result<(), IntegrationError> {
        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300)); // 5 minutes
            
            loop {
                interval.tick().await;
                debug!("Strategy optimization check");
            }
        });

        self.task_handles.push(handle);
        Ok(())
    }
}

// Strategy implementations

impl ConflictStrategy for TimestampStrategy {
    fn resolve_conflict(&self, conflict: &ConflictInfo) -> Result<ConflictResolution, IntegrationError> {
        debug!("Resolving conflict with timestamp strategy: {}", self.name);

        let chosen_version = if self.prefer_latest {
            conflict.versions.iter()
                .max_by_key(|v| v.timestamp)
                .cloned()
        } else {
            conflict.versions.iter()
                .min_by_key(|v| v.timestamp)
                .cloned()
        };

        Ok(ConflictResolution {
            conflict_id: conflict.id.clone(),
            resolution_type: ResolutionType::ChooseVersion,
            chosen_version,
            merged_data: None,
            resolved_at: Instant::now(),
            strategy_used: self.name.clone(),
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl ConflictStrategy for VectorClockStrategy {
    fn resolve_conflict(&self, conflict: &ConflictInfo) -> Result<ConflictResolution, IntegrationError> {
        debug!("Resolving conflict with vector clock strategy: {}", self.name);

        // Simplified vector clock resolution
        let chosen_version = conflict.versions.first().cloned();

        Ok(ConflictResolution {
            conflict_id: conflict.id.clone(),
            resolution_type: ResolutionType::ChooseVersion,
            chosen_version,
            merged_data: None,
            resolved_at: Instant::now(),
            strategy_used: self.name.clone(),
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl ConflictStrategy for HashStrategy {
    fn resolve_conflict(&self, conflict: &ConflictInfo) -> Result<ConflictResolution, IntegrationError> {
        debug!("Resolving conflict with hash strategy: {}", self.name);

        // Find version with most common hash
        let mut hash_counts: HashMap<String, (usize, ConflictVersion)> = HashMap::new();
        
        for version in &conflict.versions {
            let count = hash_counts.get(&version.hash).map(|(c, _)| *c).unwrap_or(0);
            hash_counts.insert(version.hash.clone(), (count + 1, version.clone()));
        }

        let chosen_version = hash_counts.values()
            .max_by_key(|(count, _)| *count)
            .map(|(_, version)| version.clone());

        Ok(ConflictResolution {
            conflict_id: conflict.id.clone(),
            resolution_type: ResolutionType::ChooseVersion,
            chosen_version,
            merged_data: None,
            resolved_at: Instant::now(),
            strategy_used: self.name.clone(),
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl ConflictStrategy for ConsensusStrategy {
    fn resolve_conflict(&self, conflict: &ConflictInfo) -> Result<ConflictResolution, IntegrationError> {
        debug!("Resolving conflict with consensus strategy: {}", self.name);

        // Simplified consensus resolution
        let chosen_version = conflict.versions.first().cloned();

        Ok(ConflictResolution {
            conflict_id: conflict.id.clone(),
            resolution_type: ResolutionType::ChooseVersion,
            chosen_version,
            merged_data: None,
            resolved_at: Instant::now(),
            strategy_used: self.name.clone(),
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl ConflictStrategy for ContentMergeStrategy {
    fn resolve_conflict(&self, conflict: &ConflictInfo) -> Result<ConflictResolution, IntegrationError> {
        debug!("Resolving conflict with content merge strategy: {}", self.name);

        // Simplified merge resolution
        let merged_data = if conflict.versions.len() >= 2 {
            let mut combined = Vec::new();
            for version in &conflict.versions {
                combined.extend_from_slice(&version.data);
            }
            Some(combined)
        } else {
            None
        };

        Ok(ConflictResolution {
            conflict_id: conflict.id.clone(),
            resolution_type: ResolutionType::MergeVersions,
            chosen_version: None,
            merged_data,
            resolved_at: Instant::now(),
            strategy_used: self.name.clone(),
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_conflict_resolver_creation() {
        let config = IntegrationConfig::default();
        let resolver = ConflictResolver::new(config);
        
        assert_eq!(resolver.strategies.len(), 5); // Default strategies
        assert_eq!(resolver.active_conflicts.len(), 0);
    }

    #[tokio::test]
    async fn test_conflict_detection() {
        let config = IntegrationConfig::default();
        let mut resolver = ConflictResolver::new(config);
        
        let version1 = ConflictVersion {
            peer_id: Some(PeerId::random()),
            timestamp: SystemTime::now(),
            data: vec![1, 2, 3],
            hash: "hash1".to_string(),
            signature: None,
        };
        
        let version2 = ConflictVersion {
            peer_id: Some(PeerId::random()),
            timestamp: SystemTime::now(),
            data: vec![4, 5, 6],
            hash: "hash2".to_string(),
            signature: None,
        };
        
        let conflict_id = resolver.detect_conflict(
            "test_key".to_string(),
            vec![version1, version2],
            ConflictType::DataInconsistency,
            ConflictPriority::Normal,
        ).await;
        
        assert!(conflict_id.is_ok());
        assert_eq!(resolver.stats.conflicts_detected, 1);
    }

    #[test]
    fn test_timestamp_strategy() {
        let strategy = TimestampStrategy {
            name: "test_timestamp".to_string(),
            prefer_latest: true,
            max_timestamp_drift: Duration::from_secs(30),
        };
        
        let version1 = ConflictVersion {
            peer_id: Some(PeerId::random()),
            timestamp: SystemTime::UNIX_EPOCH + Duration::from_secs(100),
            data: vec![1, 2, 3],
            hash: "hash1".to_string(),
            signature: None,
        };
        
        let version2 = ConflictVersion {
            peer_id: Some(PeerId::random()),
            timestamp: SystemTime::UNIX_EPOCH + Duration::from_secs(200),
            data: vec![4, 5, 6],
            hash: "hash2".to_string(),
            signature: None,
        };
        
        let conflict = ConflictInfo {
            id: "test_conflict".to_string(),
            conflict_type: ConflictType::DataInconsistency,
            key: "test_key".to_string(),
            versions: vec![version1, version2.clone()],
            detected_at: Instant::now(),
            priority: ConflictPriority::Normal,
        };
        
        let resolution = strategy.resolve_conflict(&conflict).unwrap();
        assert_eq!(resolution.chosen_version.unwrap().timestamp, version2.timestamp);
    }

    #[test]
    fn test_conflict_priority_escalation() {
        let config = IntegrationConfig::default();
        let resolver = ConflictResolver::new(config);
        
        assert_eq!(resolver.escalate_priority(ConflictPriority::Low), ConflictPriority::Normal);
        assert_eq!(resolver.escalate_priority(ConflictPriority::Normal), ConflictPriority::High);
        assert_eq!(resolver.escalate_priority(ConflictPriority::High), ConflictPriority::Critical);
        assert_eq!(resolver.escalate_priority(ConflictPriority::Critical), ConflictPriority::Critical);
    }

    #[test]
    fn test_conflict_stats_defaults() {
        let stats = ConflictStats::default();
        assert_eq!(stats.conflicts_detected, 0);
        assert_eq!(stats.conflicts_resolved, 0);
        assert_eq!(stats.conflicts_failed, 0);
        assert_eq!(stats.escalated_conflicts, 0);
    }
}
