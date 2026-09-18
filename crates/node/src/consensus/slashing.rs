//! Advanced slashing detection system for Beam Chain
//!
//! Implements double vote detection, surround vote detection, and slashing evidence
//! generation with optimized data structures for real-time validation.

use crate::types::{Slot, Epoch, ValidatorIndex, Attestation};
use crate::consensus::{ValidatorManager, FinalityGadget};
use crate::consensus::finality::Checkpoint;
use crate::crypto::bls::{BLSSignature, BLSError};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::collections::{HashMap, BTreeMap, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::sync::Arc;

// Helper function to convert types::checkpoint::Checkpoint to consensus::finality::Checkpoint
fn convert_checkpoint(types_checkpoint: &crate::types::checkpoint::Checkpoint) -> Checkpoint {
    Checkpoint {
        epoch: types_checkpoint.epoch,
        block_hash: types_checkpoint.root,
        state_root: types_checkpoint.root, // Use same root for both
    }
}

/// Slashing detection errors
#[derive(Debug, Error)]
pub enum SlashingError {
    #[error("Invalid attestation for slashing detection")]
    InvalidAttestation,
    
    #[error("Double vote detected: validator {validator} voted for conflicting targets")]
    DoubleVoteDetected { validator: ValidatorIndex },
    
    #[error("Surround vote detected: validator {validator} violated FFG slashing condition")]
    SurroundVoteDetected { validator: ValidatorIndex },
    
    #[error("Invalid slashing evidence: {reason}")]
    InvalidEvidence { reason: String },
    
    #[error("Validator not found: {0}")]
    ValidatorNotFound(ValidatorIndex),
    
    #[error("Historical data not available for epoch {0}")]
    HistoricalDataMissing(Epoch),
    
    #[error("BLS verification failed: {0}")]
    BLSVerification(#[from] BLSError),
    
    #[error("Storage error: {reason}")]
    StorageError { reason: String },
}

/// Types of slashing violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlashingType {
    /// Double vote violation (FFG-1)
    DoubleVote,
    /// Surround vote violation (FFG-2)  
    SurroundVote,
    /// Block proposal violation
    DoubleProposal,
}

/// Attestation data for slashing detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SlashingAttestation {
    /// Attesting validator
    pub validator: ValidatorIndex,
    /// Source checkpoint
    pub source: Checkpoint,
    /// Target checkpoint  
    pub target: Checkpoint,
    /// Slot when attestation was made
    pub slot: Slot,
    /// BLS signature
    pub signature: BLSSignature,
    /// Timestamp when detected
    pub timestamp: u64,
}

impl SlashingAttestation {
    /// Create from standard attestation
    pub fn from_attestation(attestation: &Attestation, validator: ValidatorIndex) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        Self {
            validator,
            source: convert_checkpoint(&attestation.data.source),
            target: convert_checkpoint(&attestation.data.target),
            slot: attestation.data.slot,
            signature: BLSSignature { point: attestation.signature.clone() },
            timestamp,
        }
    }
    
    /// Check if this attestation conflicts with another (double vote)
    pub fn conflicts_with(&self, other: &SlashingAttestation) -> bool {
        self.validator == other.validator &&
        self.target.epoch == other.target.epoch &&
        (self.target.block_hash != other.target.block_hash ||
         self.source != other.source)
    }
    
    /// Check if this attestation surrounds another
    pub fn surrounds(&self, other: &SlashingAttestation) -> bool {
        self.validator == other.validator &&
        self.source.epoch < other.source.epoch &&
        self.target.epoch > other.target.epoch
    }
}

/// Evidence of a slashing violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingEvidence {
    /// Type of slashing violation
    pub slashing_type: SlashingType,
    /// Validator who committed the violation
    pub validator: ValidatorIndex,
    /// First conflicting attestation
    pub attestation_1: SlashingAttestation,
    /// Second conflicting attestation
    pub attestation_2: SlashingAttestation,
    /// Additional metadata
    pub metadata: SlashingMetadata,
}

/// Additional metadata for slashing evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingMetadata {
    /// When the violation was detected
    pub detection_time: u64,
    /// Detection method used
    pub detection_method: String,
    /// Severity of the violation
    pub severity: SlashingSeverity,
    /// Additional context
    pub context: HashMap<String, String>,
}

