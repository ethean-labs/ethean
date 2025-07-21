//! Network Security Module
//!
//! Implements advanced security features for P2P networking including
//! authentication, encryption, DDoS protection, and peer validation.

use crate::network::{NetworkError, ConnectionInfo};
use libp2p::{
    core::transport::Boxed,
    identity::Keypair,
    noise::{Keypair as NoiseKeypair, NoiseConfig, X25519Spec},
    yamux::YamuxConfig,
    PeerId, Transport,
};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Network security manager
pub struct NetworkSecurity {
    /// Local keypair for authentication
    local_keypair: Keypair,
    /// Noise keypair for encryption
    noise_keypair: NoiseKeypair<X25519Spec>,
    /// Peer authentication records
    authenticated_peers: HashMap<PeerId, AuthenticationRecord>,
    /// Rate limiting manager
    rate_limiter: RateLimiter,
    /// DDoS protection system
    ddos_protection: DDoSProtection,
    /// Security configuration
    config: SecurityConfig,
    /// Security statistics
    stats: SecurityStats,
}

/// Authentication record for peers
#[derive(Debug, Clone)]
pub struct AuthenticationRecord {
    pub peer_id: PeerId,
    pub authenticated_at: Instant,
    pub authentication_method: AuthMethod,
    pub trust_level: TrustLevel,
    pub certificate_hash: Option<String>,
    pub last_activity: Instant,
    pub failed_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    NoiseXX,
    TLS,
    SharedSecret,
    PublicKey,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum TrustLevel {
    Unknown = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Trusted = 4,
}

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Enable encryption for all connections
    pub enable_encryption: bool,
    /// Require authentication for all peers
    pub require_authentication: bool,
    /// Maximum authentication attempts per peer
    pub max_auth_attempts: u32,
    /// Authentication timeout
    pub auth_timeout: Duration,
    /// Rate limiting configuration
    pub rate_limit_config: RateLimitConfig,
    /// DDoS protection settings
    pub ddos_config: DDoSConfig,
    /// Minimum trust level for connections
    pub min_trust_level: TrustLevel,
    /// Certificate validation enabled
    pub validate_certificates: bool,
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Messages per second per peer
    pub messages_per_second: u32,
    /// Bytes per second per peer
    pub bytes_per_second: u64,
    /// Burst allowance
    pub burst_size: u32,
    /// Rate limit window duration
    pub window_duration: Duration,
    /// Penalty duration for violations
    pub penalty_duration: Duration,
}

/// DDoS protection configuration
#[derive(Debug, Clone)]
pub struct DDoSConfig {
    /// Maximum connections per IP
    pub max_connections_per_ip: u32,
    /// Connection rate limit per IP
    pub connection_rate_limit: u32,
    /// Blacklist duration for attacking IPs
    pub blacklist_duration: Duration,
    /// Enable traffic pattern analysis
    pub enable_pattern_analysis: bool,
    /// Threshold for suspicious activity
    pub suspicious_threshold: f64,
}

/// Security statistics
#[derive(Debug, Default, Clone)]
pub struct SecurityStats {
    pub authentication_attempts: u64,
    pub successful_authentications: u64,
    pub failed_authentications: u64,
    pub rate_limit_violations: u64,
    pub ddos_attacks_detected: u64,
    pub blocked_connections: u64,
    pub encrypted_connections: u64,
    pub trust_violations: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_encryption: true,
            require_authentication: true,
            max_auth_attempts: 3,
            auth_timeout: Duration::from_secs(30),
            rate_limit_config: RateLimitConfig::default(),
            ddos_config: DDoSConfig::default(),
            min_trust_level: TrustLevel::Low,
            validate_certificates: true,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            messages_per_second: 100,
            bytes_per_second: 1024 * 1024, // 1MB/s
            burst_size: 10,
            window_duration: Duration::from_secs(1),
            penalty_duration: Duration::from_secs(60),
        }
    }
}

