//! Validator management system for Beam Chain
//!
//! Handles validator activation, exit, balance updates, and slashing detection.

use crate::types::{BeaconState, Validator, ValidatorIndex, Epoch};
use crate::types::validator::PublicKey;
use crate::storage::{StateStore, DatabaseError};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::collections::{HashMap, VecDeque};

/// Validator management errors
#[derive(Debug, Error)]
pub enum ValidatorError {
    #[error("Validator not found: {0}")]
    NotFound(ValidatorIndex),
    
    #[error("Invalid validator index: {0}")]
    InvalidIndex(ValidatorIndex),
    
    #[error("Validator already exists: {0:?}")]
    AlreadyExists(PublicKey),
    
    #[error("Insufficient balance: {balance}, required: {required}")]
    InsufficientBalance { balance: u64, required: u64 },
    
    #[error("Validator already slashed: {0}")]
    AlreadySlashed(ValidatorIndex),
    
    #[error("Storage error: {0}")]
    Storage(#[from] DatabaseError),
    
    #[error("Activation queue full")]
    ActivationQueueFull,
    
    #[error("Exit queue full")]
    ExitQueueFull,
}

/// Validator state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorState {
    pub index: ValidatorIndex,
    pub validator: Validator,
    pub effective_balance: u64,
    pub last_attestation_epoch: Option<Epoch>,
    pub slashing_epoch: Option<Epoch>,
    pub performance_score: f64,
}

/// Validator performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorPerformance {
    pub index: ValidatorIndex,
    pub attestation_success_rate: f64,
    pub block_proposal_success_rate: f64,
    pub uptime_percentage: f64,
    pub average_inclusion_delay: f64,
    pub penalties_incurred: u64,
}

/// Validator activation queue entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationQueueEntry {
    pub validator_index: ValidatorIndex,
    pub activation_epoch: Epoch,
    pub deposit_amount: u64,
    pub priority: u64,
    pub deposit_timestamp: u64,
}

/// Validator exit queue entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitQueueEntry {
    pub validator_index: ValidatorIndex,
    pub exit_epoch: Epoch,
    pub voluntary: bool,
    pub withdrawal_epoch: Epoch,
}

/// Activation queue management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationQueue {
    queue: VecDeque<ActivationQueueEntry>,
    max_activations_per_epoch: u64,
    activation_delay: u64,
}

impl ActivationQueue {
    pub fn new(max_activations_per_epoch: u64, activation_delay: u64) -> Self {
        Self {
            queue: VecDeque::new(),
            max_activations_per_epoch,
            activation_delay,
        }
    }

    pub fn enqueue(&mut self, entry: ActivationQueueEntry) -> Result<(), ValidatorError> {
        // Insert in order by priority (higher priority first)
        let insert_pos = self.queue
            .iter()
            .position(|e| e.priority < entry.priority)
            .unwrap_or(self.queue.len());
        
        self.queue.insert(insert_pos, entry);
        Ok(())
    }

    pub fn process_activations(&mut self, current_epoch: Epoch) -> Vec<ActivationQueueEntry> {
        let mut activations = Vec::new();
        let mut processed_count = 0;

        while let Some(entry) = self.queue.front() {
            if processed_count >= self.max_activations_per_epoch {
                break;
            }
            
            if entry.activation_epoch + self.activation_delay <= current_epoch {
                activations.push(self.queue.pop_front().unwrap());
                processed_count += 1;
            } else {
                break;
            }
        }

        activations
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

/// Exit queue management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitQueue {
    queue: VecDeque<ExitQueueEntry>,
    max_exits_per_epoch: u64,
    exit_delay: u64,
}

impl ExitQueue {
    pub fn new(max_exits_per_epoch: u64, exit_delay: u64) -> Self {
        Self {
            queue: VecDeque::new(),
            max_exits_per_epoch,
            exit_delay,
        }
    }

    pub fn enqueue(&mut self, entry: ExitQueueEntry) -> Result<(), ValidatorError> {
        // Insert in order by exit epoch (earliest first)
        let insert_pos = self.queue
            .iter()
            .position(|e| e.exit_epoch > entry.exit_epoch)
            .unwrap_or(self.queue.len());
        
        self.queue.insert(insert_pos, entry);
        Ok(())
    }

