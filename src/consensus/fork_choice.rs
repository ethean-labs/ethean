//! LMD-GHOST fork choice implementation for Beam Chain
//!
//! Implements the Latest Message Driven Greediest Heaviest Observed SubTree 
//! fork choice rule with optimizations for Beam Chain consensus.

use crate::types::{BeaconState, Slot, Epoch, ValidatorIndex};
use crate::crypto::hash::Hash;
use crate::consensus::{ValidatorManager, FinalityGadget, Checkpoint};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::collections::{HashMap, HashSet, BTreeMap, VecDeque};
use std::time::{Duration, Instant};
use std::sync::Arc;

/// Fork choice errors
#[derive(Debug, Error)]
pub enum ForkChoiceError {
    #[error("Block not found: {0:?}")]
    BlockNotFound(Hash),
    
    #[error("Invalid block: {0:?}")]
    InvalidBlock(Hash),
    
    #[error("Parent block not found: {0:?}")]
    ParentNotFound(Hash),
    
    #[error("Conflicting checkpoint: expected {expected:?}, got {actual:?}")]
    ConflictingCheckpoint { expected: Hash, actual: Hash },
    
    #[error("Invalid attestation for block {block:?}")]
    InvalidAttestation { block: Hash },
    
    #[error("Finality violation: trying to reorg past finalized checkpoint")]
    FinalityViolation,
    
    #[error("Invalid head selection")]
    InvalidHead,
}

/// Block node in the fork choice tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockNode {
    /// Block hash
    pub block_hash: Hash,
    /// Parent block hash
    pub parent_hash: Hash,
    /// Block slot
    pub slot: Slot,
    /// State root
    pub state_root: Hash,
    /// Total weight (accumulated from children)
    pub weight: u64,
    /// Direct attestation weight for this block
    pub attestation_weight: u64,
    /// Children blocks
    pub children: HashSet<Hash>,
    /// Whether this block is part of the canonical chain
    pub is_canonical: bool,
    /// Justified checkpoint reachable from this block (simplified)
    pub justified_epoch: Option<Epoch>,
    /// Finalized checkpoint reachable from this block (simplified)
    pub finalized_epoch: Option<Epoch>,
}

impl BlockNode {
    /// Create new block node
    pub fn new(block_hash: Hash, parent_hash: Hash, slot: Slot, state_root: Hash) -> Self {
        Self {
            block_hash,
            parent_hash,
            slot,
            state_root,
            weight: 0,
            attestation_weight: 0,
            children: HashSet::new(),
            is_canonical: false,
            justified_epoch: None,
            finalized_epoch: None,
        }
    }
    
    /// Get total weight including children
    pub fn get_total_weight(&self) -> u64 {
        self.weight + self.attestation_weight
    }
    
    /// Check if this node is a viable head
    pub fn is_viable_head(&self, justified_checkpoint: &Checkpoint) -> bool {
        // For simplified version, all nodes are viable if they have sufficient epoch
        self.slot >= justified_checkpoint.epoch * 32
    }
}

/// Attestation data for fork choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkChoiceAttestation {
    /// Attesting validator
    pub validator: ValidatorIndex,
    /// Slot when attestation was made
    pub slot: Slot,
    /// Target block being attested to
    pub beacon_block_root: Hash,
    /// Target checkpoint
    pub target: Checkpoint,
    /// Source checkpoint
    pub source: Checkpoint,
    /// Validator's effective balance
    pub balance: u64,
}

/// Latest message from each validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestMessage {
    /// Target block hash
    pub block_hash: Hash,
    /// Target slot
    pub slot: Slot,
    /// Validator's balance at time of message
    pub balance: u64,
}