/// Severity levels for slashing violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlashingSeverity {
    /// Minor violation with small penalty
    Minor,
    /// Major violation with significant penalty
    Major,
    /// Critical violation requiring immediate action
    Critical,
}

/// Double vote detector for O(1) conflict detection
#[derive(Debug, Clone)]
pub struct DoubleVoteDetector {
    /// Latest attestations per validator per epoch
    latest_attestations: HashMap<ValidatorIndex, HashMap<Epoch, SlashingAttestation>>,
    /// Performance metrics
    checks_performed: u64,
    violations_detected: u64,
    average_check_time: Duration,
}

impl DoubleVoteDetector {
    /// Create new double vote detector
    pub fn new() -> Self {
        Self {
            latest_attestations: HashMap::new(),
            checks_performed: 0,
            violations_detected: 0,
            average_check_time: Duration::ZERO,
        }
    }
    
    /// Check for double vote and update records
    pub fn check_attestation(&mut self, attestation: &SlashingAttestation) -> Result<Option<SlashingEvidence>, SlashingError> {
        let start_time = Instant::now();
        
        // Get validator's attestations for this epoch
        let validator_attestations = self.latest_attestations
            .entry(attestation.validator)
            .or_default();
            
        // Check if there's already an attestation for this epoch
        if let Some(existing) = validator_attestations.get(&attestation.target.epoch) {
            // Check for conflict
            if attestation.conflicts_with(existing) {
                self.violations_detected += 1;
                
                let evidence = SlashingEvidence {
                    slashing_type: SlashingType::DoubleVote,
                    validator: attestation.validator,
                    attestation_1: existing.clone(),
                    attestation_2: attestation.clone(),
                    metadata: SlashingMetadata {
                        detection_time: attestation.timestamp,
                        detection_method: "double_vote_detector".to_string(),
                        severity: SlashingSeverity::Major,
                        context: HashMap::new(),
                    },
                };
                
                self.update_performance_metrics(start_time.elapsed());
                return Ok(Some(evidence));
            }
        }
        
        // No conflict, update the record
        validator_attestations.insert(attestation.target.epoch, attestation.clone());
        
        self.update_performance_metrics(start_time.elapsed());
        Ok(None)
    }
    
    /// Update performance metrics
    fn update_performance_metrics(&mut self, check_time: Duration) {
        self.checks_performed += 1;
        
        // Update rolling average
        let weight = 0.1; // Weight for new sample
        self.average_check_time = Duration::from_nanos(
            ((1.0 - weight) * self.average_check_time.as_nanos() as f64 +
             weight * check_time.as_nanos() as f64) as u64
        );
    }
    
    /// Cleanup old attestations beyond finalized epoch
    pub fn cleanup_old_attestations(&mut self, finalized_epoch: Epoch) {
        for attestations in self.latest_attestations.values_mut() {
            attestations.retain(|&epoch, _| epoch >= finalized_epoch);
        }
        
        // Remove validators with no remaining attestations
        self.latest_attestations.retain(|_, attestations| !attestations.is_empty());
    }
    
    /// Get performance statistics
    pub fn get_stats(&self) -> (u64, u64, Duration) {
        (self.checks_performed, self.violations_detected, self.average_check_time)
    }
}

/// Interval tree node for efficient surround vote detection
#[derive(Debug, Clone)]
struct IntervalNode {
    /// Source epoch interval start
    source_start: Epoch,
    /// Target epoch interval end  
    target_end: Epoch,
    /// Associated attestation
    attestation: SlashingAttestation,
    /// Maximum target epoch in subtree
    max_target: Epoch,
}

impl IntervalNode {
    /// Check if this interval overlaps with given range
    pub fn overlaps_with(&self, start: Epoch, end: Epoch) -> bool {
        self.source_start <= end && self.target_end >= start
    }
    
    /// Update max_target for tree balancing
    pub fn update_max_target(&mut self, subtree_max: Epoch) {
        self.max_target = self.max_target.max(subtree_max);
    }
}

