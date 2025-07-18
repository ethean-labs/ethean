//! WebSocket API implementation for real-time streaming
//!
//! Provides WebSocket endpoints for real-time block streaming, attestation events,
//! and validator duty updates. Implements Server-Sent Events for event subscriptions.

use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        State, Path, Query,
    },
    response::Response,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn, error};

use super::{ApiState, error::Result};
use crate::types::{BeaconBlock, Attestation, Slot, Epoch};

/// WebSocket subscription types
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionType {
    Block,
    Attestation,
    ValidatorDuty,
    ChainReorg,
    FinalizedCheckpoint,
    Head,
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    Subscribe { topics: Vec<SubscriptionType> },
    Unsubscribe { topics: Vec<SubscriptionType> },
    Block(BeaconBlock),
    Attestation(Attestation),
    ValidatorDuty(ValidatorDutyUpdate),
    ChainReorg(ChainReorgEvent),
    FinalizedCheckpoint(FinalizedCheckpointEvent),
    Head(HeadEvent),
    Error(String),
    Ping,
    Pong,
}

/// Validator duty update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorDutyUpdate {
    pub epoch: Epoch,
    pub validator_index: u64,
    pub duty_type: DutyType,
    pub slot: Slot,
    pub committee_index: Option<u64>,
}

/// Duty type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DutyType {
    Attester,
    Proposer,
    SyncCommittee,
}

/// Chain reorganization event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainReorgEvent {
    pub slot: Slot,
    pub depth: u64,
    pub old_head_block: String,
    pub new_head_block: String,
    pub old_head_state: String,
    pub new_head_state: String,
    pub epoch: Epoch,
}

/// Finalized checkpoint event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizedCheckpointEvent {
    pub block: String,
    pub state: String,
    pub epoch: Epoch,
    pub execution_optimistic: bool,
}

/// Head event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadEvent {
    pub slot: Slot,
    pub block: String,
    pub state: String,
    pub epoch_transition: bool,
    pub execution_optimistic: bool,
    pub previous_duty_dependent_root: String,
    pub current_duty_dependent_root: String,
}

/// WebSocket event broadcaster
#[derive(Debug, Clone)]
pub struct EventBroadcaster {
    block_tx: broadcast::Sender<BeaconBlock>,
    attestation_tx: broadcast::Sender<Attestation>,
    validator_duty_tx: broadcast::Sender<ValidatorDutyUpdate>,
    chain_reorg_tx: broadcast::Sender<ChainReorgEvent>,
    finalized_checkpoint_tx: broadcast::Sender<FinalizedCheckpointEvent>,
    head_tx: broadcast::Sender<HeadEvent>,
}

impl EventBroadcaster {
    pub fn new() -> Self {
        let (block_tx, _) = broadcast::channel(1000);
        let (attestation_tx, _) = broadcast::channel(1000);
        let (validator_duty_tx, _) = broadcast::channel(1000);
        let (chain_reorg_tx, _) = broadcast::channel(100);
        let (finalized_checkpoint_tx, _) = broadcast::channel(100);
        let (head_tx, _) = broadcast::channel(100);

        Self {
            block_tx,
            attestation_tx,
            validator_duty_tx,
            chain_reorg_tx,
            finalized_checkpoint_tx,
            head_tx,
        }
    }

    pub fn broadcast_block(&self, block: BeaconBlock) {
        if let Err(e) = self.block_tx.send(block) {
            warn!("Failed to broadcast block: {}", e);
        }
    }

    pub fn broadcast_attestation(&self, attestation: Attestation) {
        if let Err(e) = self.attestation_tx.send(attestation) {
            warn!("Failed to broadcast attestation: {}", e);
        }
    }

    pub fn broadcast_validator_duty(&self, duty: ValidatorDutyUpdate) {
        if let Err(e) = self.validator_duty_tx.send(duty) {
            warn!("Failed to broadcast validator duty: {}", e);
        }
    }

    pub fn broadcast_chain_reorg(&self, reorg: ChainReorgEvent) {
        if let Err(e) = self.chain_reorg_tx.send(reorg) {
            warn!("Failed to broadcast chain reorg: {}", e);
        }
    }

    pub fn broadcast_finalized_checkpoint(&self, checkpoint: FinalizedCheckpointEvent) {
        if let Err(e) = self.finalized_checkpoint_tx.send(checkpoint) {
            warn!("Failed to broadcast finalized checkpoint: {}", e);
        }
    }

    pub fn broadcast_head(&self, head: HeadEvent) {
        if let Err(e) = self.head_tx.send(head) {
            warn!("Failed to broadcast head: {}", e);
        }
    }
}

/// Create WebSocket routes
pub fn create_routes() -> Router<ApiState> {
    Router::new()
        .route("/ws", get(websocket_handler))
        .route("/events", get(events_handler))
        .route("/stream/blocks", get(block_stream_handler))
        .route("/stream/attestations", get(attestation_stream_handler))
}

/// WebSocket upgrade handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<ApiState>,
) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