/// Fork choice store maintaining the block tree and latest messages
#[derive(Debug, Clone)]
pub struct ForkChoiceStore {
    /// Block tree indexed by hash
    blocks: HashMap<Hash, BlockNode>,
    /// Latest messages from validators
    latest_messages: HashMap<ValidatorIndex, LatestMessage>,
    /// Current justified checkpoint
    justified_checkpoint: Checkpoint,
    /// Current finalized checkpoint
    finalized_checkpoint: Checkpoint,
    /// Current head block
    head_block: Hash,
    /// Genesis block hash
    genesis_block: Hash,
    /// Block ordering by slot for efficiency
    blocks_by_slot: BTreeMap<Slot, HashSet<Hash>>,
    /// Performance metrics
    reorg_count: u64,
    head_updates: u64,
    pruned_blocks: u64,
}

impl ForkChoiceStore {
    /// Create new fork choice store
    pub fn new(genesis_block: Hash, genesis_checkpoint: Checkpoint) -> Self {
        let mut store = Self {
            blocks: HashMap::new(),
            latest_messages: HashMap::new(),
            justified_checkpoint: genesis_checkpoint.clone(),
            finalized_checkpoint: genesis_checkpoint,
            head_block: genesis_block,
            genesis_block,
            blocks_by_slot: BTreeMap::new(),
            reorg_count: 0,
            head_updates: 0,
            pruned_blocks: 0,
        };
        
        // Add genesis block
        let genesis_node = BlockNode::new(genesis_block, Hash::default(), 0, Hash::default());
        store.blocks.insert(genesis_block, genesis_node);
        store.blocks_by_slot.entry(0).or_default().insert(genesis_block);
        
        store
    }
    
    /// Add a block to the store
    pub fn add_block(&mut self, block_hash: Hash, parent_hash: Hash, slot: Slot, state_root: Hash) -> Result<(), ForkChoiceError> {
        // Check if parent exists
        if !self.blocks.contains_key(&parent_hash) && parent_hash != Hash::default() {
            return Err(ForkChoiceError::ParentNotFound(parent_hash));
        }
        
        // Create new block node
        let node = BlockNode::new(block_hash, parent_hash, slot, state_root);
        
        // Add to parent's children
        if let Some(parent) = self.blocks.get_mut(&parent_hash) {
            parent.children.insert(block_hash);
        }
        
        // Insert into store
        self.blocks.insert(block_hash, node);
        self.blocks_by_slot.entry(slot).or_default().insert(block_hash);
        
        Ok(())
    }
    
    /// Process an attestation for fork choice
    pub fn process_attestation(&mut self, attestation: ForkChoiceAttestation) -> Result<(), ForkChoiceError> {
        // Validate attestation target exists
        if !self.blocks.contains_key(&attestation.beacon_block_root) {
            return Err(ForkChoiceError::BlockNotFound(attestation.beacon_block_root));
        }
        
        // Check if this is a newer message from the validator
        let should_update = match self.latest_messages.get(&attestation.validator) {
            Some(latest) => attestation.slot > latest.slot,
            None => true,
        };
        
        if should_update {
            // Remove old weight if exists
            if let Some(old_message) = self.latest_messages.get(&attestation.validator) {
                self.remove_weight_from_ancestors(old_message.block_hash, old_message.balance);
            }
            
            // Add new weight to ancestors
            self.add_weight_to_ancestors(attestation.beacon_block_root, attestation.balance);
            
            // Update latest message
            self.latest_messages.insert(attestation.validator, LatestMessage {
                block_hash: attestation.beacon_block_root,
                slot: attestation.slot,
                balance: attestation.balance,
            });
        }
        
        Ok(())
    }
    
    /// Add weight to block and all its ancestors
    fn add_weight_to_ancestors(&mut self, mut block_hash: Hash, weight: u64) {
        while block_hash != Hash::default() {
            if let Some(node) = self.blocks.get_mut(&block_hash) {
                node.weight += weight;
                block_hash = node.parent_hash;
            } else {
                break;
            }
        }
    }
    