/// Surround vote detector using interval tree optimization
#[derive(Debug, Clone)]
pub struct SurroundVoteDetector {
    /// Interval tree for efficient overlap detection
    intervals: BTreeMap<Epoch, Vec<IntervalNode>>,
    /// Performance metrics
    checks_performed: u64,
    violations_detected: u64,
    average_check_time: Duration,
}

impl SurroundVoteDetector {
    /// Create new surround vote detector
    pub fn new() -> Self {
        Self {
            intervals: BTreeMap::new(),
            checks_performed: 0,
            violations_detected: 0,
            average_check_time: Duration::ZERO,
        }
    }
    
    /// Check for surround vote violations
    pub fn check_attestation(&mut self, attestation: &SlashingAttestation) -> Result<Option<SlashingEvidence>, SlashingError> {
        let start_time = Instant::now();
        
        // Check if this attestation is surrounded by any existing attestation
        if let Some(surrounding) = self.find_surrounding_attestation(attestation)? {
            self.violations_detected += 1;
            
            let evidence = SlashingEvidence {
                slashing_type: SlashingType::SurroundVote,
                validator: attestation.validator,
                attestation_1: surrounding,
                attestation_2: attestation.clone(),
                metadata: SlashingMetadata {
                    detection_time: attestation.timestamp,
                    detection_method: "surround_vote_detector".to_string(),
                    severity: SlashingSeverity::Critical,
                    context: HashMap::new(),
                },
            };
            
            self.update_performance_metrics(start_time.elapsed());
            return Ok(Some(evidence));
        }
        
        // Check if this attestation surrounds any existing attestation
        if let Some(surrounded) = self.find_surrounded_attestation(attestation)? {
            self.violations_detected += 1;
            
            let evidence = SlashingEvidence {
                slashing_type: SlashingType::SurroundVote,
                validator: attestation.validator,
                attestation_1: attestation.clone(),
                attestation_2: surrounded,
                metadata: SlashingMetadata {
                    detection_time: attestation.timestamp,
                    detection_method: "surround_vote_detector".to_string(),
                    severity: SlashingSeverity::Critical,
                    context: HashMap::new(),
                },
            };
            
            self.update_performance_metrics(start_time.elapsed());
            return Ok(Some(evidence));
        }
        
        // No violation, add to interval tree
        self.add_attestation(attestation);
        
        self.update_performance_metrics(start_time.elapsed());
        Ok(None)
    }
    
    /// Find attestation that surrounds the given attestation
    fn find_surrounding_attestation(&self, attestation: &SlashingAttestation) -> Result<Option<SlashingAttestation>, SlashingError> {
        // Look for attestations from same validator with source < attestation.source and target > attestation.target
        for intervals in self.intervals.values() {
            for node in intervals {
                if node.attestation.validator == attestation.validator &&
                   node.source_start < attestation.source.epoch &&
                   node.target_end > attestation.target.epoch {
                    return Ok(Some(node.attestation.clone()));
                }
            }
        }
        Ok(None)
    }
    
    /// Find attestation that is surrounded by the given attestation
    fn find_surrounded_attestation(&self, attestation: &SlashingAttestation) -> Result<Option<SlashingAttestation>, SlashingError> {
        // Look for attestations from same validator with source > attestation.source and target < attestation.target
        for intervals in self.intervals.values() {
            for node in intervals {
                if node.attestation.validator == attestation.validator &&
                   node.source_start > attestation.source.epoch &&
                   node.target_end < attestation.target.epoch {
                    return Ok(Some(node.attestation.clone()));
                }
            }
        }
        Ok(None)
    }
    
    /// Add attestation to interval tree
    fn add_attestation(&mut self, attestation: &SlashingAttestation) {
        let node = IntervalNode {
            source_start: attestation.source.epoch,
            target_end: attestation.target.epoch,
            attestation: attestation.clone(),
            max_target: attestation.target.epoch,
        };
        
        self.intervals
            .entry(attestation.source.epoch)
            .or_default()
            .push(node);
    }
    
    /// Update performance metrics
    fn update_performance_metrics(&mut self, check_time: Duration) {
        self.checks_performed += 1;
        
        // Update rolling average
        let weight = 0.1;
        self.average_check_time = Duration::from_nanos(
            ((1.0 - weight) * self.average_check_time.as_nanos() as f64 +
             weight * check_time.as_nanos() as f64) as u64
        );
    }
    
