//! Block processing pipeline for Beam Chain consensus
//!
//! Handles block validation, processing, and storage integration.

use crate::types::{BeaconState, BeaconBlock, Slot, Epoch};
use crate::storage::{StateStore, BlockStore, StateStorage, BlockStorage, DatabaseError, Database};
use crate::consensus::state_transition::{StateTransitionProcessor, StateTransitionError};
use crate::crypto::hash::sha256;
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// Block processing errors
#[derive(Debug, Error)]
pub enum BlockProcessingError {
    #[error("State transition error: {0}")]
    StateTransition(#[from] StateTransitionError),
    
    #[error("Storage error: {0}")]
    Storage(#[from] DatabaseError),
    
    #[error("Block not found: {0:?}")]
    BlockNotFound([u8; 32]),
    
    #[error("State not found: {0:?}")]
    StateNotFound([u8; 32]),
    
    #[error("Invalid block: {0}")]
    InvalidBlock(String),
    
    #[error("Processing failed: {0}")]
    ProcessingFailed(String),
}

/// Block processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockProcessingResult {
    /// Block hash that was processed
    pub block_hash: [u8; 32],
    /// State root after processing
    pub state_root: [u8; 32],
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Whether block was successfully processed
    pub success: bool,
    /// Number of attestations processed
    pub attestations_processed: usize,
}

/// Block processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockProcessingConfig {
    /// Maximum attestations per block
    pub max_attestations: usize,
    /// Enable validation caching
    pub enable_caching: bool,
    /// Parallel processing enabled
    pub parallel_processing: bool,
    /// Block processing timeout (seconds)
    pub timeout_seconds: u64,
}

impl Default for BlockProcessingConfig {
    fn default() -> Self {
        Self {
            max_attestations: 128,
            enable_caching: true,
            parallel_processing: false,
            timeout_seconds: 4,
        }
    }
}

/// Block processor for handling block processing pipeline
pub struct BlockProcessor {
    config: BlockProcessingConfig,
    state_transition: StateTransitionProcessor,
    state_store: StateStore,
    block_store: BlockStore,
}

impl BlockProcessor {
    /// Create new block processor
    pub fn new(
        config: BlockProcessingConfig,
        state_transition: StateTransitionProcessor,
        state_store: StateStore,
        block_store: BlockStore,
    ) -> Self {
        Self {
            config,
            state_transition,
            state_store,
            block_store,
        }
    }

    /// Process incoming block
    pub fn process_block(
        &self,
        block: &BeaconBlock,
    ) -> Result<BlockProcessingResult, BlockProcessingError> {
        let start_time = std::time::Instant::now();
        let block_hash = block.hash();

        // Get parent state
        let mut state = self.get_parent_state(block)?;

        // Process block through state transition
        self.state_transition.process_block(&mut state, block)?;

        // Calculate new state root
        let state_root = self.calculate_state_root(&state)?;

        // Store updated state
        self.state_store.store_state(&state_root, &state)?;

        // Store block
        self.block_store.store_block(&block_hash, block)?;

        // Update head if this is the new head
        self.update_head_if_needed(&block_hash, &state)?;

        // Return processing result
        Ok(BlockProcessingResult {
            block_hash,
            state_root,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            success: true,
            attestations_processed: block.body.attestations.len(),
        })
    }

    /// Get parent state for block processing
    fn get_parent_state(&self, block: &BeaconBlock) -> Result<BeaconState, BlockProcessingError> {
        // If this is slot 0, use genesis state
        if block.slot == 0 {
            return Ok(BeaconState::default());
        }

        // Try to get state by parent root
        if let Some(state) = self.state_store.get_state(&block.parent_root)? {
            return Ok(state);
        }

        // Try to get parent block and its state
        if let Some(parent_block) = self.block_store.get_block(&block.parent_root)? {
            if let Some(state) = self.state_store.get_state(&parent_block.state_root)? {
                return Ok(state);
            }
        }

        Err(BlockProcessingError::StateNotFound(block.parent_root))
    }

    /// Calculate state root from state
    fn calculate_state_root(&self, state: &BeaconState) -> Result<[u8; 32], BlockProcessingError> {
        // For now, use simple hash of serialized state
        let serialized = serde_json::to_vec(state)
            .map_err(|e| BlockProcessingError::ProcessingFailed(e.to_string()))?;
        Ok(sha256(&serialized))
    }

    /// Update head block if this is the new best block
    fn update_head_if_needed(
        &self,
        block_hash: &[u8; 32],
        state: &BeaconState,
    ) -> Result<(), BlockProcessingError> {
        // Get current head
        let current_head = self.block_store.get_latest_block()?;

        // If no head or this block is later, update head
        let should_update = match current_head {
            None => true,
            Some(head) => state.slot > head.slot,
        };

        if should_update {
            self.block_store.set_head_block(block_hash)?;
        }

        Ok(())
    }

    /// Validate block before processing
    pub fn validate_block(&self, block: &BeaconBlock) -> Result<(), BlockProcessingError> {
        // Basic block validation
        if block.body.attestations.len() > self.config.max_attestations {
            return Err(BlockProcessingError::InvalidBlock(
                format!("Too many attestations: {}", block.body.attestations.len())
            ));
        }

        // Check block structure
        if block.slot == 0 && block.parent_root != [0u8; 32] {
            return Err(BlockProcessingError::InvalidBlock(
                "Genesis block must have zero parent root".to_string()
            ));
        }

        Ok(())
    }

    /// Get processing statistics
    pub fn get_processing_stats(&self) -> Result<ProcessingStats, BlockProcessingError> {
        let latest_block = self.block_store.get_latest_block()?;
        let latest_state = if let Some(ref block) = latest_block {
            self.state_store.get_state(&block.state_root)?
        } else {
            None
        };

        Ok(ProcessingStats {
            latest_slot: latest_block.map(|b| b.slot).unwrap_or(0),
            latest_epoch: latest_state.as_ref().map(|s| s.current_epoch(32)).unwrap_or(0),
            total_validators: latest_state.as_ref().map(|s| s.validators.validators.len()).unwrap_or(0),
        })
    }
}

/// Processing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStats {
    pub latest_slot: Slot,
    pub latest_epoch: Epoch,
    pub total_validators: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::state_transition::{StateTransitionProcessor, StateTransitionConfig};

    fn setup_processor() -> BlockProcessor {
        let database = Database::in_memory();
        let state_store = StateStore::new(database.clone());
        let block_store = BlockStore::new(database);
        
        let state_transition_config = StateTransitionConfig::default();
        let state_transition = StateTransitionProcessor::new(
            state_transition_config,
            state_store.clone(),
            block_store.clone(),
        );
        
        let config = BlockProcessingConfig::default();
        BlockProcessor::new(config, state_transition, state_store, block_store)
    }

    #[test]
    fn test_block_processor_creation() {
        let processor = setup_processor();
        assert_eq!(processor.config.max_attestations, 128);
    }

    #[test]
    fn test_block_validation() {
        let processor = setup_processor();
        let block = BeaconBlock::default();
        
        let result = processor.validate_block(&block);
        assert!(result.is_ok());
    }

    #[test]
    fn test_genesis_block_processing() {
        let processor = setup_processor();
        let mut block = BeaconBlock::default();
        block.slot = 0;
        block.parent_root = [0u8; 32];
        
        let result = processor.process_block(&block);
        match &result {
            Ok(_) => {},
            Err(e) => println!("Processing error: {:?}", e),
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_processing_stats() {
        let processor = setup_processor();
        let stats = processor.get_processing_stats();
        assert!(stats.is_ok());
    }
}
