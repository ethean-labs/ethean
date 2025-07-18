//! Message handler for network messages
//!
//! Processes incoming consensus messages and validates them
//! before passing to the consensus layer.

use super::{NetworkError, NetworkMessage};
use crate::types::{BeaconBlock, Attestation, Slot, Epoch};
use crate::consensus::ValidatorManager;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Message processing result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageResult {
    /// Message processed successfully
    Accepted,
    /// Message rejected due to validation failure
    Rejected(String),
    /// Message deferred for later processing
    Deferred,
    /// Message ignored (duplicate or irrelevant)
    Ignored,
}

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    /// Low priority (status messages)
    Low = 0,
    /// Normal priority (attestations)
    Normal = 1,
    /// High priority (blocks)
    High = 2,
    /// Critical priority (slashing evidence)
    Critical = 3,
}

/// Processed message with metadata
#[derive(Debug, Clone)]
pub struct ProcessedMessage {
    /// Original network message
    pub message: NetworkMessage,
    /// Processing result
    pub result: MessageResult,
    /// Processing priority
    pub priority: MessagePriority,
    /// Processing time
    pub processing_time: Duration,
    /// Sender peer ID
    pub sender: Option<String>,
    /// Timestamp when received
    pub received_at: Instant,
}

impl ProcessedMessage {
    /// Create new processed message
    pub fn new(message: NetworkMessage, sender: Option<String>) -> Self {
        let priority = Self::determine_priority(&message);
        
        Self {
            message,
            result: MessageResult::Deferred,
            priority,
            processing_time: Duration::ZERO,
            sender,
            received_at: Instant::now(),
        }
    }
    
    /// Determine message priority
    fn determine_priority(message: &NetworkMessage) -> MessagePriority {
        match message {
            NetworkMessage::BeaconBlock(_) => MessagePriority::High,
            NetworkMessage::Attestation(_) => MessagePriority::Normal,
            NetworkMessage::BlockRequest { .. } | 
            NetworkMessage::BlockResponse { .. } => MessagePriority::Normal,
            NetworkMessage::StatusRequest | 
            NetworkMessage::StatusResponse { .. } => MessagePriority::Low,
        }
    }
    
    /// Check if message is expired
    pub fn is_expired(&self, max_age: Duration) -> bool {
        self.received_at.elapsed() > max_age
    }
}

/// Message validation context
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Current head slot
    pub head_slot: Slot,
    /// Current finalized epoch
    pub finalized_epoch: Epoch,
    /// Current justified epoch
    pub justified_epoch: Epoch,
    /// Known block roots
    pub known_blocks: HashMap<[u8; 32], Slot>,
    /// Maximum acceptable slot difference
    pub max_slot_drift: u64,
}

impl ValidationContext {
    /// Create new validation context
    pub fn new(head_slot: Slot, finalized_epoch: Epoch, justified_epoch: Epoch) -> Self {
        Self {
            head_slot,
            finalized_epoch,
            justified_epoch,
            known_blocks: HashMap::new(),
            max_slot_drift: 32, // 32 slots in the future
        }
    }
    
    /// Update context with new chain state
    pub fn update(&mut self, head_slot: Slot, finalized_epoch: Epoch, justified_epoch: Epoch) {
        self.head_slot = head_slot;
        self.finalized_epoch = finalized_epoch;
        self.justified_epoch = justified_epoch;
    }
    
    /// Add known block
    pub fn add_known_block(&mut self, block_root: [u8; 32], slot: Slot) {
        self.known_blocks.insert(block_root, slot);
        
        // Limit memory usage
        if self.known_blocks.len() > 1000 {
            // Remove oldest entries (simple cleanup)
            let min_slot = slot.saturating_sub(100);
            self.known_blocks.retain(|_, &mut s| s >= min_slot);
        }
    }
    
    /// Check if block is known
    pub fn is_known_block(&self, block_root: &[u8; 32]) -> bool {
        self.known_blocks.contains_key(block_root)
    }
}

/// Message handler statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageStats {
    pub total_processed: u64,
    pub total_accepted: u64,
    pub total_rejected: u64,
    pub total_deferred: u64,
    pub total_ignored: u64,
    pub average_processing_time_ms: f64,
    pub messages_by_type: HashMap<String, u64>,
    pub messages_by_priority: HashMap<String, u64>,
}