    /// Cleanup old intervals beyond finalized epoch
    pub fn cleanup_old_intervals(&mut self, finalized_epoch: Epoch) {
        self.intervals.retain(|&epoch, _| epoch >= finalized_epoch);
    }
    
    /// Get performance statistics
    pub fn get_stats(&self) -> (u64, u64, Duration) {
        (self.checks_performed, self.violations_detected, self.average_check_time)
    }
}

/// Historical attestation tracking database
#[derive(Debug, Clone)]
pub struct HistoricalTracker {
    /// Attestations by validator and epoch
    attestations: HashMap<ValidatorIndex, BTreeMap<Epoch, Vec<SlashingAttestation>>>,
    /// Maximum epochs to retain
    max_epochs_retained: u64,
    /// Memory usage tracking
    memory_usage_bytes: u64,
}

impl HistoricalTracker {
    /// Create new historical tracker
    pub fn new(max_epochs_retained: u64) -> Self {
        Self {
            attestations: HashMap::new(),
            max_epochs_retained,
            memory_usage_bytes: 0,
        }
    }
    
    /// Add attestation to historical tracking
    pub fn add_attestation(&mut self, attestation: &SlashingAttestation) {
        let validator_history = self.attestations
            .entry(attestation.validator)
            .or_default();
            
        validator_history
            .entry(attestation.target.epoch)
            .or_default()
            .push(attestation.clone());
        
        // Update memory usage estimate
        self.memory_usage_bytes += std::mem::size_of::<SlashingAttestation>() as u64;
    }
    
    /// Get attestations for validator in epoch range
    pub fn get_attestations(&self, validator: ValidatorIndex, start_epoch: Epoch, end_epoch: Epoch) -> Vec<SlashingAttestation> {
        if let Some(validator_history) = self.attestations.get(&validator) {
            let mut result = Vec::new();
            for (&_epoch, attestations) in validator_history.range(start_epoch..=end_epoch) {
                result.extend(attestations.iter().cloned());
            }
            result
        } else {
            Vec::new()
        }
    }
    
    /// Prune old attestations beyond retention period
    pub fn prune_old_attestations(&mut self, current_epoch: Epoch) {
        let cutoff_epoch = current_epoch.saturating_sub(self.max_epochs_retained);
        
        for validator_history in self.attestations.values_mut() {
            let old_count = validator_history.len();
            validator_history.retain(|&epoch, _| epoch >= cutoff_epoch);
            let new_count = validator_history.len();
            
            // Update memory usage
            let removed_count = old_count - new_count;
            self.memory_usage_bytes = self.memory_usage_bytes.saturating_sub(
                removed_count as u64 * std::mem::size_of::<SlashingAttestation>() as u64
            );
        }
        
        // Remove validators with no remaining attestations
        self.attestations.retain(|_, history| !history.is_empty());
    }
    
    /// Get memory usage in bytes
    pub fn get_memory_usage(&self) -> u64 {
        self.memory_usage_bytes
    }
    
    /// Get total attestation count
    pub fn get_attestation_count(&self) -> usize {
        self.attestations.values()
            .map(|history| history.values().map(|atts| atts.len()).sum::<usize>())
            .sum()
    }
}

/// Main slashing detector coordinator
#[derive(Debug)]
pub struct SlashingDetector {
    /// Double vote detector
    double_vote_detector: DoubleVoteDetector,
    /// Surround vote detector
    surround_vote_detector: SurroundVoteDetector,
    /// Historical attestation tracker
    historical_tracker: HistoricalTracker,
    /// Validator manager for verification
    validator_manager: Arc<ValidatorManager>,
    /// Finality gadget for epoch information
    finality_gadget: Option<Arc<FinalityGadget>>,
    /// Detected violations queue
    violation_queue: VecDeque<SlashingEvidence>,
    /// Performance metrics
    total_checks: u64,
    total_violations: u64,
    average_detection_time: Duration,
}

impl SlashingDetector {
    /// Create new slashing detector
    pub fn new(validator_manager: Arc<ValidatorManager>) -> Self {
        Self {
            double_vote_detector: DoubleVoteDetector::new(),
            surround_vote_detector: SurroundVoteDetector::new(),
            historical_tracker: HistoricalTracker::new(100), // Retain 100 epochs
            validator_manager,
            finality_gadget: None,
            violation_queue: VecDeque::new(),
            total_checks: 0,
            total_violations: 0,
            average_detection_time: Duration::ZERO,
        }
    }
    