    pub fn process_exits(&mut self, current_epoch: Epoch) -> Vec<ExitQueueEntry> {
        let mut exits = Vec::new();
        let mut processed_count = 0;

        while let Some(entry) = self.queue.front() {
            if processed_count >= self.max_exits_per_epoch {
                break;
            }
            
            if entry.exit_epoch <= current_epoch {
                exits.push(self.queue.pop_front().unwrap());
                processed_count += 1;
            } else {
                break;
            }
        }

        exits
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

/// Balance tracking system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceTracker {
    balances: HashMap<ValidatorIndex, u64>,
    effective_balances: HashMap<ValidatorIndex, u64>,
    balance_history: HashMap<ValidatorIndex, Vec<(Epoch, u64)>>,
    min_effective_balance: u64,
    max_effective_balance: u64,
}

impl BalanceTracker {
    pub fn new(min_effective_balance: u64, max_effective_balance: u64) -> Self {
        Self {
            balances: HashMap::new(),
            effective_balances: HashMap::new(),
            balance_history: HashMap::new(),
            min_effective_balance,
            max_effective_balance,
        }
    }

    pub fn set_balance(&mut self, validator_index: ValidatorIndex, balance: u64, epoch: Epoch) {
        self.balances.insert(validator_index, balance);
        
        // Calculate effective balance (capped between min and max)
        let effective_balance = balance
            .max(self.min_effective_balance)
            .min(self.max_effective_balance);
        self.effective_balances.insert(validator_index, effective_balance);

        // Record in history
        self.balance_history
            .entry(validator_index)
            .or_insert_with(Vec::new)
            .push((epoch, balance));
    }

    pub fn get_balance(&self, validator_index: ValidatorIndex) -> Option<u64> {
        self.balances.get(&validator_index).copied()
    }

    pub fn get_effective_balance(&self, validator_index: ValidatorIndex) -> Option<u64> {
        self.effective_balances.get(&validator_index).copied()
    }

    pub fn apply_reward(&mut self, validator_index: ValidatorIndex, reward: u64, epoch: Epoch) -> Result<(), ValidatorError> {
        let current_balance = self.get_balance(validator_index)
            .ok_or(ValidatorError::NotFound(validator_index))?;
        
        let new_balance = current_balance.saturating_add(reward);
        self.set_balance(validator_index, new_balance, epoch);
        Ok(())
    }

    pub fn apply_penalty(&mut self, validator_index: ValidatorIndex, penalty: u64, epoch: Epoch) -> Result<(), ValidatorError> {
        let current_balance = self.get_balance(validator_index)
            .ok_or(ValidatorError::NotFound(validator_index))?;
        
        let new_balance = current_balance.saturating_sub(penalty);
        self.set_balance(validator_index, new_balance, epoch);
        Ok(())
    }

    pub fn get_balance_history(&self, validator_index: ValidatorIndex) -> Option<&Vec<(Epoch, u64)>> {
        self.balance_history.get(&validator_index)
    }
}

/// Validator management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// Minimum deposit amount (in Gwei)
    pub min_deposit_amount: u64,
    /// Maximum validators per epoch
    pub max_validators_per_epoch: u64,
    /// Activation delay in epochs
    pub activation_delay: u64,
    /// Exit delay in epochs
    pub exit_delay: u64,
    /// Slashing penalty multiplier
    pub slashing_penalty_multiplier: u64,
    /// Inactivity penalty per epoch
    pub inactivity_penalty_per_epoch: u64,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            min_deposit_amount: 1_000_000_000, // 1 ETH in Gwei (Beam Chain lower requirement)
            max_validators_per_epoch: 8,
            activation_delay: 256, // ~27 hours with 4-second slots
            exit_delay: 256,
            slashing_penalty_multiplier: 3,
            inactivity_penalty_per_epoch: 1000,
        }
    }
}

/// Validator manager for handling all validator operations
pub struct ValidatorManager {
    config: ValidatorConfig,
    state_store: StateStore,
    activation_queue: ActivationQueue,
    exit_queue: ExitQueue,
    balance_tracker: BalanceTracker,
    performance_cache: HashMap<ValidatorIndex, ValidatorPerformance>,
    slashed_validators: HashMap<ValidatorIndex, Epoch>,
}