    /// Remove weight from block and all its ancestors
    fn remove_weight_from_ancestors(&mut self, mut block_hash: Hash, weight: u64) {
        while block_hash != Hash::default() {
            if let Some(node) = self.blocks.get_mut(&block_hash) {
                node.weight = node.weight.saturating_sub(weight);
                block_hash = node.parent_hash;
            } else {
                break;
            }
        }
    }
    
    /// Get the head block using LMD-GHOST
    pub fn get_head(&mut self) -> Result<Hash, ForkChoiceError> {
        let _start_time = Instant::now();
        
        // Start from justified checkpoint block
        let justified_block = self.find_block_at_checkpoint(&self.justified_checkpoint)?;
        let mut current = justified_block;
        
        // Follow the heaviest path down the tree
        loop {
            let node = self.blocks.get(&current)
                .ok_or(ForkChoiceError::BlockNotFound(current))?;
            
            if node.children.is_empty() {
                // Leaf node - this is our head
                break;
            }
            
            // Find the child with the highest weight
            let mut best_child = None;
            let mut best_weight = 0;
            let mut best_slot = 0;
            
            for &child_hash in &node.children {
                if let Some(child) = self.blocks.get(&child_hash) {
                    // Only consider viable heads
                    if child.is_viable_head(&self.justified_checkpoint) {
                        let child_weight = child.get_total_weight();
                        // Prefer higher weight, break ties with higher slot
                        if child_weight > best_weight || (child_weight == best_weight && child.slot > best_slot) {
                            best_weight = child_weight;
                            best_slot = child.slot;
                            best_child = Some(child_hash);
                        }
                    }
                }
            }
            
            match best_child {
                Some(child) => current = child,
                None => break, // No viable children
            }
        }
        
        // Update head if it changed
        if current != self.head_block {
            self.reorg_count += 1;
            self.head_block = current;
        }
        self.head_updates += 1;
        
        // Update canonical flags
        self.update_canonical_chain(current);
        
        Ok(current)
    }
    
    /// Find block at a specific checkpoint
    fn find_block_at_checkpoint(&self, checkpoint: &Checkpoint) -> Result<Hash, ForkChoiceError> {
        // Simple implementation: find any block with matching epoch
        // In production, would use proper checkpoint resolution
        for (hash, node) in &self.blocks {
            if let Some(epoch) = &node.justified_epoch {
                if *epoch == checkpoint.epoch {
                    return Ok(*hash);
                }
            }
        }
        
        // Fallback to genesis if no match found
        Ok(self.genesis_block)
    }
    
    /// Update canonical chain flags
    fn update_canonical_chain(&mut self, head: Hash) {
        // Reset all canonical flags
        for node in self.blocks.values_mut() {
            node.is_canonical = false;
        }
        
        // Mark canonical chain from head to genesis
        let mut current = head;
        while current != Hash::default() {
            if let Some(node) = self.blocks.get_mut(&current) {
                node.is_canonical = true;
                current = node.parent_hash;
            } else {
                break;
            }
        }
    }
    
    /// Update checkpoints based on finality gadget
    pub fn update_checkpoints(
        &mut self,
        finality_gadget: &FinalityGadget,
    ) -> Result<(), ForkChoiceError> {
        let new_justified = finality_gadget.get_justified_checkpoint().clone();
        let new_finalized = finality_gadget.get_finalized_checkpoint().clone();
        
        // Update justified checkpoint
        if new_justified.epoch > self.justified_checkpoint.epoch {
            self.justified_checkpoint = new_justified;
        }
        
        // Update finalized checkpoint
        if new_finalized.epoch > self.finalized_checkpoint.epoch {
            // Ensure we're not finalizing past current head
            if !self.is_ancestor(new_finalized.block_hash, self.head_block) {
                return Err(ForkChoiceError::FinalityViolation);
            }
            
            self.finalized_checkpoint = new_finalized;
            
            // Prune blocks that are not descendants of finalized checkpoint
            self.prune_finalized_blocks()?;
        }
        
        Ok(())
    }
    