/// Message handler implementation
#[derive(Debug)]
pub struct MessageHandler {
    /// Validator manager for consensus validation
    validator_manager: Arc<ValidatorManager>,
    /// Message processing queue (priority-based)
    high_priority_queue: VecDeque<ProcessedMessage>,
    normal_priority_queue: VecDeque<ProcessedMessage>,
    low_priority_queue: VecDeque<ProcessedMessage>,
    /// Validation context
    validation_context: ValidationContext,
    /// Processing statistics
    stats: MessageStats,
    /// Message deduplication cache
    message_cache: HashMap<String, Instant>,
    /// Maximum queue sizes
    max_queue_size: usize,
    /// Processing timeout
    processing_timeout: Duration,
}

impl MessageHandler {
    /// Create new message handler
    pub fn new(validator_manager: Arc<ValidatorManager>) -> Result<Self, NetworkError> {
        Ok(Self {
            validator_manager,
            high_priority_queue: VecDeque::new(),
            normal_priority_queue: VecDeque::new(),
            low_priority_queue: VecDeque::new(),
            validation_context: ValidationContext::new(0, 0, 0),
            stats: MessageStats::default(),
            message_cache: HashMap::new(),
            max_queue_size: 1000,
            processing_timeout: Duration::from_secs(5),
        })
    }
    
    /// Handle incoming network message
    pub fn handle_message(&mut self, message: NetworkMessage, sender: Option<String>) -> Result<MessageResult, NetworkError> {
        let start_time = Instant::now();
        let mut processed_msg = ProcessedMessage::new(message.clone(), sender);
        
        // Check for duplicates
        let message_id = self.generate_message_id(&message);
        if let Some(&last_seen) = self.message_cache.get(&message_id) {
            if last_seen.elapsed() < Duration::from_secs(60) {
                processed_msg.result = MessageResult::Ignored;
                self.stats.total_ignored += 1;
                return Ok(MessageResult::Ignored);
            }
        }
        
        // Validate message
        let validation_result = self.validate_message(&message)?;
        processed_msg.result = validation_result.clone();
        processed_msg.processing_time = start_time.elapsed();
        
        // Update statistics
        self.update_stats(&message, &validation_result, processed_msg.processing_time);
        
        // Cache message ID
        self.message_cache.insert(message_id, Instant::now());
        
        // Add to appropriate queue based on result and priority
        if validation_result == MessageResult::Accepted || validation_result == MessageResult::Deferred {
            self.enqueue_message(processed_msg)?;
        }
        
        Ok(validation_result)
    }
    
    /// Validate network message
    fn validate_message(&self, message: &NetworkMessage) -> Result<MessageResult, NetworkError> {
        match message {
            NetworkMessage::BeaconBlock(block) => self.validate_beacon_block(block),
            NetworkMessage::Attestation(attestation) => self.validate_attestation(attestation),
            NetworkMessage::BlockRequest { slot, count } => self.validate_block_request(*slot, *count),
            NetworkMessage::BlockResponse { blocks } => self.validate_block_response(blocks),
            NetworkMessage::StatusRequest => Ok(MessageResult::Accepted),
            NetworkMessage::StatusResponse { head_slot, finalized_epoch } => {
                self.validate_status_response(*head_slot, *finalized_epoch)
            },
        }
    }
    
    /// Validate beacon block
    fn validate_beacon_block(&self, block: &BeaconBlock) -> Result<MessageResult, NetworkError> {
        // Check slot is reasonable
        if block.slot > self.validation_context.head_slot + self.validation_context.max_slot_drift {
            return Ok(MessageResult::Rejected("Block slot too far in future".to_string()));
        }
        
        // Generate block root for comparison
        let block_root = self.calculate_block_root(block);
        
        // Check if block is already known
        if self.validation_context.is_known_block(&block_root) {
            return Ok(MessageResult::Ignored);
        }
        
        // Check basic block structure - ensure it has required fields
        if block.body.attestations.is_empty() && block.slot > 0 {
            // Allow empty attestations for genesis or early blocks, but warn for others
        }
        
        // Basic slot validation
        if block.slot == 0 && self.validation_context.head_slot > 0 {
            return Ok(MessageResult::Rejected("Genesis block received after chain start".to_string()));
        }
        
        Ok(MessageResult::Accepted)
    }
    