impl ValidatorManager {
    /// Create new validator manager
    pub fn new(config: ValidatorConfig, state_store: StateStore) -> Self {
        let activation_queue = ActivationQueue::new(
            config.max_validators_per_epoch,
            config.activation_delay,
        );
        let exit_queue = ExitQueue::new(
            config.max_validators_per_epoch,
            config.exit_delay,
        );
        let balance_tracker = BalanceTracker::new(
            config.min_deposit_amount,
            32_000_000_000, // 32 ETH max effective balance
        );

        Self {
            config,
            state_store,
            activation_queue,
            exit_queue,
            balance_tracker,
            performance_cache: HashMap::new(),
            slashed_validators: HashMap::new(),
        }
    }

    /// Add new validator to activation queue
    pub fn add_validator(
        &mut self,
        pubkey: PublicKey,
        withdrawal_credentials: [u8; 32],
        deposit_amount: u64,
    ) -> Result<ValidatorIndex, ValidatorError> {
        // Validate deposit amount
        if deposit_amount < self.config.min_deposit_amount {
            return Err(ValidatorError::InsufficientBalance {
                balance: deposit_amount,
                required: self.config.min_deposit_amount,
            });
        }

        // Create new validator
        let _validator = Validator::new(pubkey, withdrawal_credentials);
        let validator_index = self.get_next_validator_index();

        // Add to activation queue with timestamp
        let entry = ActivationQueueEntry {
            validator_index,
            activation_epoch: self.calculate_activation_epoch(),
            deposit_amount,
            priority: self.calculate_priority(deposit_amount),
            deposit_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.activation_queue.enqueue(entry)?;

        // Initialize balance tracking
        self.balance_tracker.set_balance(validator_index, deposit_amount, 0);

        Ok(validator_index)
    }

    /// Process validator activations for current epoch
    pub fn process_activations(
        &mut self,
        state: &mut BeaconState,
        current_epoch: Epoch,
    ) -> Result<Vec<ValidatorIndex>, ValidatorError> {
        let activations = self.activation_queue.process_activations(current_epoch);
        let mut activated = Vec::new();

        // Activate validators from queue
        for entry in activations {
            self.activate_validator(state, entry.validator_index, current_epoch)?;
            activated.push(entry.validator_index);
        }

        Ok(activated)
    }

    /// Process validator exits for current epoch
    pub fn process_exits(
        &mut self,
        state: &mut BeaconState,
        current_epoch: Epoch,
    ) -> Result<Vec<ValidatorIndex>, ValidatorError> {
        let exits = self.exit_queue.process_exits(current_epoch);
        let mut exited = Vec::new();

        // Process exits from queue
        for entry in exits {
            self.exit_validator(state, entry.validator_index, entry.withdrawal_epoch)?;
            exited.push(entry.validator_index);
        }

        Ok(exited)
    }

    /// Request validator exit
    pub fn request_exit(
        &mut self,
        state: &BeaconState,
        validator_index: ValidatorIndex,
        voluntary: bool,
    ) -> Result<(), ValidatorError> {
        let validator = state.validators.validators.get(validator_index as usize)
            .ok_or(ValidatorError::NotFound(validator_index))?;

        // Calculate exit epoch
        let current_epoch = state.current_epoch();
        let exit_epoch = if voluntary {
            current_epoch + self.config.exit_delay
        } else {
            current_epoch // Immediate for slashing
        };

        let withdrawal_epoch = exit_epoch + self.config.exit_delay;

        let entry = ExitQueueEntry {
            validator_index,
            exit_epoch,
            voluntary,
            withdrawal_epoch,
        };

        self.exit_queue.enqueue(entry)?;
        Ok(())
    }

    /// Apply rewards to validator
    pub fn apply_reward(
        &mut self,
        validator_index: ValidatorIndex,
        reward: u64,
        epoch: Epoch,
    ) -> Result<(), ValidatorError> {
        self.balance_tracker.apply_reward(validator_index, reward, epoch)
    }

    /// Apply penalty to validator
    pub fn apply_penalty(
        &mut self,
        validator_index: ValidatorIndex,
        penalty: u64,
        epoch: Epoch,
    ) -> Result<(), ValidatorError> {
        self.balance_tracker.apply_penalty(validator_index, penalty, epoch)
    }

    /// Get validator balance
    pub fn get_balance(&self, validator_index: ValidatorIndex) -> Option<u64> {
        self.balance_tracker.get_balance(validator_index)
    }

    /// Get validator effective balance
    pub fn get_effective_balance(&self, validator_index: ValidatorIndex) -> Option<u64> {
        self.balance_tracker.get_effective_balance(validator_index)
    }

    /// Exit a specific validator
    fn exit_validator(
        &mut self,
        state: &mut BeaconState,
        validator_index: ValidatorIndex,
        withdrawal_epoch: Epoch,
    ) -> Result<(), ValidatorError> {
        let validator = state.validators.validators.get_mut(validator_index as usize)
            .ok_or(ValidatorError::NotFound(validator_index))?;

        validator.exit_epoch = withdrawal_epoch;
        validator.withdrawable_epoch = withdrawal_epoch + self.config.exit_delay;
        
        Ok(())
    }

    /// Enhanced slashing with proper penalty calculation
    pub fn slash_validator(
        &mut self,
        state: &mut BeaconState,
        validator_index: ValidatorIndex,
        slashing_epoch: Epoch,
        reason: &str,
    ) -> Result<(), ValidatorError> {
        // Check if already slashed
        if self.slashed_validators.contains_key(&validator_index) {
            return Err(ValidatorError::AlreadySlashed(validator_index));
        }

        let validator = state.validators.validators.get_mut(validator_index as usize)
            .ok_or(ValidatorError::NotFound(validator_index))?;

        // Calculate slashing penalty (1/32 of effective balance)
        let effective_balance = self.get_effective_balance(validator_index)
            .unwrap_or(self.config.min_deposit_amount);
        let slashing_penalty = effective_balance / 32;

        // Apply immediate penalty
        self.apply_penalty(validator_index, slashing_penalty, slashing_epoch)?;

        // Mark as slashed
        validator.slashed = true;
        self.slashed_validators.insert(validator_index, slashing_epoch);

        // Force exit
        self.request_exit(state, validator_index, false)?;

        println!("Validator {} slashed at epoch {} for: {}", 
                validator_index, slashing_epoch, reason);

        Ok(())
    }

    /// Check for slashing conditions
    pub fn detect_slashing_conditions(
        &self,
        state: &BeaconState,
        validator_index: ValidatorIndex,
    ) -> Vec<String> {
        let mut violations = Vec::new();

        // Check if validator is active
        let validator = match state.validators.validators.get(validator_index as usize) {
            Some(v) => v,
            None => return violations,
        };

        let current_epoch = state.current_epoch();

        // Double proposal detection (simplified)
        if let Some(performance) = self.performance_cache.get(&validator_index) {
            if performance.block_proposal_success_rate > 1.0 {
                violations.push("Double block proposal detected".to_string());
            }
        }

        // Inactivity detection
        if current_epoch > validator.activation_epoch + 4 {
            if let Some(last_attestation) = self.get_last_attestation_epoch(validator_index) {
                if current_epoch - last_attestation > 4 {
                    violations.push("Extended inactivity detected".to_string());
                }
            }
        }

        violations
    }

    /// Get last attestation epoch for validator
    fn get_last_attestation_epoch(&self, validator_index: ValidatorIndex) -> Option<Epoch> {
        self.performance_cache
            .get(&validator_index)
            .and_then(|perf| {
                // This would be populated from attestation processing
                // For now, return None as placeholder
                None
            })
    }

    /// Update validator performance metrics
    pub fn update_performance(
        &mut self,
        validator_index: ValidatorIndex,
        attestation_success: bool,
        inclusion_delay: u64,
    ) -> Result<(), ValidatorError> {
        let performance = self.performance_cache
            .entry(validator_index)
            .or_insert_with(|| ValidatorPerformance {
                index: validator_index,
                attestation_success_rate: 0.0,
                block_proposal_success_rate: 0.0,
                uptime_percentage: 100.0,
                average_inclusion_delay: 0.0,
                penalties_incurred: 0,
            });

        // Update attestation success rate (simple moving average)
        let weight = 0.1; // Weight for new data
        if attestation_success {
            performance.attestation_success_rate = 
                performance.attestation_success_rate * (1.0 - weight) + weight;
        } else {
            performance.attestation_success_rate *= 1.0 - weight;
        }

        // Update inclusion delay
        performance.average_inclusion_delay = 
            performance.average_inclusion_delay * (1.0 - weight) + 
            inclusion_delay as f64 * weight;

        Ok(())
    }

    /// Get activation queue status
    pub fn get_activation_queue_status(&self) -> (usize, bool) {
        (self.activation_queue.len(), self.activation_queue.is_empty())
    }

    /// Get exit queue status
    pub fn get_exit_queue_status(&self) -> (usize, bool) {
        (self.exit_queue.len(), self.exit_queue.is_empty())
    }

    /// Process validator exits for current epoch
    pub fn process_exits(
        &mut self,
        state: &mut BeaconState,
        current_epoch: Epoch,
    ) -> Result<Vec<ValidatorIndex>, ValidatorError> {
        let mut exited = Vec::new();
        let mut to_exit = Vec::new();

        // Find validators ready for exit
        while let Some(entry) = self.exit_queue.front() {
            if entry.exit_epoch <= current_epoch {
                to_exit.push(self.exit_queue.pop_front().unwrap());
            } else {
                break;
            }
        }

        // Exit validators
        for entry in to_exit {
            self.exit_validator(state, entry.validator_index, current_epoch)?;
            exited.push(entry.validator_index);
        }

        Ok(exited)
    }

    /// Exit a specific validator
    fn exit_validator(
        &self,
        state: &mut BeaconState,
        validator_index: ValidatorIndex,
        exit_epoch: Epoch,
    ) -> Result<(), ValidatorError> {
        let validator = state.validators.validators.get_mut(validator_index as usize)
            .ok_or(ValidatorError::NotFound(validator_index))?;

        validator.exit_epoch = exit_epoch;
        
        Ok(())
    }

    /// Update validator balance (legacy method for compatibility)
    pub fn update_balance(
        &mut self,
        state: &mut BeaconState,
        validator_index: ValidatorIndex,
        new_balance: u64,
    ) -> Result<(), ValidatorError> {
        let balance = state.balances.get_mut(validator_index as usize)
            .ok_or(ValidatorError::InvalidIndex(validator_index))?;
        
        *balance = new_balance;
        
        // Update in balance tracker as well
        self.balance_tracker.set_balance(validator_index, new_balance, state.current_epoch());
        
        Ok(())
    }

    /// Get validator performance metrics
    pub fn get_performance(
        &self,
        validator_index: ValidatorIndex,
    ) -> Option<&ValidatorPerformance> {
        self.performance_cache.get(&validator_index)
    }

    /// Get active validator count
    pub fn get_active_validator_count(
        &self,
        state: &BeaconState,
        epoch: Epoch,
    ) -> usize {
        state.validators.active_at(epoch).len()
    }

    /// Calculate next validator index
    fn get_next_validator_index(&self) -> ValidatorIndex {
        // This would typically be the current validator set size
        // For now, return a placeholder
        0
    }

    /// Calculate activation epoch based on queue and delay
    fn calculate_activation_epoch(&self) -> Epoch {
        // Add activation delay to current epoch
        // For now, return a placeholder
        256
    }

    /// Calculate exit epoch based on queue and delay
    fn calculate_exit_epoch(&self) -> Epoch {
        // Add exit delay to current epoch
        // For now, return a placeholder
        256
    }

    /// Calculate validator priority based on deposit amount
    fn calculate_priority(&self, deposit_amount: u64) -> u64 {
        // Higher deposits get higher priority
        deposit_amount
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{Database, DatabaseConfig};

    fn setup_manager() -> ValidatorManager {
        let config = ValidatorConfig::default();
        let database = Database::in_memory();
        let state_store = StateStore::new(database);
        
        ValidatorManager::new(config, state_store)
    }

    #[test]
    fn test_validator_manager_creation() {
        let manager = setup_manager();
        assert_eq!(manager.config.min_deposit_amount, 1_000_000_000);
    }

    #[test]
    fn test_add_validator() {
        let mut manager = setup_manager();
        let pubkey = vec![1u8; 48];
        let withdrawal_credentials = [2u8; 32];
        let deposit_amount = 2_000_000_000; // 2 ETH
        
        let result = manager.add_validator(pubkey, withdrawal_credentials, deposit_amount);
        assert!(result.is_ok());
    }

    #[test]
    fn test_insufficient_deposit() {
        let mut manager = setup_manager();
        let pubkey = vec![1u8; 48];
        let withdrawal_credentials = [2u8; 32];
        let deposit_amount = 500_000_000; // 0.5 ETH - insufficient
        
        let result = manager.add_validator(pubkey, withdrawal_credentials, deposit_amount);
        assert!(result.is_err());
    }

    #[test]
    fn test_validator_exit_request() {
        let mut manager = setup_manager();
        let state = BeaconState::default();
        let result = manager.request_exit(&state, 0, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_activation_queue() {
        let mut manager = setup_manager();
        let pubkey = PublicKey([1u8; 48]);
        let withdrawal_credentials = [1u8; 32];
        let deposit_amount = 1_000_000_000; // 1 ETH

        // Add validator to activation queue
        let result = manager.add_validator(pubkey, withdrawal_credentials, deposit_amount);
        assert!(result.is_ok());

        let (queue_size, is_empty) = manager.get_activation_queue_status();
        assert_eq!(queue_size, 1);
        assert!(!is_empty);
    }

    #[test]
    fn test_exit_queue() {
        let mut manager = setup_manager();
        let state = BeaconState::default();
        
        // Request exit
        let result = manager.request_exit(&state, 0, true);
        assert!(result.is_ok());

        let (queue_size, is_empty) = manager.get_exit_queue_status();
        assert_eq!(queue_size, 1);
        assert!(!is_empty);
    }

    #[test]
    fn test_balance_tracking() {
        let mut manager = setup_manager();
        let validator_index = 0;
        let initial_balance = 1_000_000_000; // 1 ETH

        // Apply reward
        let result = manager.apply_reward(validator_index, 1000000, 100);
        assert!(result.is_err()); // Validator not found initially

        // Add validator first
        let pubkey = PublicKey([1u8; 48]);
        let withdrawal_credentials = [1u8; 32];
        manager.add_validator(pubkey, withdrawal_credentials, initial_balance).unwrap();

        // Now apply reward
        let result = manager.apply_reward(validator_index, 1000000, 100);
        assert!(result.is_ok());

        let balance = manager.get_balance(validator_index).unwrap();
        assert_eq!(balance, initial_balance + 1000000);
    }

    #[test]
    fn test_slashing_detection() {
        let manager = setup_manager();
        let state = BeaconState::default();
        let validator_index = 0;

        let violations = manager.detect_slashing_conditions(&state, validator_index);
        // Should be empty for default state
        assert!(violations.is_empty());
    }

    #[test]
    fn test_performance_update() {
        let mut manager = setup_manager();
        let validator_index = 0;

        // Update performance
        let result = manager.update_performance(validator_index, true, 2);
        assert!(result.is_ok());

        // Check performance cache
        let performance = manager.get_performance(validator_index);
        assert!(performance.is_some());
    }

    #[test]
    fn test_activation_processing() {
        let mut manager = setup_manager();
        let mut state = BeaconState::default();
        
        // Add validators to queue
        for i in 0..3 {
            let pubkey = PublicKey([i as u8; 48]);
            let withdrawal_credentials = [i as u8; 32];
            let deposit_amount = 1_000_000_000; // 1 ETH
            manager.add_validator(pubkey, withdrawal_credentials, deposit_amount).unwrap();
        }

        // Process activations
        let current_epoch = 257; // After activation delay
        let result = manager.process_activations(&mut state, current_epoch);
        assert!(result.is_ok());
        
        let activated = result.unwrap();
        assert!(!activated.is_empty());
    }

    #[test]
    fn test_exit_processing() {
        let mut manager = setup_manager();
        let mut state = BeaconState::default();
        
        // Request exits
        for i in 0..2 {
            let result = manager.request_exit(&state, i, true);
            assert!(result.is_ok());
        }

        // Process exits
        let current_epoch = 300; // After exit delay
        let result = manager.process_exits(&mut state, current_epoch);
        assert!(result.is_ok());
        
        let exited = result.unwrap();
        assert!(!exited.is_empty());
    }
}