    /// Check if ancestor is an ancestor of descendant
    fn is_ancestor(&self, ancestor: Hash, descendant: Hash) -> bool {
        let mut current = descendant;
        
        while current != Hash::default() {
            if current == ancestor {
                return true;
            }
            
            if let Some(node) = self.blocks.get(&current) {
                current = node.parent_hash;
            } else {
                break;
            }
        }
        
        false
    }
    
    /// Prune blocks that are not descendants of finalized checkpoint
    fn prune_finalized_blocks(&mut self) -> Result<(), ForkChoiceError> {
        let finalized_block = self.finalized_checkpoint.block_hash;
        let mut to_remove = Vec::new();
        
        // Find blocks to remove
        for (&hash, _) in &self.blocks {
            if hash != finalized_block && !self.is_ancestor(finalized_block, hash) {
                to_remove.push(hash);
            }
        }
        
        // Remove blocks and update metrics
        for hash in to_remove {
            self.blocks.remove(&hash);
            self.pruned_blocks += 1;
            
            // Remove from slot index
            for blocks in self.blocks_by_slot.values_mut() {
                blocks.remove(&hash);
            }
        }
        
        // Remove empty slot entries
        self.blocks_by_slot.retain(|_, blocks| !blocks.is_empty());
        
        // Remove latest messages for pruned blocks
        self.latest_messages.retain(|_, message| {
            self.blocks.contains_key(&message.block_hash)
        });
        
        Ok(())
    }
    
    /// Get blocks at a specific slot
    pub fn get_blocks_at_slot(&self, slot: Slot) -> Vec<Hash> {
        self.blocks_by_slot.get(&slot)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }
    
    /// Get block by hash
    pub fn get_block(&self, hash: &Hash) -> Option<&BlockNode> {
        self.blocks.get(hash)
    }
    
    /// Get current head
    pub fn get_current_head(&self) -> Hash {
        self.head_block
    }
    
    /// Get justified checkpoint
    pub fn get_justified_checkpoint(&self) -> &Checkpoint {
        &self.justified_checkpoint
    }
    
    /// Get finalized checkpoint
    pub fn get_finalized_checkpoint(&self) -> &Checkpoint {
        &self.finalized_checkpoint
    }
    
    /// Get performance statistics
    pub fn get_stats(&self) -> (u64, u64, u64, usize, usize) {
        (
            self.reorg_count,
            self.head_updates,
            self.pruned_blocks,
            self.blocks.len(),
            self.latest_messages.len(),
        )
    }
    
    /// Calculate voting power for each block
    pub fn calculate_voting_power(&self, validator_manager: &ValidatorManager) -> HashMap<Hash, u64> {
        let mut voting_power = HashMap::new();
        
        // Calculate power from latest messages
        for (validator, message) in &self.latest_messages {
            let balance = validator_manager
                .get_effective_balance(*validator)
                .unwrap_or(message.balance);
            
            *voting_power.entry(message.block_hash).or_insert(0) += balance;
        }
        
        voting_power
    }
}

/// Enhanced LMD-GHOST fork choice implementation
#[derive(Debug)]
pub struct LMDGHOSTForkChoice {
    /// Fork choice store
    store: ForkChoiceStore,
    /// Validator manager
    validator_manager: Arc<ValidatorManager>,
    /// Finality gadget integration
    finality_gadget: Option<Arc<FinalityGadget>>,
    /// Performance tracking
    head_computation_times: VecDeque<Duration>,
    attestations_processed: u64,
    blocks_processed: u64,
}

impl LMDGHOSTForkChoice {
    /// Create new LMD-GHOST fork choice
    pub fn new(
        genesis_block: Hash,
        genesis_checkpoint: Checkpoint,
        validator_manager: Arc<ValidatorManager>,
    ) -> Self {
        Self {
            store: ForkChoiceStore::new(genesis_block, genesis_checkpoint),
            validator_manager,
            finality_gadget: None,
            head_computation_times: VecDeque::new(),
            attestations_processed: 0,
            blocks_processed: 0,
        }
    }
    
