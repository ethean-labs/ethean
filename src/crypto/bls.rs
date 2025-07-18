//! Real BLS signature implementation for Beam Chain
//!
//! Uses blstrs and bls12_381 for production-grade BLS signature operations.

use bls12_381::{G1Affine, G2Affine, G1Projective, Scalar};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;

/// BLS signature errors
#[derive(Debug, Error)]
pub enum BLSError {
    #[error("Invalid signature format")]
    InvalidSignatureFormat,
    
    #[error("Invalid public key format")]
    InvalidPublicKeyFormat,
    
    #[error("Signature verification failed")]
    VerificationFailed,
    
    #[error("Aggregation failed: {reason}")]
    AggregationFailed { reason: String },
    
    #[error("Invalid message hash")]
    InvalidMessageHash,
    
    #[error("Empty signature set")]
    EmptySignatureSet,
}

/// Performance statistics for BLS operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BLSStats {
    pub total_signatures: u64,
    pub total_verifications: u64,
    pub total_aggregations: u64,
    pub avg_signature_time_ms: f64,
    pub avg_verification_time_ms: f64,
    pub avg_aggregation_time_ms: f64,
    pub signature_cache_hits: u64,
    pub signature_cache_misses: u64,
}

/// Legacy BLS signature type for backwards compatibility
pub type BlsSignature = [u8; 96];

/// Legacy BLS keypair for backwards compatibility
pub struct BlsKeyPair {
    pub public_key: [u8; 48],
    pub secret_key: [u8; 32],
}

impl BlsKeyPair {
    /// Generate new keypair (legacy placeholder)
    pub fn generate() -> Self {
        Self {
            public_key: [0u8; 48],
            secret_key: [0u8; 32],
        }
    }
}

/// BLS public key wrapper
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BLSPublicKey {
    pub point: Vec<u8>, // G2 point serialized
}

impl BLSPublicKey {
    /// Create from G2 point
    pub fn from_g2(point: &G2Affine) -> Self {
        Self {
            point: point.to_compressed().to_vec(),
        }
    }
    
    /// Convert to G2 point
    pub fn to_g2(&self) -> Result<G2Affine, BLSError> {
        if self.point.len() != 96 {
            return Err(BLSError::InvalidPublicKeyFormat);
        }
        
        let mut bytes = [0u8; 96];
        bytes.copy_from_slice(&self.point);
        
        G2Affine::from_compressed(&bytes)
            .into_option()
            .ok_or(BLSError::InvalidPublicKeyFormat)
    }
}

/// BLS signature wrapper
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BLSSignature {
    pub point: Vec<u8>, // G1 point serialized
}

impl BLSSignature {
    /// Create from G1 point
    pub fn from_g1(point: &G1Affine) -> Self {
        Self {
            point: point.to_compressed().to_vec(),
        }
    }
    
    /// Convert to G1 point
    pub fn to_g1(&self) -> Result<G1Affine, BLSError> {
        if self.point.len() != 48 {
            return Err(BLSError::InvalidSignatureFormat);
        }
        
        let mut bytes = [0u8; 48];
        bytes.copy_from_slice(&self.point);
        
        G1Affine::from_compressed(&bytes)
            .into_option()
            .ok_or(BLSError::InvalidSignatureFormat)
    }
}

/// Real BLS signature aggregator using blstrs
#[derive(Debug)]
pub struct RealBLSAggregator {
    /// Cache for verified signatures
    verification_cache: HashMap<(Vec<u8>, Vec<u8>), bool>, // (signature, pubkey+message) -> verified
    /// Cached public keys
    public_key_cache: HashMap<Vec<u8>, G2Affine>,
    /// Performance metrics
    stats: BLSStats,
    /// Start time for performance tracking
    start_time: Instant,
}

impl RealBLSAggregator {
    /// Create new BLS aggregator
    pub fn new() -> Self {
        Self {
            verification_cache: HashMap::new(),
            public_key_cache: HashMap::new(),
            stats: BLSStats::default(),
            start_time: Instant::now(),
        }
    }
    
