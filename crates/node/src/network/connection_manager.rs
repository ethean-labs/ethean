//! Advanced connection management for P2P networking
//!
//! Implements connection pool management, health monitoring, and automatic
//! recovery mechanisms for robust Ethereum Beacon Chain networking.

use crate::network::NetworkError;
use libp2p::{
    core::connection::ConnectionId,
    swarm::{ConnectionHandler, NetworkBehaviour},
    Multiaddr, PeerId,
};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

/// Connection pool manager for efficient peer connection management
pub struct ConnectionPool {
    /// Active connections indexed by peer ID
    active_connections: HashMap<PeerId, ConnectionInfo>,
    /// Pending connection attempts
    pending_connections: HashMap<PeerId, ConnectionAttempt>,
    /// Connection statistics
    stats: ConnectionStats,
    /// Configuration parameters
    config: ConnectionPoolConfig,
    /// Health monitor for connection quality assessment
    health_monitor: HealthMonitor,
    /// Recovery manager for failed connections
    recovery_manager: RecoveryManager,
    /// Load balancer for connection distribution
    load_balancer: LoadBalancer,
}

/// Detailed connection information
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub peer_id: PeerId,
    pub connection_id: ConnectionId,
    pub address: Multiaddr,
    pub established_at: Instant,
    pub last_activity: Instant,
    pub connection_type: ConnectionType,
    pub quality_score: f64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub latency: Option<Duration>,
    pub bandwidth_utilization: f64,
    pub error_count: u32,
    pub status: ConnectionStatus,
}

/// Connection attempt tracking
#[derive(Debug, Clone)]
pub struct ConnectionAttempt {
    pub peer_id: PeerId,
    pub target_address: Multiaddr,
    pub started_at: Instant,
    pub attempt_count: u32,
    pub last_error: Option<String>,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    Inbound,
    Outbound,
    Bootstrap,
    Peer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Establishing,
    Active,
    Degraded,
    Failing,
    Closed,
}

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Maximum connections per peer
    pub max_connections_per_peer: usize,
    /// Connection timeout duration
    pub connection_timeout: Duration,
    /// Connection idle timeout
    pub idle_timeout: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Minimum connection quality score
    pub min_quality_score: f64,
    /// Connection retry configuration
    pub retry_config: RetryConfig,
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub backoff_multiplier: f64,
}