    /// Set finality gadget for epoch tracking
    pub fn set_finality_gadget(&mut self, finality_gadget: Arc<FinalityGadget>) {
        self.finality_gadget = Some(finality_gadget);
    }
    
    /// Process an attestation for slashing detection
    pub fn process_attestation(&mut self, attestation: &Attestation, validator: ValidatorIndex) -> Result<Vec<SlashingEvidence>, SlashingError> {
        let start_time = Instant::now();
        let mut violations = Vec::new();
        
        // Convert to slashing attestation format
        let slashing_attestation = SlashingAttestation::from_attestation(attestation, validator);
        
        // Check for double vote
        if let Some(evidence) = self.double_vote_detector.check_attestation(&slashing_attestation)? {
            violations.push(evidence);
        }
        
        // Check for surround vote  
        if let Some(evidence) = self.surround_vote_detector.check_attestation(&slashing_attestation)? {
            violations.push(evidence);
        }
        
        // Add to historical tracking
        self.historical_tracker.add_attestation(&slashing_attestation);
        
        // Update statistics
        self.total_checks += 1;
        self.total_violations += violations.len() as u64;
        
        // Add violations to queue
        for violation in &violations {
            self.violation_queue.push_back(violation.clone());
        }
        
        // Update performance metrics
        self.update_performance_metrics(start_time.elapsed());
        
        Ok(violations)
    }
    
    /// Verify slashing evidence
    pub fn verify_evidence(&self, evidence: &SlashingEvidence) -> Result<bool, SlashingError> {
        // Verify both attestations are valid
        if evidence.attestation_1.validator != evidence.attestation_2.validator {
            return Ok(false);
        }
        
        match evidence.slashing_type {
            SlashingType::DoubleVote => {
                Ok(evidence.attestation_1.conflicts_with(&evidence.attestation_2))
            },
            SlashingType::SurroundVote => {
                Ok(evidence.attestation_1.surrounds(&evidence.attestation_2) ||
                   evidence.attestation_2.surrounds(&evidence.attestation_1))
            },
            SlashingType::DoubleProposal => {
                // Implement block proposal slashing
                Ok(evidence.block_header_1.slot == evidence.block_header_2.slot &&
                   evidence.block_header_1.proposer_index == evidence.block_header_2.proposer_index &&
                   evidence.block_header_1.block_root != evidence.block_header_2.block_root)
            },
        }
    }
    
    /// Process proposer slashing
    fn process_proposer_slashing(
        &mut self,
        state: &mut BeaconState,
        proposer_slashing: &ProposerSlashing,
    ) -> Result<(), SlashingError> {
        let validator_index = proposer_slashing.signed_header_1.message.proposer_index;
        
        // Verify the slashing conditions
        if proposer_slashing.signed_header_1.message.slot != proposer_slashing.signed_header_2.message.slot {
            return Err(SlashingError::InvalidEvidence("Headers from different slots".to_string()));
        }
        
        if proposer_slashing.signed_header_1.message.proposer_index != proposer_slashing.signed_header_2.message.proposer_index {
            return Err(SlashingError::InvalidEvidence("Headers from different proposers".to_string()));
        }
        
        if proposer_slashing.signed_header_1.message == proposer_slashing.signed_header_2.message {
            return Err(SlashingError::InvalidEvidence("Identical headers".to_string()));
        }
        
        // Get validator
        if let Some(validator) = state.validators.get_mut(validator_index) {
            if validator.slashed {
                return Err(SlashingError::AlreadySlashed);
            }
            
            // Apply slashing
            validator.slashed = true;
            validator.withdrawable_epoch = validator.withdrawable_epoch.max(
                state.current_epoch(self.config.slots_per_epoch) + self.config.epochs_per_slashings_vector
            );
            
            // Calculate slashing penalty
            let penalty = validator.effective_balance / self.config.min_slashing_penalty_quotient;
            validator.effective_balance = validator.effective_balance.saturating_sub(penalty);
            
            self.stats.total_proposer_slashings += 1;
        }
        
        Ok(())
    }
    