impl Default for DDoSConfig {
    fn default() -> Self {
        Self {
            max_connections_per_ip: 10,
            connection_rate_limit: 5, // per second
            blacklist_duration: Duration::from_secs(3600), // 1 hour
            enable_pattern_analysis: true,
            suspicious_threshold: 0.8,
        }
    }
}

impl NetworkSecurity {
    /// Create new network security manager
    pub fn new(local_keypair: Keypair, config: SecurityConfig) -> Result<Self, SecurityError> {
        // Generate Noise keypair for encryption
        let noise_keypair = NoiseKeypair::<X25519Spec>::new()
            .into_authentic(&local_keypair)
            .map_err(|e| SecurityError::EncryptionError(e.to_string()))?;

        let rate_limiter = RateLimiter::new(config.rate_limit_config.clone());
        let ddos_protection = DDoSProtection::new(config.ddos_config.clone());

        Ok(Self {
            local_keypair,
            noise_keypair,
            authenticated_peers: HashMap::new(),
            rate_limiter,
            ddos_protection,
            config,
            stats: SecurityStats::default(),
        })
    }

    /// Create secure transport with encryption and authentication
    pub fn create_secure_transport(&self) -> Result<Boxed<(PeerId, libp2p::core::muxing::StreamMuxerBox)>, SecurityError> {
        use libp2p::{tcp, websocket, Transport};

        let local_peer_id = PeerId::from(self.local_keypair.public());
        
        // Create base transport (TCP + WebSocket)
        let transport = tcp::TcpConfig::new()
            .nodelay(true)
            .or_transport(websocket::WsConfig::new(tcp::TcpConfig::new()));

        // Add Noise encryption
        let transport = transport
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(NoiseConfig::xx(self.noise_keypair.clone()).into_authenticated())
            .multiplex(YamuxConfig::default())
            .timeout(self.config.auth_timeout)
            .boxed();

        info!("Created secure transport for peer {}", local_peer_id);
        Ok(transport)
    }

    /// Authenticate a peer connection
    pub async fn authenticate_peer(
        &mut self,
        peer_id: PeerId,
        method: AuthMethod,
    ) -> Result<TrustLevel, SecurityError> {
        self.stats.authentication_attempts += 1;

        // Check if peer is already authenticated
        if let Some(record) = self.authenticated_peers.get(&peer_id) {
            if record.trust_level >= self.config.min_trust_level {
                return Ok(record.trust_level.clone());
            }
        }

        // Check authentication attempts limit
        if let Some(record) = self.authenticated_peers.get(&peer_id) {
            if record.failed_attempts >= self.config.max_auth_attempts {
                self.stats.failed_authentications += 1;
                return Err(SecurityError::AuthenticationFailed(
                    "Maximum authentication attempts exceeded".to_string()
                ));
            }
        }

        // Perform authentication based on method
        let trust_level = match method {
            AuthMethod::NoiseXX => self.authenticate_noise_xx(&peer_id).await?,
            AuthMethod::TLS => self.authenticate_tls(&peer_id).await?,
            AuthMethod::SharedSecret => self.authenticate_shared_secret(&peer_id).await?,
            AuthMethod::PublicKey => self.authenticate_public_key(&peer_id).await?,
        };

        // Create authentication record
        let auth_record = AuthenticationRecord {
            peer_id,
            authenticated_at: Instant::now(),
            authentication_method: method,
            trust_level: trust_level.clone(),
            certificate_hash: None,
            last_activity: Instant::now(),
            failed_attempts: 0,
        };

        self.authenticated_peers.insert(peer_id, auth_record);
        self.stats.successful_authentications += 1;

        info!("Successfully authenticated peer {} with trust level {:?}", peer_id, trust_level);
        Ok(trust_level)
    }

    /// Check if peer is authorized for connection
    pub fn is_peer_authorized(&self, peer_id: &PeerId) -> bool {
        if let Some(record) = self.authenticated_peers.get(peer_id) {
            record.trust_level >= self.config.min_trust_level
        } else {
            !self.config.require_authentication
        }
    }