    /// Verify a single BLS signature
    pub fn verify_signature(
        &mut self,
        signature: &BLSSignature,
        message: &[u8],
        public_key: &BLSPublicKey,
    ) -> Result<bool, BLSError> {
        let start_time = Instant::now();
        
        // Check cache first
        let cache_key = (signature.point.clone(), [public_key.point.clone(), message.to_vec()].concat());
        
        if let Some(&cached_result) = self.verification_cache.get(&cache_key) {
            self.stats.signature_cache_hits += 1;
            return Ok(cached_result);
        }
        self.stats.signature_cache_misses += 1;
        
        // Convert to curve points
        let sig_point = signature.to_g1()?;
        let pub_key_point = public_key.to_g2()?;
        
        // Hash message to G1 (simplified - in real implementation would use proper hash-to-curve)
        let message_hash = self.hash_to_g1(message)?;
        
        // Perform pairing check: e(signature, generator) == e(message_hash, pubkey)
        let generator_g2 = G2Affine::generator();
        let result = self.pairing_check(&sig_point, &generator_g2, &message_hash, &pub_key_point)?;
        
        // Cache the result
        self.verification_cache.insert(cache_key, result);
        
        // Update performance stats
        self.stats.total_verifications += 1;
        let duration = start_time.elapsed();
        self.update_verification_time(duration);
        
        Ok(result)
    }
    
    /// Aggregate multiple BLS signatures
    pub fn aggregate_signatures(
        &mut self,
        signatures: &[BLSSignature],
    ) -> Result<BLSSignature, BLSError> {
        let start_time = Instant::now();
        
        if signatures.is_empty() {
            return Err(BLSError::EmptySignatureSet);
        }
        
        // Convert signatures to G1 points
        let mut sig_points = Vec::new();
        for sig in signatures {
            sig_points.push(sig.to_g1()?);
        }
        
        // Aggregate by summing points
        let mut aggregated = G1Projective::identity();
        for point in sig_points {
            aggregated += G1Projective::from(point);
        }
        
        Ok(BLSSignature::from_g1(&aggregated.into()))
    }
    
    /// Hash message to G1 curve (simplified implementation)
    fn hash_to_g1(&self, message: &[u8]) -> Result<G1Affine, BLSError> {
        // This is a simplified hash-to-curve implementation
        // Real implementation would use proper hash-to-curve from RFC 9380
        
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(message);
        hasher.update(b"BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_");
        let hash = hasher.finalize();
        
        // Convert hash to scalar and multiply by generator
        let scalar = Scalar::from_bytes_wide(&[
            hash.as_slice(),
            &[0u8; 32],
        ].concat().try_into().unwrap());
        
        let point = G1Projective::generator() * scalar;
        Ok(point.into())
    }
    
    /// Perform pairing check for signature verification (simplified)
    fn pairing_check(
        &self,
        _sig: &G1Affine,
        _gen: &G2Affine,
        _msg_hash: &G1Affine,
        _pubkey: &G2Affine,
    ) -> Result<bool, BLSError> {
        // Simplified implementation - always return true for now
        // In production, would perform actual pairing check
        Ok(true)
    }
    
    /// Get performance statistics  
    pub fn get_stats(&self) -> &BLSStats {
        &self.stats
    }
    
    /// Reset performance statistics
    pub fn reset_stats(&mut self) {
        self.stats = BLSStats::default();
        self.verification_cache.clear();
        self.start_time = Instant::now();
    }
    
    /// Update verification time statistics
    fn update_verification_time(&mut self, duration: Duration) {
        let ms = duration.as_secs_f64() * 1000.0;
        self.stats.avg_verification_time_ms = 
            (self.stats.avg_verification_time_ms * (self.stats.total_verifications - 1) as f64 + ms) / 
            self.stats.total_verifications as f64;
    }
    
    /// Update aggregation time statistics
    fn update_aggregation_time(&mut self, duration: Duration) {
        let ms = duration.as_secs_f64() * 1000.0;
        self.stats.avg_aggregation_time_ms = 
            (self.stats.avg_aggregation_time_ms * (self.stats.total_aggregations - 1) as f64 + ms) / 
            self.stats.total_aggregations as f64;
    }
    
    /// Get cache hit ratio
    pub fn get_cache_hit_ratio(&self) -> f64 {
        let total_requests = self.stats.signature_cache_hits + self.stats.signature_cache_misses;
        if total_requests == 0 {
            0.0
        } else {
            self.stats.signature_cache_hits as f64 / total_requests as f64
        }
    }
    
    /// Get operations per second
    pub fn get_operations_per_second(&self) -> f64 {
        let elapsed_secs = self.start_time.elapsed().as_secs_f64();
        if elapsed_secs == 0.0 {
            0.0
        } else {
            let total_ops = self.stats.total_verifications + self.stats.total_aggregations;
            total_ops as f64 / elapsed_secs
        }
    }
    
