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

        // Add to activation queue
        let entry = ActivationQueueEntry {
            validator_index,
            activation_epoch: self.calculate_activation_epoch(),
            deposit_amount,
            priority: self.calculate_priority(deposit_amount),
        };

        self.activation_queue.push_back(entry);

        Ok(validator_index)
    }

    /// Process validator activations for current epoch
    pub fn process_activations(
        &mut self,
        state: &mut BeaconState,
        current_epoch: Epoch,
    ) -> Result<Vec<ValidatorIndex>, ValidatorError> {
        let mut activated = Vec::new();
        let mut to_activate = Vec::new();

        // Find validators ready for activation
        while let Some(entry) = self.activation_queue.front() {
            if entry.activation_epoch <= current_epoch && to_activate.len() < self.config.max_validators_per_epoch as usize {
                to_activate.push(self.activation_queue.pop_front().unwrap());
            } else {
                break;
            }
        }

        // Activate validators
        for entry in to_activate {
            self.activate_validator(state, entry.validator_index, current_epoch)?;
            activated.push(entry.validator_index);
        }

        Ok(activated)
    }

    /// Activate a specific validator
    fn activate_validator(
        &self,
        state: &mut BeaconState,
        validator_index: ValidatorIndex,
        activation_epoch: Epoch,
    ) -> Result<(), ValidatorError> {
        let validator = state.validators.validators.get_mut(validator_index as usize)
            .ok_or(ValidatorError::NotFound(validator_index))?;

        validator.activation_epoch = activation_epoch;
        
        Ok(())
    }

    /// Request validator exit
    pub fn request_exit(
        &mut self,
        validator_index: ValidatorIndex,
        voluntary: bool,
    ) -> Result<(), ValidatorError> {
        let exit_epoch = self.calculate_exit_epoch();
        
        let entry = ExitQueueEntry {
            validator_index,
            exit_epoch,
            voluntary,
        };

        self.exit_queue.push_back(entry);
        Ok(())
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

    /// Slash validator for misconduct
    pub fn slash_validator(
        &mut self,
        state: &mut BeaconState,
        validator_index: ValidatorIndex,
        _slashing_epoch: Epoch,
    ) -> Result<u64, ValidatorError> {
        let validator = state.validators.validators.get_mut(validator_index as usize)
            .ok_or(ValidatorError::NotFound(validator_index))?;

        if validator.slashed {
            return Err(ValidatorError::AlreadySlashed(validator_index));
        }

        validator.slashed = true;
        
        // Calculate slashing penalty
        let penalty = validator.effective_balance / self.config.slashing_penalty_multiplier;
        
        // Reduce balance
        let balance = state.balances.get_mut(validator_index as usize)
            .ok_or(ValidatorError::InvalidIndex(validator_index))?;
        
        *balance = balance.saturating_sub(penalty);

        // Force exit
        self.request_exit(validator_index, false)?;

        Ok(penalty)
    }

    /// Update validator balance
    pub fn update_balance(
        &self,
        state: &mut BeaconState,
        validator_index: ValidatorIndex,
        new_balance: u64,
    ) -> Result<(), ValidatorError> {
        let balance = state.balances.get_mut(validator_index as usize)
            .ok_or(ValidatorError::InvalidIndex(validator_index))?;
        
        *balance = new_balance;
        
        Ok(())
    }

    /// Get validator performance metrics
    pub fn get_performance(
        &self,
        validator_index: ValidatorIndex,
    ) -> Option<&ValidatorPerformance> {
        self.performance_cache.get(&validator_index)
    }

    /// Update validator performance
    pub fn update_performance(
        &mut self,
        validator_index: ValidatorIndex,
        performance: ValidatorPerformance,
    ) {
        self.performance_cache.insert(validator_index, performance);
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
        let result = manager.request_exit(0, true);
        assert!(result.is_ok());
    }
}