    /// Validate attestation
    fn validate_attestation(&self, attestation: &Attestation) -> Result<MessageResult, NetworkError> {
        // Check if attestation is for current or recent slot
        let attestation_slot = attestation.data.slot;
        if attestation_slot > self.validation_context.head_slot {
            return Ok(MessageResult::Rejected("Attestation for future slot".to_string()));
        }
        
        // Check if attestation is too old
        if attestation_slot + 32 < self.validation_context.head_slot {
            return Ok(MessageResult::Rejected("Attestation too old".to_string()));
        }
        
        // Check aggregation bits
        if attestation.aggregation_bits.is_empty() {
            return Ok(MessageResult::Rejected("Empty aggregation bits".to_string()));
        }
        
        // Check signature
        if attestation.signature.is_empty() {
            return Ok(MessageResult::Rejected("Missing signature".to_string()));
        }
        
        Ok(MessageResult::Accepted)
    }
    
    /// Validate block request
    fn validate_block_request(&self, slot: Slot, count: u32) -> Result<MessageResult, NetworkError> {
        if count == 0 || count > 1024 {
            return Ok(MessageResult::Rejected("Invalid block count".to_string()));
        }
        
        if slot > self.validation_context.head_slot + 32 {
            return Ok(MessageResult::Rejected("Block request for future slot".to_string()));
        }
        
        Ok(MessageResult::Accepted)
    }
    
    /// Validate block response
    fn validate_block_response(&self, blocks: &[BeaconBlock]) -> Result<MessageResult, NetworkError> {
        if blocks.is_empty() {
            return Ok(MessageResult::Rejected("Empty block response".to_string()));
        }
        
        if blocks.len() > 1024 {
            return Ok(MessageResult::Rejected("Too many blocks in response".to_string()));
        }
        
        // Check blocks are in order
        for window in blocks.windows(2) {
            if window[1].slot <= window[0].slot {
                return Ok(MessageResult::Rejected("Blocks not in slot order".to_string()));
            }
        }
        
        Ok(MessageResult::Accepted)
    }
    
    /// Validate status response
    fn validate_status_response(&self, head_slot: Slot, finalized_epoch: Epoch) -> Result<MessageResult, NetworkError> {
        // Basic sanity checks
        if head_slot == 0 && finalized_epoch > 0 {
            return Ok(MessageResult::Rejected("Invalid status: zero head with non-zero finalized".to_string()));
        }
        
        if finalized_epoch > head_slot / 32 + 1 {
            return Ok(MessageResult::Rejected("Finalized epoch exceeds head".to_string()));
        }
        
        Ok(MessageResult::Accepted)
    }
    
    /// Generate message ID for deduplication
    fn generate_message_id(&self, message: &NetworkMessage) -> String {
        match message {
            NetworkMessage::BeaconBlock(block) => {
                let block_root = self.calculate_block_root(block);
                format!("block_{}_{:?}", block.slot, block_root)
            },
            NetworkMessage::Attestation(att) => {
                format!("att_{}_{}_{:?}", att.data.slot, att.data.index, att.signature)
            },
            NetworkMessage::BlockRequest { slot, count } => {
                format!("req_{}_{}", slot, count)
            },
            NetworkMessage::BlockResponse { blocks } => {
                format!("resp_{}", blocks.len())
            },
            NetworkMessage::StatusRequest => "status_req".to_string(),
            NetworkMessage::StatusResponse { head_slot, finalized_epoch } => {
                format!("status_resp_{}_{}", head_slot, finalized_epoch)
            },
        }
    }
    
    /// Add message to appropriate priority queue
    fn enqueue_message(&mut self, message: ProcessedMessage) -> Result<(), NetworkError> {
        let queue = match message.priority {
            MessagePriority::Critical | MessagePriority::High => &mut self.high_priority_queue,
            MessagePriority::Normal => &mut self.normal_priority_queue,
            MessagePriority::Low => &mut self.low_priority_queue,
        };
        
        if queue.len() >= self.max_queue_size {
            return Err(NetworkError::MessageValidation {
                reason: format!("Queue full for priority {:?}", message.priority)
            });
        }
        
        queue.push_back(message);
        Ok(())
    }
    