/// Connection statistics
#[derive(Debug, Default, Clone)]
pub struct ConnectionStats {
    pub total_connections: u64,
    pub active_connections: u32,
    pub failed_connections: u64,
    pub dropped_connections: u64,
    pub average_connection_duration: Duration,
    pub bytes_transferred: u64,
    pub messages_processed: u64,
    pub connection_errors: u64,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            max_connections_per_peer: 2,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300), // 5 minutes
            health_check_interval: Duration::from_secs(30),
            min_quality_score: 0.5,
            retry_config: RetryConfig::default(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            backoff_multiplier: 2.0,
        }
    }
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(config: ConnectionPoolConfig) -> Self {
        Self {
            active_connections: HashMap::new(),
            pending_connections: HashMap::new(),
            stats: ConnectionStats::default(),
            health_monitor: HealthMonitor::new(config.health_check_interval),
            recovery_manager: RecoveryManager::new(config.retry_config.clone()),
            load_balancer: LoadBalancer::new(),
            config,
        }
    }

    /// Attempt to establish connection to a peer
    pub async fn connect_to_peer(
        &mut self,
        peer_id: PeerId,
        address: Multiaddr,
    ) -> Result<(), ConnectionError> {
        // Check if we already have a connection to this peer
        if self.active_connections.contains_key(&peer_id) {
            return Ok(());
        }

        // Check connection limits
        if self.active_connections.len() >= self.config.max_connections {
            return Err(ConnectionError::ConnectionLimitReached);
        }

        // Check if connection attempt is already in progress
        if self.pending_connections.contains_key(&peer_id) {
            return Err(ConnectionError::ConnectionInProgress);
        }

        // Create connection attempt
        let attempt = ConnectionAttempt {
            peer_id,
            target_address: address,
            started_at: Instant::now(),
            attempt_count: 1,
            last_error: None,
            timeout: self.config.connection_timeout,
        };

        self.pending_connections.insert(peer_id, attempt);
        info!("Initiating connection to peer {} at {}", peer_id, address);

        // In production, this would trigger actual connection establishment
        Ok(())
    }

    /// Handle successful connection establishment
    pub fn on_connection_established(
        &mut self,
        peer_id: PeerId,
        connection_id: ConnectionId,
        address: Multiaddr,
        connection_type: ConnectionType,
    ) {
        // Remove from pending connections
        self.pending_connections.remove(&peer_id);

        // Create connection info
        let connection_info = ConnectionInfo {
            peer_id,
            connection_id,
            address,
            established_at: Instant::now(),
            last_activity: Instant::now(),
            connection_type,
            quality_score: 1.0, // Start with perfect score
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
            latency: None,
            bandwidth_utilization: 0.0,
            error_count: 0,
            status: ConnectionStatus::Active,
        };

        self.active_connections.insert(peer_id, connection_info);
        self.stats.total_connections += 1;
        self.stats.active_connections += 1;

        // Register with health monitor
        self.health_monitor.register_connection(peer_id);

        info!("Connection established with peer {}", peer_id);
    }

    /// Handle connection failure
    pub fn on_connection_failed(
        &mut self,
        peer_id: PeerId,
        error: String,
    ) {
        if let Some(mut attempt) = self.pending_connections.remove(&peer_id) {
            attempt.last_error = Some(error.clone());
            attempt.attempt_count += 1;

            self.stats.failed_connections += 1;

            // Check if we should retry
            if attempt.attempt_count <= self.config.retry_config.max_attempts {
                // Schedule retry with exponential backoff
                self.recovery_manager.schedule_retry(attempt);
                warn!("Connection to peer {} failed: {}. Retrying...", peer_id, error);
            } else {
                error!("Connection to peer {} failed permanently: {}", peer_id, error);
            }
        }
    }

    /// Handle connection closed
    pub fn on_connection_closed(&mut self, peer_id: PeerId) {
        if let Some(connection) = self.active_connections.remove(&peer_id) {
            self.stats.active_connections -= 1;
            self.stats.dropped_connections += 1;

            // Update average connection duration
            let duration = connection.established_at.elapsed();
            self.update_average_duration(duration);

            // Unregister from health monitor
            self.health_monitor.unregister_connection(&peer_id);

            info!("Connection with peer {} closed", peer_id);
        }
    }

    /// Update connection statistics
    pub fn update_connection_stats(
        &mut self,
        peer_id: &PeerId,
        bytes_sent: u64,
        bytes_received: u64,
        messages_sent: u64,
        messages_received: u64,
    ) {
        if let Some(connection) = self.active_connections.get_mut(peer_id) {
            connection.bytes_sent += bytes_sent;
            connection.bytes_received += bytes_received;
            connection.messages_sent += messages_sent;
            connection.messages_received += messages_received;
            connection.last_activity = Instant::now();

            // Update global stats
            self.stats.bytes_transferred += bytes_sent + bytes_received;
            self.stats.messages_processed += messages_sent + messages_received;
        }
    }

    /// Update connection latency
    pub fn update_latency(&mut self, peer_id: &PeerId, latency: Duration) {
        if let Some(connection) = self.active_connections.get_mut(peer_id) {
            connection.latency = Some(latency);
            connection.last_activity = Instant::now();

            // Update quality score based on latency
            self.update_quality_score(peer_id);
        }
    }

    /// Get connection information for a peer
    pub fn get_connection_info(&self, peer_id: &PeerId) -> Option<&ConnectionInfo> {
        self.active_connections.get(peer_id)
    }

    /// Get all active connections
    pub fn get_active_connections(&self) -> &HashMap<PeerId, ConnectionInfo> {
        &self.active_connections
    }

    /// Get connection statistics
    pub fn get_stats(&self) -> &ConnectionStats {
        &self.stats
    }

    /// Get best connections for load balancing
    pub fn get_best_connections(&self, count: usize) -> Vec<PeerId> {
        self.load_balancer.select_best_connections(&self.active_connections, count)
    }

    /// Perform periodic maintenance
    pub async fn maintenance(&mut self) {
        // Check for idle connections
        self.cleanup_idle_connections().await;

        // Run health checks
        self.health_monitor.run_health_checks(&mut self.active_connections).await;

        // Process recovery queue
        self.recovery_manager.process_retries().await;

        // Update load balancer metrics
        self.load_balancer.update_metrics(&self.active_connections);
    }

    /// Clean up idle connections
    async fn cleanup_idle_connections(&mut self) {
        let idle_timeout = self.config.idle_timeout;
        let now = Instant::now();
        
        let idle_peers: Vec<PeerId> = self.active_connections
            .iter()
            .filter(|(_, conn)| now.duration_since(conn.last_activity) > idle_timeout)
            .map(|(peer_id, _)| *peer_id)
            .collect();

        for peer_id in idle_peers {
            debug!("Closing idle connection to peer {}", peer_id);
            self.on_connection_closed(peer_id);
        }
    }

    /// Update quality score for a connection
    fn update_quality_score(&mut self, peer_id: &PeerId) {
        if let Some(connection) = self.active_connections.get_mut(peer_id) {
            let mut score = 1.0;

            // Factor in latency
            if let Some(latency) = connection.latency {
                let latency_ms = latency.as_millis() as f64;
                score *= (1000.0 / (1000.0 + latency_ms)).max(0.1);
            }

            // Factor in error rate
            let total_messages = connection.messages_sent + connection.messages_received;
            if total_messages > 0 {
                let error_rate = connection.error_count as f64 / total_messages as f64;
                score *= (1.0 - error_rate).max(0.1);
            }

            connection.quality_score = score;
        }
    }

    /// Update average connection duration
    fn update_average_duration(&mut self, duration: Duration) {
        let current_avg = self.stats.average_connection_duration;
        let total_connections = self.stats.total_connections;

        if total_connections == 1 {
            self.stats.average_connection_duration = duration;
        } else {
            let weighted_avg = (current_avg.as_millis() * (total_connections - 1) as u128 
                + duration.as_millis()) / total_connections as u128;
            self.stats.average_connection_duration = Duration::from_millis(weighted_avg as u64);
        }
    }
}