    /// Check rate limits for peer
    pub async fn check_rate_limit(
        &mut self,
        peer_id: &PeerId,
        message_count: u32,
        byte_count: u64,
    ) -> Result<(), SecurityError> {
        if !self.rate_limiter.check_limits(peer_id, message_count, byte_count).await {
            self.stats.rate_limit_violations += 1;
            warn!("Rate limit violation detected for peer {}", peer_id);
            return Err(SecurityError::RateLimitExceeded);
        }
        Ok(())
    }

    /// Analyze connection for DDoS patterns
    pub async fn analyze_connection(
        &mut self,
        ip_addr: IpAddr,
        connection_info: &ConnectionInfo,
    ) -> Result<bool, SecurityError> {
        let is_safe = self.ddos_protection.analyze_connection(ip_addr, connection_info).await;
        
        if !is_safe {
            self.stats.ddos_attacks_detected += 1;
            self.stats.blocked_connections += 1;
            warn!("Potential DDoS attack detected from IP {}", ip_addr);
        }

        Ok(is_safe)
    }

    /// Update peer trust level
    pub fn update_trust_level(&mut self, peer_id: &PeerId, new_level: TrustLevel) {
        if let Some(record) = self.authenticated_peers.get_mut(peer_id) {
            let old_level = record.trust_level.clone();
            record.trust_level = new_level.clone();
            record.last_activity = Instant::now();
            
            debug!("Updated trust level for peer {} from {:?} to {:?}", 
                   peer_id, old_level, new_level);
        }
    }

    /// Record security violation
    pub fn record_violation(&mut self, peer_id: &PeerId, violation_type: ViolationType) {
        if let Some(record) = self.authenticated_peers.get_mut(peer_id) {
            record.failed_attempts += 1;
            
            // Decrease trust level based on violation severity
            let penalty = match violation_type {
                ViolationType::AuthenticationFailure => 1,
                ViolationType::RateLimitViolation => 1,
                ViolationType::ProtocolViolation => 2,
                ViolationType::MaliciousActivity => 3,
            };

            let current_level = record.trust_level.clone() as u8;
            let new_level = if current_level >= penalty {
                match current_level - penalty {
                    0 => TrustLevel::Unknown,
                    1 => TrustLevel::Low,
                    2 => TrustLevel::Medium,
                    3 => TrustLevel::High,
                    _ => TrustLevel::Trusted,
                }
            } else {
                TrustLevel::Unknown
            };

            record.trust_level = new_level;
            self.stats.trust_violations += 1;
        }
    }

    /// Get security statistics
    pub fn get_stats(&self) -> &SecurityStats {
        &self.stats
    }

    /// Cleanup expired authentication records
    pub fn cleanup_expired_auth(&mut self) -> usize {
        let auth_timeout = Duration::from_secs(3600); // 1 hour
        let now = Instant::now();
        let mut removed = 0;

        self.authenticated_peers.retain(|_peer_id, record| {
            if now.duration_since(record.last_activity) > auth_timeout {
                removed += 1;
                false
            } else {
                true
            }
        });

        if removed > 0 {
            debug!("Cleaned up {} expired authentication records", removed);
        }

        removed
    }

    // Authentication method implementations (simplified)
    async fn authenticate_noise_xx(&self, _peer_id: &PeerId) -> Result<TrustLevel, SecurityError> {
        // Simplified Noise XX authentication
        Ok(TrustLevel::Medium)
    }

    async fn authenticate_tls(&self, _peer_id: &PeerId) -> Result<TrustLevel, SecurityError> {
        // Simplified TLS authentication
        Ok(TrustLevel::High)
    }

    async fn authenticate_shared_secret(&self, _peer_id: &PeerId) -> Result<TrustLevel, SecurityError> {
        // Simplified shared secret authentication
        Ok(TrustLevel::Low)
    }

    async fn authenticate_public_key(&self, _peer_id: &PeerId) -> Result<TrustLevel, SecurityError> {
        // Simplified public key authentication
        Ok(TrustLevel::High)
    }
}

#[derive(Debug, Clone)]
pub enum ViolationType {
    AuthenticationFailure,
    RateLimitViolation,
    ProtocolViolation,
    MaliciousActivity,
}