    /// Get next message to process (priority order)
    pub fn get_next_message(&mut self) -> Option<ProcessedMessage> {
        self.high_priority_queue.pop_front()
            .or_else(|| self.normal_priority_queue.pop_front())
            .or_else(|| self.low_priority_queue.pop_front())
    }
    
    /// Update validation context
    pub fn update_context(&mut self, head_slot: Slot, finalized_epoch: Epoch, justified_epoch: Epoch) {
        self.validation_context.update(head_slot, finalized_epoch, justified_epoch);
    }
    
    /// Add known block to context
    pub fn add_known_block(&mut self, block_root: [u8; 32], slot: Slot) {
        self.validation_context.add_known_block(block_root, slot);
    }
    
    /// Update processing statistics
    fn update_stats(&mut self, message: &NetworkMessage, result: &MessageResult, processing_time: Duration) {
        self.stats.total_processed += 1;
        
        match result {
            MessageResult::Accepted => self.stats.total_accepted += 1,
            MessageResult::Rejected(_) => self.stats.total_rejected += 1,
            MessageResult::Deferred => self.stats.total_deferred += 1,
            MessageResult::Ignored => self.stats.total_ignored += 1,
        }
        
        // Update average processing time
        let total_time = self.stats.average_processing_time_ms * (self.stats.total_processed - 1) as f64;
        self.stats.average_processing_time_ms = (total_time + processing_time.as_millis() as f64) / self.stats.total_processed as f64;
        
        // Update message type stats
        let message_type = match message {
            NetworkMessage::BeaconBlock(_) => "beacon_block",
            NetworkMessage::Attestation(_) => "attestation",
            NetworkMessage::BlockRequest { .. } => "block_request",
            NetworkMessage::BlockResponse { .. } => "block_response",
            NetworkMessage::StatusRequest => "status_request",
            NetworkMessage::StatusResponse { .. } => "status_response",
        };
        
        *self.stats.messages_by_type.entry(message_type.to_string()).or_insert(0) += 1;
    }
    
    /// Cleanup expired messages and cache entries
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        
        // Remove expired cache entries
        self.message_cache.retain(|_, &mut timestamp| now.duration_since(timestamp) < Duration::from_secs(300));
        