/// Connection-related errors
#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("Connection limit reached")]
    ConnectionLimitReached,
    #[error("Connection already in progress")]
    ConnectionInProgress,
    #[error("Connection timeout")]
    Timeout,
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
}

/// Health monitoring for connections
pub struct HealthMonitor {
    check_interval: Duration,
    monitored_connections: HashMap<PeerId, HealthStatus>,
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub last_check: Instant,
    pub ping_latency: Option<Duration>,
    pub consecutive_failures: u32,
    pub health_score: f64,
}

impl HealthMonitor {
    pub fn new(check_interval: Duration) -> Self {
        Self {
            check_interval,
            monitored_connections: HashMap::new(),
        }
    }

    pub fn register_connection(&mut self, peer_id: PeerId) {
        let status = HealthStatus {
            last_check: Instant::now(),
            ping_latency: None,
            consecutive_failures: 0,
            health_score: 1.0,
        };
        self.monitored_connections.insert(peer_id, status);
    }

    pub fn unregister_connection(&mut self, peer_id: &PeerId) {
        self.monitored_connections.remove(peer_id);
    }

    pub async fn run_health_checks(&mut self, connections: &mut HashMap<PeerId, ConnectionInfo>) {
        let now = Instant::now();
        
        for (peer_id, health_status) in &mut self.monitored_connections {
            if now.duration_since(health_status.last_check) >= self.check_interval {
                // Perform health check (simplified)
                let health_check_result = self.ping_peer(*peer_id).await;
                
                match health_check_result {
                    Ok(latency) => {
                        health_status.ping_latency = Some(latency);
                        health_status.consecutive_failures = 0;
                        health_status.health_score = (health_status.health_score + 0.1).min(1.0);
                        
                        // Update connection status
                        if let Some(connection) = connections.get_mut(peer_id) {
                            connection.latency = Some(latency);
                            connection.status = ConnectionStatus::Active;
                        }
                    }
                    Err(_) => {
                        health_status.consecutive_failures += 1;
                        health_status.health_score = (health_status.health_score - 0.2).max(0.0);
                        
                        // Update connection status based on failure count
                        if let Some(connection) = connections.get_mut(peer_id) {
                            connection.status = if health_status.consecutive_failures >= 3 {
                                ConnectionStatus::Failing
                            } else {
                                ConnectionStatus::Degraded
                            };
                        }
                    }
                }
                
                health_status.last_check = now;
            }
        }
    }

    async fn ping_peer(&self, _peer_id: PeerId) -> Result<Duration, ConnectionError> {
        // Simplified ping implementation
        // In production, this would send an actual ping message
        Ok(Duration::from_millis(50))
    }
}

