//! Message handler for Lean network messages (thin Phase 03 surface).

use super::{NetworkError, NetworkMessage};
use ethean_types::{Attestation, Block, Epoch, Slot};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::consensus::ValidatorManager;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageResult {
    Accepted,
    Rejected(String),
    Deferred,
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Debug, Clone)]
pub struct ProcessedMessage {
    pub message: NetworkMessage,
    pub result: MessageResult,
    pub priority: MessagePriority,
    pub processing_time: Duration,
    pub sender: Option<String>,
    pub received_at: Instant,
}

impl ProcessedMessage {
    pub fn new(message: NetworkMessage, sender: Option<String>) -> Self {
        let priority = match &message {
            NetworkMessage::Block(_) => MessagePriority::High,
            NetworkMessage::Attestation(_) => MessagePriority::Normal,
            NetworkMessage::BlockRequest { .. } | NetworkMessage::BlockResponse { .. } => {
                MessagePriority::Normal
            }
            NetworkMessage::StatusRequest | NetworkMessage::StatusResponse { .. } => {
                MessagePriority::Low
            }
        };
        Self {
            message,
            result: MessageResult::Deferred,
            priority,
            processing_time: Duration::ZERO,
            sender,
            received_at: Instant::now(),
        }
    }

    pub fn is_expired(&self, max_age: Duration) -> bool {
        self.received_at.elapsed() > max_age
    }
}

#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub head_slot: Slot,
    pub finalized_slot: Slot,
    pub justified_slot: Slot,
    pub known_blocks: HashMap<[u8; 32], Slot>,
}

pub struct MessageHandler {
    _validator_manager: Arc<ValidatorManager>,
    queue: VecDeque<ProcessedMessage>,
    context: ValidationContext,
}

impl MessageHandler {
    pub fn new(validator_manager: Arc<ValidatorManager>) -> Self {
        Self {
            _validator_manager: validator_manager,
            queue: VecDeque::new(),
            context: ValidationContext {
                head_slot: Slot::ZERO,
                finalized_slot: Slot::ZERO,
                justified_slot: Slot::ZERO,
                known_blocks: HashMap::new(),
            },
        }
    }

    pub fn handle(&mut self, message: NetworkMessage) -> Result<MessageResult, NetworkError> {
        match &message {
            NetworkMessage::Block(block) => self.validate_block(block),
            NetworkMessage::Attestation(att) => self.validate_attestation(att),
            NetworkMessage::BlockResponse { blocks } => self.validate_block_response(blocks),
            _ => Ok(MessageResult::Accepted),
        }
    }

    fn validate_block(&self, block: &Block) -> Result<MessageResult, NetworkError> {
        if block.slot.get() + 32 < self.context.head_slot.get() {
            return Ok(MessageResult::Rejected("block too old".into()));
        }
        Ok(MessageResult::Accepted)
    }

    fn validate_attestation(&self, _attestation: &Attestation) -> Result<MessageResult, NetworkError> {
        Ok(MessageResult::Accepted)
    }

    fn validate_block_response(&self, blocks: &[Block]) -> Result<MessageResult, NetworkError> {
        for block in blocks {
            self.validate_block(block)?;
        }
        Ok(MessageResult::Accepted)
    }

    pub fn enqueue(&mut self, message: NetworkMessage, sender: Option<String>) {
        self.queue.push_back(ProcessedMessage::new(message, sender));
    }

    pub fn update_context(&mut self, head: Slot, finalized: Slot, justified: Slot) {
        self.context.head_slot = head;
        self.context.finalized_slot = finalized;
        self.context.justified_slot = justified;
        let _ = Epoch::ZERO;
    }
}