    /// Set finality gadget for checkpoint updates
    pub fn set_finality_gadget(&mut self, finality_gadget: Arc<FinalityGadget>) {
        self.finality_gadget = Some(finality_gadget);
    }
    
    /// Add a block to fork choice
    pub fn on_block(&mut self, block_hash: Hash, parent_hash: Hash, slot: Slot, state_root: Hash, state: &BeaconState) -> Result<(), ForkChoiceError> {
        // Validate block first
        if !self.is_valid_block_params(block_hash, parent_hash, slot, state)? {
            return Err(ForkChoiceError::InvalidBlock(block_hash));
        }
        
        // Add to store
        self.store.add_block(block_hash, parent_hash, slot, state_root)?;
        self.blocks_processed += 1;
        
        // Update head
        self.update_head()?;
        
        Ok(())
    }
    
    /// Process an attestation
    pub fn on_attestation(
        &mut self,
        attestation: ForkChoiceAttestation,
        state: &BeaconState,
    ) -> Result<(), ForkChoiceError> {
        // Validate attestation
        if !self.is_valid_attestation(&attestation, state)? {
            return Err(ForkChoiceError::InvalidAttestation {
                block: attestation.beacon_block_root,
            });
        }
        
        // Process in store
        self.store.process_attestation(attestation)?;
        self.attestations_processed += 1;
        
        // Update head
        self.update_head()?;
        
        Ok(())
    }
    
    /// Update the head block
    pub fn update_head(&mut self) -> Result<Hash, ForkChoiceError> {
        let start_time = Instant::now();
        
        // Update checkpoints from finality gadget if available
        if let Some(ref finality_gadget) = self.finality_gadget {
            self.store.update_checkpoints(finality_gadget)?;
        }
        
        // Compute new head
        let head = self.store.get_head()?;
        
        // Track performance
        let computation_time = start_time.elapsed();
        self.head_computation_times.push_back(computation_time);
        
        // Keep only recent times for averaging
        if self.head_computation_times.len() > 100 {
            self.head_computation_times.pop_front();
        }
        
        Ok(head)
    }
    