    /// Get next violation from queue
    pub fn get_next_violation(&mut self) -> Option<SlashingEvidence> {
        self.violation_queue.pop_front()
    }
    
    /// Get validator information for slashing detection
    pub fn get_validator_info(&self, validator_index: ValidatorIndex) -> Option<bool> {
        // Use validator manager to check if validator has balance (indicating it's active)
        self.validator_manager.get_balance(validator_index).map(|balance| balance > 0)
    }
    
    /// Cleanup old data based on finalized epoch
    pub fn cleanup_old_data(&mut self) {
        if let Some(ref finality_gadget) = self.finality_gadget {
            let finalized_epoch = finality_gadget.get_finalized_checkpoint().epoch;
            
            self.double_vote_detector.cleanup_old_attestations(finalized_epoch);
            self.surround_vote_detector.cleanup_old_intervals(finalized_epoch);
            self.historical_tracker.prune_old_attestations(finalized_epoch);
        }
    }
    
    /// Update performance metrics
    fn update_performance_metrics(&mut self, detection_time: Duration) {
        let weight = 0.1;
        self.average_detection_time = Duration::from_nanos(
            ((1.0 - weight) * self.average_detection_time.as_nanos() as f64 +
             weight * detection_time.as_nanos() as f64) as u64
        );
    }
    
    /// Get comprehensive performance statistics
    pub fn get_stats(&self) -> SlashingStats {
        let (double_checks, double_violations, double_time) = self.double_vote_detector.get_stats();
        let (surround_checks, surround_violations, surround_time) = self.surround_vote_detector.get_stats();
        
        SlashingStats {
            total_checks: self.total_checks,
            total_violations: self.total_violations,
            average_detection_time: self.average_detection_time,
            double_vote_checks: double_checks,
            double_vote_violations: double_violations,
            double_vote_avg_time: double_time,
            surround_vote_checks: surround_checks,
            surround_vote_violations: surround_violations,
            surround_vote_avg_time: surround_time,
            memory_usage_bytes: self.historical_tracker.get_memory_usage(),
            historical_attestation_count: self.historical_tracker.get_attestation_count(),
            violation_queue_size: self.violation_queue.len(),
        }
    }
}

/// Comprehensive slashing detection statistics
#[derive(Debug, Clone)]
pub struct SlashingStats {
    pub total_checks: u64,
    pub total_violations: u64,
    pub average_detection_time: Duration,
    pub double_vote_checks: u64,
    pub double_vote_violations: u64,
    pub double_vote_avg_time: Duration,
    pub surround_vote_checks: u64,
    pub surround_vote_violations: u64,
    pub surround_vote_avg_time: Duration,
    pub memory_usage_bytes: u64,
    pub historical_attestation_count: usize,
    pub violation_queue_size: usize,
}

