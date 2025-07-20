//! Example of Network-Storage Integration
//!
//! Demonstrates how to set up and use the integrated network-storage system
//! for a complete Ethereum Beacon Chain client.

use panro::integration::{
    init_integration, health_check, IntegrationBuilder, IntegrationConfig,
    NetworkStorageBridge, HealthStatus,
};
use panro::network::orchestrator::{NetworkOrchestrator, OrchestratorBuilder};
use panro::storage::{StorageManager, StorageConfig};
use libp2p::identity::Keypair;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting Network-Storage Integration Example");

    // Create network orchestrator
    let keypair = Keypair::generate_ed25519();
    let bootstrap_nodes = vec![
        "/ip4/127.0.0.1/tcp/8000".parse()?,
    ];

    let network_orchestrator = OrchestratorBuilder::new()
        .with_keypair(keypair)
        .with_bootstrap_nodes(bootstrap_nodes)
        .with_max_connections(50)
        .with_security_enabled(true)
        .with_performance_optimization(true)
        .build()
        .await?;

    let network_orchestrator = Arc::new(RwLock::new(network_orchestrator));

    // Create storage manager
    let storage_config = StorageConfig {
        database: panro::storage::DatabaseConfig {
            path: "/tmp/panro_integration_example".to_string(),
            ..Default::default()
        },
        enable_pruning: false,
        archive_mode: true,
        ..Default::default()
    };

    let storage_manager = StorageManager::new(storage_config).await?;
    let storage_manager = Arc::new(RwLock::new(storage_manager));

    // Create integration configuration
    let integration_config = IntegrationConfig {
        sync_interval: Duration::from_secs(10),
        max_sync_batch_size: 500,
        enable_realtime_sync: true,
        enable_distributed_backup: true,
        consistency_check_interval: Duration::from_secs(60),
        max_concurrent_syncs: 5,
        ..Default::default()
    };

    // Initialize integration bridge
    let mut bridge = IntegrationBuilder::new()
        .with_config(integration_config)
        .with_network_orchestrator(network_orchestrator.clone())
        .with_storage_manager(storage_manager.clone())
        .build()
        .await?;

    info!("Integration bridge created successfully");

    // Start the integration bridge
    bridge.start().await?;
    info!("Integration bridge started");

    // Simulate some operations
    tokio::spawn(async move {
        // Wait a bit for initialization
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        // Simulate state sync
        match bridge.trigger_state_sync(None).await {
            Ok(sync_id) => {
                info!("Triggered state sync: {}", sync_id);
            }
            Err(e) => {
                error!("Failed to trigger state sync: {}", e);
            }
        }

        // Perform health check every 30 seconds
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            
            match health_check(&bridge).await {
                Ok(health_report) => {
                    info!("Health check completed");
                    info!("Overall health: {:?}", health_report.overall_health);
                    info!("Integration stats: {:?}", health_report.integration_stats);
                    info!("Sync stats: {:?}", health_report.sync_stats);
                    
                    if !health_report.recommendations.is_empty() {
                        warn!("Health recommendations:");
                        for recommendation in &health_report.recommendations {
                            warn!("  - {}", recommendation);
                        }
                    }

                    match health_report.overall_health {
                        HealthStatus::Critical | HealthStatus::Offline => {
                            error!("Critical health status detected!");
                        }
                        HealthStatus::Warning => {
                            warn!("Warning health status detected");
                        }
                        HealthStatus::Healthy => {
                            info!("System is healthy");
                        }
                    }
                }
                Err(e) => {
                    error!("Health check failed: {}", e);
                }
            }
        }
    });

    // Keep the main thread alive
    info!("Integration example running. Press Ctrl+C to exit.");
    tokio::signal::ctrl_c().await?;
    
    info!("Shutting down integration example...");
    bridge.stop().await?;
    
    info!("Integration example shutdown complete");
    Ok(())
}