        // Remove expired messages from queues
        let max_age = Duration::from_secs(60);
        self.high_priority_queue.retain(|msg| !msg.is_expired(max_age));
        self.normal_priority_queue.retain(|msg| !msg.is_expired(max_age));
        self.low_priority_queue.retain(|msg| !msg.is_expired(max_age));
    }
    
    /// Calculate block root (simplified implementation)
    fn calculate_block_root(&self, block: &BeaconBlock) -> [u8; 32] {
        // Simplified block root calculation
        // In a real implementation, this would use proper SSZ hashing
        let mut root = [0u8; 32];
        let slot_bytes = block.slot.to_le_bytes();
        root[..8].copy_from_slice(&slot_bytes);
        root[8..16].copy_from_slice(&block.proposer_index.to_le_bytes());
        root[16..].copy_from_slice(&block.parent_root[..16]);
        root
    }
    
    /// Get processing statistics
    pub fn get_stats(&self) -> MessageStats {
        self.stats.clone()
    }
    
    /// Get queue sizes
    pub fn get_queue_sizes(&self) -> (usize, usize, usize) {
        (
            self.high_priority_queue.len(),
            self.normal_priority_queue.len(),
            self.low_priority_queue.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BeaconBlock, block::BeaconBlockBody, Attestation};
    use crate::consensus::validator_management::{ValidatorConfig, ValidatorManager};
    use crate::storage::{StateStore, Database};
    
    fn create_test_validator_manager() -> Arc<ValidatorManager> {
        let config = ValidatorConfig::default();
        let state_store = StateStore::new(Database::in_memory());
        Arc::new(ValidatorManager::new(config, state_store))
    }
    
    fn create_test_block(slot: Slot) -> BeaconBlock {
        BeaconBlock {
            slot,
            proposer_index: 0,
            parent_root: [0u8; 32],
            state_root: [1u8; 32],
            body: BeaconBlockBody {
                randao_reveal: vec![0u8; 96],
                eth1_data: crate::types::block::Eth1Data {
                    deposit_root: [0u8; 32],
                    deposit_count: 0,
                    block_hash: [0u8; 32],
                },
                graffiti: [0u8; 32],
                proposer_slashings: Vec::new(),
                attester_slashings: Vec::new(),
                attestations: Vec::new(),
                deposits: Vec::new(),
                voluntary_exits: Vec::new(),
                sync_aggregate: None,
                execution_payload: None,
            },
            block_root: [slot as u8; 32],
        }
    }
    
    #[test]
    fn test_message_handler_creation() {
        let validator_manager = create_test_validator_manager();
        let handler = MessageHandler::new(validator_manager);
        
        assert!(handler.is_ok());
        let handler = handler.unwrap();
        assert_eq!(handler.stats.total_processed, 0);
    }
    
    #[test]
    fn test_message_priority_determination() {
        let block_msg = NetworkMessage::BeaconBlock(create_test_block(1));
        let processed = ProcessedMessage::new(block_msg, None);
        assert_eq!(processed.priority, MessagePriority::High);
        
        let status_msg = NetworkMessage::StatusRequest;
        let processed = ProcessedMessage::new(status_msg, None);
        assert_eq!(processed.priority, MessagePriority::Low);
    }
    
    #[test]
    fn test_block_validation() {
        let validator_manager = create_test_validator_manager();
        let handler = MessageHandler::new(validator_manager).unwrap();
        
        let block = create_test_block(1);
        let result = handler.validate_beacon_block(&block);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), MessageResult::Accepted);
    }
    
    #[test]
    fn test_block_validation_future_slot() {
        let validator_manager = create_test_validator_manager();
        let handler = MessageHandler::new(validator_manager).unwrap();
        
        let block = create_test_block(100); // Far in future
        let result = handler.validate_beacon_block(&block);
        
        assert!(result.is_ok());
        if let MessageResult::Rejected(reason) = result.unwrap() {
            assert!(reason.contains("future"));
        } else {
            panic!("Expected rejection for future block");
        }
    }
    
    #[test]
    fn test_message_deduplication() {
        let validator_manager = create_test_validator_manager();
        let mut handler = MessageHandler::new(validator_manager).unwrap();
        
        let message = NetworkMessage::StatusRequest;
        
        // First message should be accepted
        let result1 = handler.handle_message(message.clone(), None);
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), MessageResult::Accepted);
        
        // Duplicate message should be ignored
        let result2 = handler.handle_message(message, None);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), MessageResult::Ignored);
    }
    
    #[test]
    fn test_queue_management() {
        let validator_manager = create_test_validator_manager();
        let mut handler = MessageHandler::new(validator_manager).unwrap();
        
        // Add messages with different priorities
        let high_msg = NetworkMessage::BeaconBlock(create_test_block(1));
        let low_msg = NetworkMessage::StatusRequest;
        
        handler.handle_message(low_msg, None).unwrap();
        handler.handle_message(high_msg, None).unwrap();
        
        let (high_size, normal_size, low_size) = handler.get_queue_sizes();
        assert_eq!(high_size, 1); // Block message
        assert_eq!(low_size, 1);  // Status message
        
        // High priority should come first
        let next_msg = handler.get_next_message();
        assert!(next_msg.is_some());
        assert_eq!(next_msg.unwrap().priority, MessagePriority::High);
    }
    
    #[test]
    fn test_validation_context_update() {
        let validator_manager = create_test_validator_manager();
        let mut handler = MessageHandler::new(validator_manager).unwrap();
        
        handler.update_context(100, 3, 3);
        assert_eq!(handler.validation_context.head_slot, 100);
        assert_eq!(handler.validation_context.finalized_epoch, 3);
        
        // Add known block
        let block_root = [1u8; 32];
        handler.add_known_block(block_root, 50);
        assert!(handler.validation_context.is_known_block(&block_root));
    }
}