/// Recovery manager for failed connections
pub struct RecoveryManager {
    retry_config: RetryConfig,
    retry_queue: VecDeque<RetryAttempt>,
}

#[derive(Debug, Clone)]
struct RetryAttempt {
    connection_attempt: ConnectionAttempt,
    next_retry: Instant,
    backoff_duration: Duration,
}

impl RecoveryManager {
    pub fn new(retry_config: RetryConfig) -> Self {
        Self {
            retry_config,
            retry_queue: VecDeque::new(),
        }
    }

    pub fn schedule_retry(&mut self, mut attempt: ConnectionAttempt) {
        let backoff_duration = self.calculate_backoff(attempt.attempt_count);
        let next_retry = Instant::now() + backoff_duration;

        let retry_attempt = RetryAttempt {
            connection_attempt: attempt,
            next_retry,
            backoff_duration,
        };

        self.retry_queue.push_back(retry_attempt);
    }

    pub async fn process_retries(&mut self) {
        let now = Instant::now();
        
        while let Some(retry_attempt) = self.retry_queue.front() {
            if retry_attempt.next_retry <= now {
                let retry_attempt = self.retry_queue.pop_front().unwrap();
                
                // In production, this would trigger actual connection retry
                debug!("Retrying connection to peer {}", retry_attempt.connection_attempt.peer_id);
            } else {
                break;
            }
        }
    }

    fn calculate_backoff(&self, attempt_count: u32) -> Duration {
        let backoff = self.retry_config.initial_backoff.as_millis() as f64
            * self.retry_config.backoff_multiplier.powi(attempt_count.saturating_sub(1) as i32);
        
        Duration::from_millis(backoff.min(self.retry_config.max_backoff.as_millis() as f64) as u64)
    }
}

/// Load balancer for connection distribution
pub struct LoadBalancer {
    connection_weights: HashMap<PeerId, f64>,
}

impl LoadBalancer {
    pub fn new() -> Self {
        Self {
            connection_weights: HashMap::new(),
        }
    }

    pub fn select_best_connections(
        &self,
        connections: &HashMap<PeerId, ConnectionInfo>,
        count: usize,
    ) -> Vec<PeerId> {
        let mut connection_scores: Vec<(PeerId, f64)> = connections
            .iter()
            .filter(|(_, conn)| matches!(conn.status, ConnectionStatus::Active))
            .map(|(peer_id, conn)| {
                let weight = self.connection_weights.get(peer_id).copied().unwrap_or(1.0);
                let score = conn.quality_score * weight;
                (*peer_id, score)
            })
            .collect();

        // Sort by score (descending)
        connection_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        connection_scores
            .into_iter()
            .take(count)
            .map(|(peer_id, _)| peer_id)
            .collect()
    }

    pub fn update_metrics(&mut self, connections: &HashMap<PeerId, ConnectionInfo>) {
        // Update connection weights based on performance metrics
        for (peer_id, connection) in connections {
            let weight = self.calculate_weight(connection);
            self.connection_weights.insert(*peer_id, weight);
        }
    }

    fn calculate_weight(&self, connection: &ConnectionInfo) -> f64 {
        let mut weight = 1.0;

        // Factor in bandwidth utilization
        weight *= (1.0 - connection.bandwidth_utilization.min(0.9));

        // Factor in connection age (prefer stable connections)
        let age_hours = connection.established_at.elapsed().as_secs() as f64 / 3600.0;
        weight *= (1.0 + age_hours.min(24.0) / 24.0);

        // Factor in quality score
        weight *= connection.quality_score;

        weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_pool_creation() {
        let config = ConnectionPoolConfig::default();
        let pool = ConnectionPool::new(config);
        
        assert_eq!(pool.active_connections.len(), 0);
        assert_eq!(pool.pending_connections.len(), 0);
    }

    #[test]
    fn test_health_monitor() {
        let monitor = HealthMonitor::new(Duration::from_secs(30));
        assert_eq!(monitor.monitored_connections.len(), 0);
    }

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_backoff, Duration::from_secs(1));
    }

    #[test]
    fn test_load_balancer() {
        let balancer = LoadBalancer::new();
        let connections = HashMap::new();
        let best = balancer.select_best_connections(&connections, 5);
        assert_eq!(best.len(), 0);
    }
}