/// Handle WebSocket connection
async fn handle_websocket(socket: WebSocket, state: ApiState) {
    let (mut sender, mut receiver) = socket.split();
    let broadcaster = Arc::new(EventBroadcaster::new());
    
    // Create subscription tracking
    let mut subscriptions = std::collections::HashSet::new();
    let mut receivers = Vec::new();

    info!("WebSocket connection established");

    // Handle incoming messages
    let receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(msg) => {
                    if let Ok(text) = msg.to_text() {
                        if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(text) {
                            match ws_msg {
                                WsMessage::Subscribe { topics } => {
                                    for topic in topics {
                                        subscriptions.insert(topic.clone());
                                        match topic {
                                            SubscriptionType::Block => {
                                                let rx = broadcaster.block_tx.subscribe();
                                                receivers.push(rx);
                                            }
                                            SubscriptionType::Attestation => {
                                                let rx = broadcaster.attestation_tx.subscribe();
                                                receivers.push(rx);
                                            }
                                            SubscriptionType::ValidatorDuty => {
                                                let rx = broadcaster.validator_duty_tx.subscribe();
                                                receivers.push(rx);
                                            }
                                            SubscriptionType::ChainReorg => {
                                                let rx = broadcaster.chain_reorg_tx.subscribe();
                                                receivers.push(rx);
                                            }
                                            SubscriptionType::FinalizedCheckpoint => {
                                                let rx = broadcaster.finalized_checkpoint_tx.subscribe();
                                                receivers.push(rx);
                                            }
                                            SubscriptionType::Head => {
                                                let rx = broadcaster.head_tx.subscribe();
                                                receivers.push(rx);
                                            }
                                        }
                                    }
                                }
                                WsMessage::Unsubscribe { topics } => {
                                    for topic in topics {
                                        subscriptions.remove(&topic);
                                    }
                                }
                                WsMessage::Ping => {
                                    // Send pong response
                                    let pong_msg = WsMessage::Pong;
                                    if let Ok(json) = serde_json::to_string(&pong_msg) {
                                        if let Err(e) = sender.send(axum::extract::ws::Message::Text(json)).await {
                                            error!("Failed to send pong: {}", e);
                                            break;
                                        }
                                    }
                                }
                                _ => {
                                    warn!("Unexpected message type received");
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    });

    // Wait for the receiving task to complete
    let _ = receive_task.await;
    info!("WebSocket connection closed");
}

/// Server-Sent Events handler
pub async fn events_handler(
    State(_state): State<ApiState>,
) -> Response {
    use axum::response::sse::{Event, Sse};
    use futures::stream::{self, Stream};
    use std::convert::Infallible;
    use std::time::Duration;

    let stream = stream::unfold(0, |state| async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let event = Event::default()
            .event("heartbeat")
            .data(format!("heartbeat-{}", state));
        Some((Ok::<_, Infallible>(event), state + 1))
    });

    Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(30))
                .text("keep-alive"),
        )
        .into_response()
}

/// Block stream handler
pub async fn block_stream_handler(
    State(_state): State<ApiState>,
) -> Response {
    use axum::response::sse::{Event, Sse};
    use futures::stream::{self, Stream};
    use std::convert::Infallible;
    use std::time::Duration;

    let stream = stream::unfold(0, |state| async move {
        tokio::time::sleep(Duration::from_secs(12)).await; // Simulate block time
        let block_event = Event::default()
            .event("block")
            .data(format!("{{\"slot\": {}, \"block_root\": \"0x{:064x}\"}}", state, state));
        Some((Ok::<_, Infallible>(block_event), state + 1))
    });

    Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(30))
                .text("keep-alive"),
        )
        .into_response()
}

/// Attestation stream handler
pub async fn attestation_stream_handler(
    State(_state): State<ApiState>,
) -> Response {
    use axum::response::sse::{Event, Sse};
    use futures::stream::{self, Stream};
    use std::convert::Infallible;
    use std::time::Duration;

    let stream = stream::unfold(0, |state| async move {
        tokio::time::sleep(Duration::from_secs(4)).await; // Simulate attestation frequency
        let attestation_event = Event::default()
            .event("attestation")
            .data(format!("{{\"slot\": {}, \"committee_index\": {}, \"validator_count\": {}}}", 
                state, state % 64, (state % 10) + 1));
        Some((Ok::<_, Infallible>(attestation_event), state + 1))
    });

    Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(30))
                .text("keep-alive"),
        )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_event_broadcaster() {
        let broadcaster = EventBroadcaster::new();
        let mut block_rx = broadcaster.block_tx.subscribe();
        
        // Create a test block
        let test_block = BeaconBlock::default();
        
        // Broadcast the block
        broadcaster.broadcast_block(test_block.clone());
        
        // Verify we received the block
        let received = tokio::time::timeout(Duration::from_millis(100), block_rx.recv()).await;
        assert!(received.is_ok());
        assert!(received.unwrap().is_ok());
    }

    #[tokio::test]
    async fn test_websocket_message_serialization() {
        let msg = WsMessage::Subscribe {
            topics: vec![SubscriptionType::Block, SubscriptionType::Attestation],
        };
        
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: WsMessage = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            WsMessage::Subscribe { topics } => {
                assert_eq!(topics.len(), 2);
                assert!(topics.contains(&SubscriptionType::Block));
                assert!(topics.contains(&SubscriptionType::Attestation));
            }
            _ => panic!("Unexpected message type"),
        }
    }

    #[tokio::test]
    async fn test_validator_duty_update_serialization() {
        let duty = ValidatorDutyUpdate {
            epoch: 12345,
            validator_index: 67890,
            duty_type: DutyType::Attester,
            slot: 123456,
            committee_index: Some(7),
        };
        
        let json = serde_json::to_string(&duty).unwrap();
        let deserialized: ValidatorDutyUpdate = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.epoch, 12345);
        assert_eq!(deserialized.validator_index, 67890);
        assert_eq!(deserialized.slot, 123456);
        assert_eq!(deserialized.committee_index, Some(7));
    }
}