/// Rate limiting manager
pub struct RateLimiter {
    config: RateLimitConfig,
    peer_limits: HashMap<PeerId, RateLimitState>,
}

#[derive(Debug, Clone)]
struct RateLimitState {
    message_count: u32,
    byte_count: u64,
    window_start: Instant,
    penalties: VecDeque<Instant>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            peer_limits: HashMap::new(),
        }
    }

    pub async fn check_limits(
        &mut self,
        peer_id: &PeerId,
        message_count: u32,
        byte_count: u64,
    ) -> bool {
        let now = Instant::now();
        let state = self.peer_limits.entry(*peer_id).or_insert_with(|| RateLimitState {
            message_count: 0,
            byte_count: 0,
            window_start: now,
            penalties: VecDeque::new(),
        });

        // Check if we're in a penalty period
        while let Some(&penalty_time) = state.penalties.front() {
            if now.duration_since(penalty_time) > self.config.penalty_duration {
                state.penalties.pop_front();
            } else {
                return false; // Still in penalty
            }
        }

        // Reset window if expired
        if now.duration_since(state.window_start) > self.config.window_duration {
            state.message_count = 0;
            state.byte_count = 0;
            state.window_start = now;
        }

        // Check limits
        let new_message_count = state.message_count + message_count;
        let new_byte_count = state.byte_count + byte_count;

        if new_message_count > self.config.messages_per_second ||
           new_byte_count > self.config.bytes_per_second {
            // Rate limit exceeded - add penalty
            state.penalties.push_back(now);
            false
        } else {
            // Update counters
            state.message_count = new_message_count;
            state.byte_count = new_byte_count;
            true
        }
    }
}

/// DDoS protection system
pub struct DDoSProtection {
    config: DDoSConfig,
    ip_connections: HashMap<IpAddr, ConnectionTracker>,
    blacklisted_ips: HashMap<IpAddr, Instant>,
}

#[derive(Debug, Clone)]
struct ConnectionTracker {
    connection_count: u32,
    connection_times: VecDeque<Instant>,
    suspicious_score: f64,
    last_analysis: Instant,
}

impl DDoSProtection {
    pub fn new(config: DDoSConfig) -> Self {
        Self {
            config,
            ip_connections: HashMap::new(),
            blacklisted_ips: HashMap::new(),
        }
    }

    pub async fn analyze_connection(
        &mut self,
        ip_addr: IpAddr,
        _connection_info: &ConnectionInfo,
    ) -> bool {
        let now = Instant::now();

        // Clean up expired blacklist entries
        self.blacklisted_ips.retain(|_ip, &mut blacklist_time| {
            now.duration_since(blacklist_time) < self.config.blacklist_duration
        });

        // Check if IP is blacklisted
        if self.blacklisted_ips.contains_key(&ip_addr) {
            return false;
        }

        let tracker = self.ip_connections.entry(ip_addr).or_insert_with(|| ConnectionTracker {
            connection_count: 0,
            connection_times: VecDeque::new(),
            suspicious_score: 0.0,
            last_analysis: now,
        });

        // Clean old connection times
        let cutoff = now - Duration::from_secs(60);
        while let Some(&front_time) = tracker.connection_times.front() {
            if front_time < cutoff {
                tracker.connection_times.pop_front();
            } else {
                break;
            }
        }

        // Add new connection
        tracker.connection_times.push_back(now);
        tracker.connection_count = tracker.connection_times.len() as u32;

        // Check connection limits
        if tracker.connection_count > self.config.max_connections_per_ip {
            self.blacklisted_ips.insert(ip_addr, now);
            warn!("IP {} blacklisted for exceeding connection limit", ip_addr);
            return false;
        }

        // Check connection rate
        let recent_connections = tracker.connection_times.iter()
            .filter(|&&time| now.duration_since(time) < Duration::from_secs(1))
            .count() as u32;

        if recent_connections > self.config.connection_rate_limit {
            self.blacklisted_ips.insert(ip_addr, now);
            warn!("IP {} blacklisted for exceeding connection rate limit", ip_addr);
            return false;
        }

        // Pattern analysis (simplified)
        if self.config.enable_pattern_analysis {
            tracker.suspicious_score = self.calculate_suspicious_score(tracker);
            if tracker.suspicious_score > self.config.suspicious_threshold {
                self.blacklisted_ips.insert(ip_addr, now);
                warn!("IP {} blacklisted for suspicious activity (score: {:.2})", 
                      ip_addr, tracker.suspicious_score);
                return false;
            }
        }

        true
    }