impl Default for DoubleVoteDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SurroundVoteDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::validator_management::{ValidatorConfig, ValidatorManager};
    use crate::storage::{StateStore, Database};
    
    fn create_test_attestation(validator: ValidatorIndex, source_epoch: Epoch, target_epoch: Epoch, target_hash: [u8; 32]) -> SlashingAttestation {
        SlashingAttestation {
            validator,
            source: Checkpoint {
                epoch: source_epoch,
                block_hash: [source_epoch.get() as u8; 32],
                state_root: [source_epoch.get() as u8; 32],
            },
            target: Checkpoint {
                epoch: target_epoch,
                block_hash: target_hash,
                state_root: [target_epoch.get() as u8; 32],
            },
            slot: target_epoch * 32,
            signature: create_mock_signature(validator, target_epoch),
            timestamp: target_epoch * 1000,
        }
    }
    
    fn create_mock_signature(validator: ValidatorIndex, epoch: Epoch) -> BLSSignature {
        use bls12_381::{G1Projective, Scalar};
        
        let scalar = Scalar::from(validator + epoch + 1);
        let point = G1Projective::generator() * scalar;
        BLSSignature::from_g1(&point.into())
    }
    
    fn setup_slashing_detector() -> SlashingDetector {
        let validator_config = ValidatorConfig::default();
        let database = Database::in_memory();
        let state_store = StateStore::new(database);
        let validator_manager = Arc::new(ValidatorManager::new(validator_config, state_store));
        
        SlashingDetector::new(validator_manager)
    }
    
    #[test]
    fn test_slashing_detector_creation() {
        let detector = setup_slashing_detector();
        let stats = detector.get_stats();
        
        assert_eq!(stats.total_checks, 0);
        assert_eq!(stats.total_violations, 0);
        assert_eq!(stats.violation_queue_size, 0);
    }
    
    #[test]
    fn test_double_vote_detection() {
        let mut detector = DoubleVoteDetector::new();
        
        // Create two conflicting attestations
        let att1 = create_test_attestation(0, 1, 2, [1u8; 32]);
        let att2 = create_test_attestation(0, 1, 2, [2u8; 32]); // Different target hash
        
        // First attestation should be fine
        let result1 = detector.check_attestation(&att1).unwrap();
        assert!(result1.is_none());
        
        // Second attestation should trigger double vote detection
        let result2 = detector.check_attestation(&att2).unwrap();
        assert!(result2.is_some());
        
        let evidence = result2.unwrap();
        assert_eq!(evidence.slashing_type, SlashingType::DoubleVote);
        assert_eq!(evidence.validator, 0);
        
        let (checks, violations, _) = detector.get_stats();
        assert_eq!(checks, 2);
        assert_eq!(violations, 1);
    }
    
    #[test]
    fn test_surround_vote_detection() {
        let mut detector = SurroundVoteDetector::new();
        
        // Create surrounding attestation (source=1, target=5)
        let surrounding = create_test_attestation(0, 1, 5, [1u8; 32]);
        
        // Create surrounded attestation (source=2, target=4)
        let surrounded = create_test_attestation(0, 2, 4, [2u8; 32]);
        
        // Add surrounding attestation first
        let result1 = detector.check_attestation(&surrounding).unwrap();
        assert!(result1.is_none());
        
        // Add surrounded attestation - should trigger violation
        let result2 = detector.check_attestation(&surrounded).unwrap();
        assert!(result2.is_some());
        
        let evidence = result2.unwrap();
        assert_eq!(evidence.slashing_type, SlashingType::SurroundVote);
        assert_eq!(evidence.validator, 0);
        
        let (checks, violations, _) = detector.get_stats();
        assert_eq!(checks, 2);
        assert_eq!(violations, 1);
    }
    
    #[test]
    fn test_historical_tracker() {
        let mut tracker = HistoricalTracker::new(10); // Retain 10 epochs
        
        // Add attestations for different epochs
        for epoch in 1u64..15 {
            let att = create_test_attestation(0, epoch.saturating_sub(1), epoch, [epoch.get() as u8; 32]);
            tracker.add_attestation(&att);
        }
        
        assert_eq!(tracker.get_attestation_count(), 14);
        
        // Prune old attestations (current epoch = 20, retain last 10)
        tracker.prune_old_attestations(20);
        
        // Should retain epochs 10-14 (5 attestations)
        assert!(tracker.get_attestation_count() <= 10);
        
        // Check memory usage tracking
        assert!(tracker.get_memory_usage() > 0);
    }
    
    #[test]
    fn test_evidence_verification() {
        let detector = setup_slashing_detector();
        
        // Create valid double vote evidence
        let att1 = create_test_attestation(0, 1, 2, [1u8; 32]);
        let att2 = create_test_attestation(0, 1, 2, [2u8; 32]);
        
        let evidence = SlashingEvidence {
            slashing_type: SlashingType::DoubleVote,
            validator: 0,
            attestation_1: att1,
            attestation_2: att2,
            metadata: SlashingMetadata {
                detection_time: 1000,
                detection_method: "test".to_string(),
                severity: SlashingSeverity::Major,
                context: HashMap::new(),
            },
        };
        
        let is_valid = detector.verify_evidence(&evidence).unwrap();
        assert!(is_valid);
    }
    
    #[test]
    fn test_performance_tracking() {
        let mut detector = setup_slashing_detector();
        
        // Process multiple attestations
        for i in 1u64..10 {
            let att = create_test_attestation(i % 3, i.saturating_sub(1), i, [i as u8; 32]);
            
            // Create mock attestation for processing
            let attestation = Attestation {
                data: crate::types::attestation::AttestationData {
                    slot: att.slot,
                    source: crate::types::checkpoint::Checkpoint {
                        epoch: att.source.epoch,
                        root: att.source.block_hash,
                    },
                    target: crate::types::checkpoint::Checkpoint {
                        epoch: att.target.epoch,
                        root: att.target.block_hash,
                    },
                    beacon_block_root: [i as u8; 32],
                    index: 0,
                },
                signature: att.signature.point.clone(),
                aggregation_bits: vec![true; 10],
            };
            
            detector.process_attestation(&attestation, att.validator).unwrap();
        }
        
        let stats = detector.get_stats();
        assert_eq!(stats.total_checks, 9);
        assert!(stats.average_detection_time < Duration::from_millis(100)); // Should be fast
        assert!(stats.memory_usage_bytes > 0);
    }
    
    #[test]
    fn test_cleanup_operations() {
        let mut detector = setup_slashing_detector();
        
        // Add some attestations
        for i in 1u64..5 {
            let att = create_test_attestation(0, i.saturating_sub(1), i, [i as u8; 32]);
            
            let attestation = Attestation {
                data: crate::types::attestation::AttestationData {
                    slot: att.slot,
                    source: crate::types::checkpoint::Checkpoint {
                        epoch: att.source.epoch,
                        root: att.source.block_hash,
                    },
                    target: crate::types::checkpoint::Checkpoint {
                        epoch: att.target.epoch,
                        root: att.target.block_hash,
                    },
                    beacon_block_root: [i as u8; 32],
                    index: 0,
                },
                signature: att.signature.point.clone(),
                aggregation_bits: vec![true; 10],
            };
            
            detector.process_attestation(&attestation, att.validator).unwrap();
        }
        
        // Verify data exists
        let initial_stats = detector.get_stats();
        assert!(initial_stats.historical_attestation_count > 0);
        
        // Cleanup (without finality gadget, should be safe operation)
        detector.cleanup_old_data();
        
        // Data should still exist since no finality gadget is set
        let final_stats = detector.get_stats();
        assert_eq!(final_stats.historical_attestation_count, initial_stats.historical_attestation_count);
    }
    
    #[test]
    fn test_violation_queue() {
        let mut detector = setup_slashing_detector();
        
        // Create conflicting attestations to generate violations
        let att1 = create_test_attestation(0, 1, 2, [1u8; 32]);
        let att2 = create_test_attestation(0, 1, 2, [2u8; 32]);
        
        let attestation1 = Attestation {
            data: crate::types::attestation::AttestationData {
                slot: att1.slot,
                source: crate::types::checkpoint::Checkpoint {
                    epoch: att1.source.epoch,
                    root: att1.source.block_hash,
                },
                target: crate::types::checkpoint::Checkpoint {
                    epoch: att1.target.epoch,
                    root: att1.target.block_hash,
                },
                beacon_block_root: att1.target.block_hash,
                index: 0,
            },
            signature: att1.signature.point.clone(),
            aggregation_bits: vec![true; 10],
        };
        
        let attestation2 = Attestation {
            data: crate::types::attestation::AttestationData {
                slot: att2.slot,
                source: crate::types::checkpoint::Checkpoint {
                    epoch: att2.source.epoch,
                    root: att2.source.block_hash,
                },
                target: crate::types::checkpoint::Checkpoint {
                    epoch: att2.target.epoch,
                    root: att2.target.block_hash,
                },
                beacon_block_root: att2.target.block_hash,
                index: 0,
            },
            signature: att2.signature.point.clone(),
            aggregation_bits: vec![true; 10],
        };
        
        // Process attestations
        detector.process_attestation(&attestation1, 0).unwrap();
        let violations = detector.process_attestation(&attestation2, 0).unwrap();
        
        assert_eq!(violations.len(), 1);
        
        // Check violation queue
        let stats = detector.get_stats();
        assert_eq!(stats.violation_queue_size, 1);
        
        // Get violation from queue
        let violation = detector.get_next_violation();
        assert!(violation.is_some());
        
        // Queue should now be empty
        let final_stats = detector.get_stats();
        assert_eq!(final_stats.violation_queue_size, 0);
    }
}