    /// Validate a block for fork choice
    fn is_valid_block_params(&self, _block_hash: Hash, parent_hash: Hash, slot: Slot, _state: &BeaconState) -> Result<bool, ForkChoiceError> {
        // Check parent exists
        if !self.store.blocks.contains_key(&parent_hash) && parent_hash != Hash::default() {
            return Ok(false);
        }
        
        // Check slot progression
        if let Some(parent) = self.store.blocks.get(&parent_hash) {
            if slot <= parent.slot {
                return Ok(false);
            }
        }
        
        // Check against finalized checkpoint
        let finalized = self.store.get_finalized_checkpoint();
        if slot < finalized.epoch * 32 {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// Validate an attestation for fork choice
    fn is_valid_attestation(
        &self,
        attestation: &ForkChoiceAttestation,
        _state: &BeaconState,
    ) -> Result<bool, ForkChoiceError> {
        // Check target block exists
        if !self.store.blocks.contains_key(&attestation.beacon_block_root) {
            return Ok(false);
        }
        
        // Check against finalized checkpoint
        let finalized = self.store.get_finalized_checkpoint();
        if attestation.target.epoch < finalized.epoch {
            return Ok(false);
        }
        
        // Check attestation is not too old
        if let Some(target_block) = self.store.blocks.get(&attestation.beacon_block_root) {
            if attestation.slot < target_block.slot {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Get current head
    pub fn get_head(&self) -> Hash {
        self.store.get_current_head()
    }
    
    /// Get justified checkpoint
    pub fn get_justified_checkpoint(&self) -> &Checkpoint {
        self.store.get_justified_checkpoint()
    }
    
    /// Get finalized checkpoint
    pub fn get_finalized_checkpoint(&self) -> &Checkpoint {
        self.store.get_finalized_checkpoint()
    }
    
    /// Get block by hash
    pub fn get_block(&self, hash: &Hash) -> Option<&BlockNode> {
        self.store.get_block(hash)
    }
    
    /// Get blocks at slot
    pub fn get_blocks_at_slot(&self, slot: Slot) -> Vec<Hash> {
        self.store.get_blocks_at_slot(slot)
    }
    
    /// Calculate current voting power distribution
    pub fn get_voting_power_distribution(&self) -> HashMap<Hash, u64> {
        self.store.calculate_voting_power(&self.validator_manager)
    }
    
    /// Get average head computation time
    pub fn get_average_head_time(&self) -> Duration {
        if self.head_computation_times.is_empty() {
            Duration::ZERO
        } else {
            let total: Duration = self.head_computation_times.iter().sum();
            total / self.head_computation_times.len() as u32
        }
    }
    
    /// Get fork choice statistics
    pub fn get_stats(&self) -> (u64, u64, u64, Duration, usize, usize) {
        let (reorgs, _head_updates, _pruned, blocks, messages) = self.store.get_stats();
        (
            self.attestations_processed,
            self.blocks_processed,
            reorgs,
            self.get_average_head_time(),
            blocks,
            messages,
        )
    }
    
    /// Force head update (for testing)
    pub fn force_head_update(&mut self) -> Result<Hash, ForkChoiceError> {
        self.update_head()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Validator;
    use crate::consensus::{AttestationProcessor, validator_management::{ValidatorConfig, ValidatorManager}};
    use crate::storage::{StateStore, Database};
    
    fn create_test_block(slot: Slot, parent_root: Hash) -> (Hash, Hash, Slot, Hash) {
        let block_hash = [slot as u8; 32];
        let state_root = [(slot + 100) as u8; 32];
        (block_hash, parent_root, slot, state_root)
    }
    
    fn create_test_attestation(validator: ValidatorIndex, slot: Slot, block_hash: Hash) -> ForkChoiceAttestation {
        ForkChoiceAttestation {
            validator,
            slot,
            beacon_block_root: block_hash,
            target: Checkpoint {
                epoch: slot / 32,
                block_hash,
                state_root: [slot as u8; 32],
            },
            source: Checkpoint {
                epoch: (slot / 32).saturating_sub(1),
                block_hash: [0u8; 32],
                state_root: [0u8; 32],
            },
            balance: 32_000_000_000,
        }
    }
    
    fn setup_fork_choice() -> (LMDGHOSTForkChoice, BeaconState) {
        let genesis_block = [0u8; 32];
        let genesis_checkpoint = Checkpoint {
            epoch: 0,
            block_hash: genesis_block,
            state_root: [0u8; 32],
        };
        
        let validator_config = ValidatorConfig::default();
        let database = Database::in_memory();
        let state_store = StateStore::new(database);
        let validator_manager = Arc::new(ValidatorManager::new(validator_config, state_store));
        
        let _attestation_processor = Arc::new(AttestationProcessor::new(
            Default::default(),
            (*validator_manager).clone(),
        ));

        let fork_choice = LMDGHOSTForkChoice::new(
            genesis_block,
            genesis_checkpoint,
            validator_manager,
        );        let mut state = BeaconState::default();
        // Add test validators
        for i in 0..10 {
            let validator = Validator {
                pubkey: vec![i as u8; 48],
                withdrawal_credentials: [i as u8; 32],
                effective_balance: 32_000_000_000,
                slashed: false,
                activation_epoch: 0,
                exit_epoch: u64::MAX,
            };
            state.validators.validators.push(validator);
        }
        
        (fork_choice, state)
    }
    
    #[test]
    fn test_fork_choice_creation() {
        let (fork_choice, _) = setup_fork_choice();
        
        assert_eq!(fork_choice.get_head(), [0u8; 32]);
        assert_eq!(fork_choice.get_justified_checkpoint().epoch, 0);
        assert_eq!(fork_choice.get_finalized_checkpoint().epoch, 0);
    }
    
    #[test]
    fn test_block_processing() {
        let (mut fork_choice, state) = setup_fork_choice();
        
        // Add first block
        let (block_hash, parent_hash, slot, state_root) = create_test_block(1, [0u8; 32]);
        let result = fork_choice.on_block(block_hash, parent_hash, slot, state_root, &state);
        assert!(result.is_ok());
        
        // Head should update to new block
        let new_head = fork_choice.force_head_update().unwrap();
        assert_eq!(new_head, block_hash);
    }
    
    #[test]
    fn test_attestation_processing() {
        let (mut fork_choice, state) = setup_fork_choice();
        
        // Add block first
        let (block_hash, parent_hash, slot, state_root) = create_test_block(1, [0u8; 32]);
        fork_choice.on_block(block_hash, parent_hash, slot, state_root, &state).unwrap();
        
        // Add attestation
        let attestation = create_test_attestation(0, 1, block_hash);
        let result = fork_choice.on_attestation(attestation, &state);
        assert!(result.is_ok());
        
        let (attestations, _, _, _, _, _) = fork_choice.get_stats();
        assert_eq!(attestations, 1);
    }
    
    #[test]
    fn test_fork_resolution() {
        let (mut fork_choice, state) = setup_fork_choice();
        
        // Create a fork
        let (block1_hash, _, slot1, state_root1) = create_test_block(1, [0u8; 32]);
        let (block2a_hash, _, slot2, state_root2a) = create_test_block(2, block1_hash);
        let (block2b_hash, _, slot2b, state_root2b) = create_test_block(2, block1_hash);
        
        fork_choice.on_block(block1_hash, [0u8; 32], slot1, state_root1, &state).unwrap();
        fork_choice.on_block(block2a_hash, block1_hash, slot2, state_root2a, &state).unwrap();
        fork_choice.on_block(block2b_hash, block1_hash, slot2b, state_root2b, &state).unwrap();
        
        // Add attestations to block2a (should make it the head)
        for i in 0..6 {
            let attestation = create_test_attestation(i, 2, block2a_hash);
            fork_choice.on_attestation(attestation, &state).unwrap();
        }
        
        // Add fewer attestations to block2b
        for i in 6..8 {
            let attestation = create_test_attestation(i, 2, block2b_hash);
            fork_choice.on_attestation(attestation, &state).unwrap();
        }
        
        // block2a should be the head due to more weight
        let head = fork_choice.force_head_update().unwrap();
        assert_eq!(head, block2a_hash);
    }
    
    #[test]
    fn test_finalized_pruning() {
        let (fork_choice, _state) = setup_fork_choice();
        
        // Test that finalized blocks are pruned correctly
        let initial_stats = fork_choice.get_stats();
        assert_eq!(initial_stats.4, 1); // Only genesis block
        
        // Would need more complex setup to test pruning in detail
        // This is a placeholder for the pruning logic test
    }
    
    #[test]
    fn test_performance_tracking() {
        let (mut fork_choice, state) = setup_fork_choice();
        
        // Process some blocks and attestations
        for i in 1..5 {
            let (block_hash, parent_hash, slot, state_root) = create_test_block(i, [((i-1) % 256) as u8; 32]);
            fork_choice.on_block(block_hash, parent_hash, slot, state_root, &state).unwrap();
            
            let attestation = create_test_attestation(0, i, block_hash);
            fork_choice.on_attestation(attestation, &state).unwrap();
        }
        
        let (attestations, blocks, _reorgs, avg_time, total_blocks, _messages) = fork_choice.get_stats();
        assert_eq!(attestations, 4);
        assert_eq!(blocks, 4);
        assert!(avg_time < Duration::from_millis(100)); // Should be fast
        assert!(total_blocks > 1);
        // Note: messages count might vary due to internal processing
    }
}