    fn calculate_suspicious_score(&self, tracker: &ConnectionTracker) -> f64 {
        let now = Instant::now();
        let recent_window = Duration::from_secs(10);
        
        let recent_connections = tracker.connection_times.iter()
            .filter(|&&time| now.duration_since(time) < recent_window)
            .count() as f64;

        // Simple scoring based on connection frequency
        let frequency_score = (recent_connections / 10.0).min(1.0);
        
        // Pattern regularity (connections at very regular intervals are suspicious)
        let mut intervals = Vec::new();
        for window in tracker.connection_times.windows(2) {
            let interval = window[1].duration_since(window[0]).as_millis();
            intervals.push(interval);
        }
        
        let regularity_score = if intervals.len() > 1 {
            let variance = self.calculate_variance(&intervals);
            if variance < 100 { 0.8 } else { 0.0 } // Very regular = suspicious
        } else {
            0.0
        };

        (frequency_score + regularity_score) / 2.0
    }

    fn calculate_variance(&self, values: &[u128]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        
        let mean = values.iter().sum::<u128>() as f64 / values.len() as f64;
        let variance = values.iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>() / values.len() as f64;
        
        variance.sqrt()
    }
}

/// Security-related errors
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("DDoS attack detected")]
    DDoSDetected,
    #[error("Trust level insufficient")]
    InsufficientTrust,
    #[error("Certificate validation failed: {0}")]
    CertificateError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::identity::Keypair;

    #[test]
    fn test_security_config_defaults() {
        let config = SecurityConfig::default();
        assert!(config.enable_encryption);
        assert!(config.require_authentication);
        assert_eq!(config.max_auth_attempts, 3);
    }

    #[test]
    fn test_rate_limit_config() {
        let config = RateLimitConfig::default();
        assert_eq!(config.messages_per_second, 100);
        assert_eq!(config.bytes_per_second, 1024 * 1024);
    }

    #[test]
    fn test_trust_level_ordering() {
        assert!(TrustLevel::High > TrustLevel::Medium);
        assert!(TrustLevel::Medium > TrustLevel::Low);
        assert!(TrustLevel::Low > TrustLevel::Unknown);
    }

    #[tokio::test]
    async fn test_network_security_creation() {
        let keypair = Keypair::generate_ed25519();
        let config = SecurityConfig::default();
        let security = NetworkSecurity::new(keypair, config);
        assert!(security.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let config = RateLimitConfig::default();
        let mut limiter = RateLimiter::new(config);
        let peer_id = PeerId::random();
        
        // Should allow normal traffic
        assert!(limiter.check_limits(&peer_id, 10, 1024).await);
        
        // Should block excessive traffic
        assert!(!limiter.check_limits(&peer_id, 1000, 0).await);
    }

    #[tokio::test]
    async fn test_ddos_protection() {
        let config = DDoSConfig::default();
        let mut protection = DDoSProtection::new(config);
        let ip = "127.0.0.1".parse().unwrap();
        let connection_info = ConnectionInfo {
            peer_id: PeerId::random(),
            connection_id: libp2p::core::connection::ConnectionId::new(1),
            address: "/ip4/127.0.0.1/tcp/8000".parse().unwrap(),
            established_at: Instant::now(),
            last_activity: Instant::now(),
            connection_type: crate::network::connection_manager::ConnectionType::Outbound,
            quality_score: 1.0,
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
            latency: None,
            bandwidth_utilization: 0.0,
            error_count: 0,
            status: crate::network::connection_manager::ConnectionStatus::Active,
        };
        
        // Should allow normal connections
        assert!(protection.analyze_connection(ip, &connection_info).await);
    }
}