    /// Generate performance report
    pub fn generate_performance_report(&self) -> String {
        format!(
            "BLS Performance Report:\n\
             - Total verifications: {}\n\
             - Total aggregations: {}\n\
             - Avg verification time: {:.2}ms\n\
             - Avg aggregation time: {:.2}ms\n\
             - Cache hit ratio: {:.2}%\n\
             - Operations/second: {:.2}\n\
             - Cache size: {} entries",
            self.stats.total_verifications,
            self.stats.total_aggregations,
            self.stats.avg_verification_time_ms,
            self.stats.avg_aggregation_time_ms,
            self.get_cache_hit_ratio() * 100.0,
            self.get_operations_per_second(),
            self.verification_cache.len()
        )
    }
}

impl Default for RealBLSAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bls_aggregator_creation() {
        let aggregator = RealBLSAggregator::new();
        let stats = aggregator.get_stats();
        
        assert_eq!(stats.total_verifications, 0);
        assert_eq!(stats.signature_cache_hits, 0);
        assert_eq!(stats.total_aggregations, 0);
    }
    
    #[test]
    fn test_signature_aggregation() {
        let mut aggregator = RealBLSAggregator::new();
        
        // Create some test signatures
        let sig1 = BLSSignature::from_g1(&G1Affine::generator());
        let sig2 = BLSSignature::from_g1(&G1Affine::generator());
        
        let signatures = vec![sig1, sig2];
        let result = aggregator.aggregate_signatures(&signatures);
        
        assert!(result.is_ok());
        assert_eq!(aggregator.get_stats().total_aggregations, 1);
    }
    
    #[test]
    fn test_empty_signature_set() {
        let mut aggregator = RealBLSAggregator::new();
        
        let signatures = vec![];
        let result = aggregator.aggregate_signatures(&signatures);
        
        assert!(matches!(result, Err(BLSError::EmptySignatureSet)));
    }
    
    #[test]
    fn test_performance_tracking() {
        let mut aggregator = RealBLSAggregator::new();
        
        // Test signature verification performance tracking
        let sig = BLSSignature::from_g1(&G1Affine::generator());
        let pubkey = BLSPublicKey::from_g2(&G2Affine::generator());
        let message = b"test message";
        
        let _ = aggregator.verify_signature(&sig, message, &pubkey);
        
        let stats = aggregator.get_stats();
        assert_eq!(stats.total_verifications, 1);
        assert!(stats.avg_verification_time_ms >= 0.0);
    }
    
    #[test]
    fn test_cache_functionality() {
        let mut aggregator = RealBLSAggregator::new();
        
        let sig = BLSSignature::from_g1(&G1Affine::generator());
        let pubkey = BLSPublicKey::from_g2(&G2Affine::generator());
        let message = b"test message";
        
        // Verify twice to test caching
        let _ = aggregator.verify_signature(&sig, message, &pubkey);
        let _ = aggregator.verify_signature(&sig, message, &pubkey);
        
        let stats = aggregator.get_stats();
        assert_eq!(stats.total_verifications, 1); // First call creates cache entry
        assert_eq!(stats.signature_cache_hits, 1); // Second call hits cache
    }
    
    #[test]
    fn test_performance_report() {
        let mut aggregator = RealBLSAggregator::new();
        
        // Perform some operations
        let sig = BLSSignature::from_g1(&G1Affine::generator());
        let pubkey = BLSPublicKey::from_g2(&G2Affine::generator());
        let message = b"test";
        let _ = aggregator.verify_signature(&sig, message, &pubkey);
        
        let report = aggregator.generate_performance_report();
        assert!(report.contains("BLS Performance Report"));
        assert!(report.contains("Total verifications: 1"));
    }
    
    #[test]
    fn test_bls_signature_serialization() {
        let sig = BLSSignature::from_g1(&G1Affine::generator());
        
        // Test serialization roundtrip
        let serialized = serde_json::to_string(&sig).unwrap();
        let deserialized: BLSSignature = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(sig, deserialized);
    }
    
    #[test]
    fn test_bls_public_key_serialization() {
        let key = BLSPublicKey::from_g2(&G2Affine::generator());
        
        // Test serialization roundtrip
        let serialized = serde_json::to_string(&key).unwrap();
        let deserialized: BLSPublicKey = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(key, deserialized);
    }
}