/// Example function showing how to handle network events
async fn handle_network_events(bridge: &NetworkStorageBridge) -> Result<(), Box<dyn std::error::Error>> {
    use panro::integration::{IntegrationEvent, NetworkEventType};
    use std::time::{SystemTime, UNIX_EPOCH};

    // Simulate a peer connection event
    let peer_connected_event = IntegrationEvent::NetworkEvent {
        event_type: NetworkEventType::PeerConnected,
        peer_id: Some("12D3KooWExample".to_string()),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        data: vec![],
    };

    bridge.handle_network_event(peer_connected_event).await?;
    info!("Handled peer connection event");

    // Simulate a message received event
    let message_received_event = IntegrationEvent::NetworkEvent {
        event_type: NetworkEventType::MessageReceived,
        peer_id: Some("12D3KooWExample".to_string()),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        data: b"Hello from peer".to_vec(),
    };

    bridge.handle_network_event(message_received_event).await?;
    info!("Handled message received event");

    Ok(())
}

/// Example function showing how to handle storage events
async fn handle_storage_events(bridge: &NetworkStorageBridge) -> Result<(), Box<dyn std::error::Error>> {
    use panro::integration::{IntegrationEvent, StorageEventType};
    use std::time::{SystemTime, UNIX_EPOCH};

    // Simulate a data stored event
    let data_stored_event = IntegrationEvent::StorageEvent {
        event_type: StorageEventType::DataStored,
        key: "block_123".to_string(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        data: b"block_data".to_vec(),
    };

    bridge.handle_storage_event(data_stored_event).await?;
    info!("Handled data stored event");

    // Simulate an index updated event
    let index_updated_event = IntegrationEvent::StorageEvent {
        event_type: StorageEventType::IndexUpdated,
        key: "slot_index".to_string(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        data: vec![],
    };

    bridge.handle_storage_event(index_updated_event).await?;
    info!("Handled index updated event");

    Ok(())
}

/// Example function showing consistency check
async fn demonstrate_consistency_check(bridge: &NetworkStorageBridge) -> Result<(), Box<dyn std::error::Error>> {
    info!("Performing consistency check demonstration");

    let consistency_report = bridge.perform_consistency_check().await?;
    
    info!("Consistency Check Results:");
    info!("  Network consistent: {}", consistency_report.network_consistent);
    info!("  Storage consistent: {}", consistency_report.storage_consistent);
    info!("  Overall consistent: {}", consistency_report.overall_consistent);
    info!("  Check duration: {:?}", consistency_report.check_duration);
    info!("  Issues found: {}", consistency_report.issues_found.len());

    if !consistency_report.issues_found.is_empty() {
        warn!("Consistency issues detected:");
        for issue in &consistency_report.issues_found {
            warn!("  - {}: {}", issue.issue_type, issue.description);
        }
    }

    if !consistency_report.recommendations.is_empty() {
        info!("Recommendations:");
        for recommendation in &consistency_report.recommendations {
            info!("  - {}", recommendation);
        }
    }

    Ok(())
}

/// Example function showing integration statistics
async fn show_integration_statistics(bridge: &NetworkStorageBridge) -> Result<(), Box<dyn std::error::Error>> {
    let integration_stats = bridge.get_stats().await;
    let sync_stats = bridge.get_sync_stats().await;

    info!("Integration Statistics:");
    info!("  Total events processed: {}", integration_stats.total_events_processed);
    info!("  Network events: {}", integration_stats.network_events_processed);
    info!("  Storage events: {}", integration_stats.storage_events_processed);
    info!("  Sync operations completed: {}", integration_stats.sync_operations_completed);
    info!("  Sync operations failed: {}", integration_stats.sync_operations_failed);
    info!("  Conflicts detected: {}", integration_stats.conflicts_detected);
    info!("  Conflicts resolved: {}", integration_stats.conflicts_resolved);
    info!("  Data consistency rate: {:.2}%", integration_stats.data_consistency_rate * 100.0);

    info!("Sync Statistics:");
    info!("  Total sync requests: {}", sync_stats.total_sync_requests);
    info!("  Completed syncs: {}", sync_stats.completed_syncs);
    info!("  Failed syncs: {}", sync_stats.failed_syncs);
    info!("  Active sync count: {}", sync_stats.active_sync_count);
    info!("  Sync queue size: {}", sync_stats.sync_queue_size);
    info!("  Data transferred: {} bytes", sync_stats.data_transferred);

    Ok(())
}
