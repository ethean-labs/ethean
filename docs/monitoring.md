# Monitoring and Observability

## Overview

Comprehensive monitoring and observability stack for Panro beacon chain implementation. This guide covers metrics collection, alerting, dashboards, logging, and performance monitoring for production deployments.

## Monitoring Architecture

### Stack Overview

```
┌─────────────────────────────────────────────────────────────┐
│                 Observability Stack                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ Metrics     │  │ Logs        │  │ Traces              │ │
│  │ Collection  │  │ Aggregation │  │ Analysis            │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ Prometheus  │  │ Grafana     │  │ Alertmanager        │ │
│  │ TSDB        │  │ Dashboard   │  │ Notifications       │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ ELK Stack   │  │ Jaeger      │  │ Custom              │ │
│  │ Logging     │  │ Tracing     │  │ Dashboards          │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Metrics Collection

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    monitor: 'panro-monitor'
    environment: 'production'

rule_files:
  - "panro_rules.yml"
  - "node_rules.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

scrape_configs:
  # Panro beacon node metrics
  - job_name: 'panro-beacon'
    static_configs:
      - targets: ['panro-beacon:8081']
    scrape_interval: 10s
    metrics_path: /metrics
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
      - source_labels: [__address__]
        regex: '([^:]+):\d+'
        target_label: node
        replacement: '${1}'

  # Panro validator metrics
  - job_name: 'panro-validator'
    static_configs:
      - targets: ['panro-validator:8082']
    scrape_interval: 10s
    metrics_path: /metrics

  # Node exporter for system metrics
  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']
    scrape_interval: 15s

  # Database metrics
  - job_name: 'postgres-exporter'
    static_configs:
      - targets: ['postgres-exporter:9187']
    scrape_interval: 30s

  # Network monitoring
  - job_name: 'blackbox'
    metrics_path: /probe
    params:
      module: [http_2xx]
    static_configs:
      - targets:
        - http://panro-beacon:8080/eth/v1/node/health
        - http://panro-beacon:8080/eth/v1/node/version
    relabel_configs:
      - source_labels: [__address__]
        target_label: __param_target
      - source_labels: [__param_target]
        target_label: instance
      - target_label: __address__
        replacement: blackbox-exporter:9115
```

### Custom Metrics Implementation

```rust
use prometheus::{
    Counter, Gauge, Histogram, IntCounter, IntGauge, 
    Registry, Opts, HistogramOpts
};
use std::sync::Arc;

pub struct PanroMetrics {
    registry: Registry,
    
    // Consensus metrics
    current_slot: IntGauge,
    current_epoch: IntGauge,
    finalized_epoch: IntGauge,
    justified_epoch: IntGauge,
    head_slot: IntGauge,
    
    // Block metrics
    blocks_processed: IntCounter,
    blocks_imported: IntCounter,
    blocks_rejected: IntCounter,
    block_processing_time: Histogram,
    orphaned_blocks: IntCounter,
    
    // Attestation metrics
    attestations_received: IntCounter,
    attestations_processed: IntCounter,
    attestations_aggregated: IntCounter,
    attestation_processing_time: Histogram,
    
    // Network metrics
    peers_connected: IntGauge,
    peer_connections_total: IntCounter,
    peer_disconnections_total: IntCounter,
    network_bytes_sent: IntCounter,
    network_bytes_received: IntCounter,
    gossip_messages_sent: IntCounter,
    gossip_messages_received: IntCounter,
    
    // Validator metrics
    validator_duties_scheduled: IntGauge,
    validator_duties_completed: IntCounter,
    validator_duties_missed: IntCounter,
    validator_balance: Gauge,
    validator_effectiveness: Gauge,
    
    // System metrics
    memory_usage: Gauge,
    cpu_usage: Gauge,
    disk_usage: Gauge,
    database_size: Gauge,
    
    // Performance metrics
    sync_distance: IntGauge,
    sync_speed: Gauge,
    fork_choice_time: Histogram,
    state_transition_time: Histogram,
}

impl PanroMetrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();
        
        // Consensus metrics
        let current_slot = IntGauge::with_opts(
            Opts::new("panro_beacon_current_slot", "Current slot number")
        )?;
        registry.register(Box::new(current_slot.clone()))?;
        
        let current_epoch = IntGauge::with_opts(
            Opts::new("panro_beacon_current_epoch", "Current epoch number")
        )?;
        registry.register(Box::new(current_epoch.clone()))?;
        
        let finalized_epoch = IntGauge::with_opts(
            Opts::new("panro_beacon_finalized_epoch", "Latest finalized epoch")
        )?;
        registry.register(Box::new(finalized_epoch.clone()))?;
        
        // Block metrics
        let blocks_processed = IntCounter::with_opts(
            Opts::new("panro_beacon_blocks_processed_total", 
                     "Total number of blocks processed")
        )?;
        registry.register(Box::new(blocks_processed.clone()))?;
        
        let block_processing_time = Histogram::with_opts(
            HistogramOpts::new("panro_beacon_block_processing_seconds",
                              "Time spent processing blocks")
                .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0])
        )?;
        registry.register(Box::new(block_processing_time.clone()))?;
        
        // Network metrics
        let peers_connected = IntGauge::with_opts(
            Opts::new("panro_beacon_peers_connected", "Number of connected peers")
        )?;
        registry.register(Box::new(peers_connected.clone()))?;
        
        let network_bytes_sent = IntCounter::with_opts(
            Opts::new("panro_beacon_network_bytes_sent_total", 
                     "Total bytes sent over network")
        )?;
        registry.register(Box::new(network_bytes_sent.clone()))?;
        
        // Continue with other metrics...
        
        Ok(Self {
            registry,
            current_slot,
            current_epoch,
            finalized_epoch,
            justified_epoch: IntGauge::with_opts(
                Opts::new("panro_beacon_justified_epoch", "Latest justified epoch")
            )?,
            head_slot: IntGauge::with_opts(
                Opts::new("panro_beacon_head_slot", "Head slot number")
            )?,
            blocks_processed,
            blocks_imported: IntCounter::with_opts(
                Opts::new("panro_beacon_blocks_imported_total", "Blocks imported")
            )?,
            blocks_rejected: IntCounter::with_opts(
                Opts::new("panro_beacon_blocks_rejected_total", "Blocks rejected")
            )?,
            block_processing_time,
            orphaned_blocks: IntCounter::with_opts(
                Opts::new("panro_beacon_orphaned_blocks_total", "Orphaned blocks")
            )?,
            attestations_received: IntCounter::with_opts(
                Opts::new("panro_beacon_attestations_received_total", "Attestations received")
            )?,
            attestations_processed: IntCounter::with_opts(
                Opts::new("panro_beacon_attestations_processed_total", "Attestations processed")
            )?,
            attestations_aggregated: IntCounter::with_opts(
                Opts::new("panro_beacon_attestations_aggregated_total", "Attestations aggregated")
            )?,
            attestation_processing_time: Histogram::with_opts(
                HistogramOpts::new("panro_beacon_attestation_processing_seconds",
                                  "Attestation processing time")
            )?,
            peers_connected,
            peer_connections_total: IntCounter::with_opts(
                Opts::new("panro_beacon_peer_connections_total", "Total peer connections")
            )?,
            peer_disconnections_total: IntCounter::with_opts(
                Opts::new("panro_beacon_peer_disconnections_total", "Total peer disconnections")
            )?,
            network_bytes_sent,
            network_bytes_received: IntCounter::with_opts(
                Opts::new("panro_beacon_network_bytes_received_total", "Total bytes received")
            )?,
            gossip_messages_sent: IntCounter::with_opts(
                Opts::new("panro_beacon_gossip_messages_sent_total", "Gossip messages sent")
            )?,
            gossip_messages_received: IntCounter::with_opts(
                Opts::new("panro_beacon_gossip_messages_received_total", "Gossip messages received")
            )?,
            validator_duties_scheduled: IntGauge::with_opts(
                Opts::new("panro_validator_duties_scheduled", "Scheduled validator duties")
            )?,
            validator_duties_completed: IntCounter::with_opts(
                Opts::new("panro_validator_duties_completed_total", "Completed validator duties")
            )?,
            validator_duties_missed: IntCounter::with_opts(
                Opts::new("panro_validator_duties_missed_total", "Missed validator duties")
            )?,
            validator_balance: Gauge::with_opts(
                Opts::new("panro_validator_balance_gwei", "Validator balance in Gwei")
            )?,
            validator_effectiveness: Gauge::with_opts(
                Opts::new("panro_validator_effectiveness", "Validator effectiveness percentage")
            )?,
            memory_usage: Gauge::with_opts(
                Opts::new("panro_system_memory_usage_bytes", "Memory usage in bytes")
            )?,
            cpu_usage: Gauge::with_opts(
                Opts::new("panro_system_cpu_usage_percent", "CPU usage percentage")
            )?,
            disk_usage: Gauge::with_opts(
                Opts::new("panro_system_disk_usage_bytes", "Disk usage in bytes")
            )?,
            database_size: Gauge::with_opts(
                Opts::new("panro_database_size_bytes", "Database size in bytes")
            )?,
            sync_distance: IntGauge::with_opts(
                Opts::new("panro_beacon_sync_distance", "Distance from head in slots")
            )?,
            sync_speed: Gauge::with_opts(
                Opts::new("panro_beacon_sync_speed", "Sync speed in slots per second")
            )?,
            fork_choice_time: Histogram::with_opts(
                HistogramOpts::new("panro_beacon_fork_choice_seconds", "Fork choice time")
            )?,
            state_transition_time: Histogram::with_opts(
                HistogramOpts::new("panro_beacon_state_transition_seconds", "State transition time")
            )?,
        })
    }
    
    pub fn update_consensus_metrics(&self, state: &BeaconState) {
        self.current_slot.set(state.slot().as_u64() as i64);
        self.current_epoch.set(state.current_epoch().as_u64() as i64);
        self.finalized_epoch.set(state.finalized_checkpoint().epoch.as_u64() as i64);
        self.justified_epoch.set(state.current_justified_checkpoint().epoch.as_u64() as i64);
    }
    
    pub fn record_block_processed(&self, processing_time: f64) {
        self.blocks_processed.inc();
        self.block_processing_time.observe(processing_time);
    }
    
    pub fn record_attestation_processed(&self, processing_time: f64) {
        self.attestations_processed.inc();
        self.attestation_processing_time.observe(processing_time);
    }
    
    pub fn update_network_metrics(&self, peer_count: usize, bytes_sent: u64, bytes_received: u64) {
        self.peers_connected.set(peer_count as i64);
        self.network_bytes_sent.inc_by(bytes_sent);
        self.network_bytes_received.inc_by(bytes_received);
    }
    
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

// Metrics server
pub struct MetricsServer {
    metrics: Arc<PanroMetrics>,
    server_handle: Option<tokio::task::JoinHandle<()>>,
}

impl MetricsServer {
    pub fn new(metrics: Arc<PanroMetrics>) -> Self {
        Self {
            metrics,
            server_handle: None,
        }
    }
    
    pub async fn start(&mut self, addr: SocketAddr) -> Result<(), std::io::Error> {
        let metrics = self.metrics.clone();
        
        let handle = tokio::spawn(async move {
            let make_svc = make_service_fn(move |_conn| {
                let metrics = metrics.clone();
                async move {
                    Ok::<_, Infallible>(service_fn(move |req| {
                        let metrics = metrics.clone();
                        async move {
                            match req.uri().path() {
                                "/metrics" => {
                                    let encoder = TextEncoder::new();
                                    let metric_families = metrics.registry().gather();
                                    let mut buffer = Vec::new();
                                    encoder.encode(&metric_families, &mut buffer).unwrap();
                                    
                                    Ok::<_, Infallible>(Response::builder()
                                        .status(200)
                                        .header("content-type", encoder.format_type())
                                        .body(Body::from(buffer))
                                        .unwrap())
                                }
                                "/health" => {
                                    Ok(Response::builder()
                                        .status(200)
                                        .body(Body::from("OK"))
                                        .unwrap())
                                }
                                _ => {
                                    Ok(Response::builder()
                                        .status(404)
                                        .body(Body::from("Not Found"))
                                        .unwrap())
                                }
                            }
                        }
                    }))
                }
            });
            
            let server = Server::bind(&addr).serve(make_svc);
            
            if let Err(e) = server.await {
                eprintln!("Metrics server error: {}", e);
            }
        });
        
        self.server_handle = Some(handle);
        
        Ok(())
    }
}
```

## Alerting Rules

### Prometheus Alerting Rules

```yaml
# panro_rules.yml
groups:
  - name: panro.rules
    rules:
      # Consensus alerts
      - alert: PanroNotSynced
        expr: panro_beacon_sync_distance > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Panro node is not synced"
          description: "Panro node {{ $labels.instance }} is {{ $value }} slots behind"
      
      - alert: PanroSyncStalled
        expr: rate(panro_beacon_current_slot[5m]) == 0
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Panro sync appears stalled"
          description: "Panro node {{ $labels.instance }} has not progressed in 2 minutes"
      
      - alert: PanroFinalizationDelay
        expr: panro_beacon_current_epoch - panro_beacon_finalized_epoch > 4
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Finalization delayed"
          description: "Finalization is {{ $value }} epochs behind current epoch"
      
      # Network alerts
      - alert: PanroLowPeerCount
        expr: panro_beacon_peers_connected < 10
        for: 3m
        labels:
          severity: warning
        annotations:
          summary: "Low peer count"
          description: "Panro node {{ $labels.instance }} has only {{ $value }} connected peers"
      
      - alert: PanroNoPeers
        expr: panro_beacon_peers_connected == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "No peers connected"
          description: "Panro node {{ $labels.instance }} has no connected peers"
      
      - alert: PanroHighNetworkLatency
        expr: panro_beacon_network_latency_seconds > 1.0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High network latency"
          description: "Network latency is {{ $value }}s for {{ $labels.instance }}"
      
      # Performance alerts
      - alert: PanroHighBlockProcessingTime
        expr: histogram_quantile(0.95, rate(panro_beacon_block_processing_seconds_bucket[5m])) > 2.0
        for: 3m
        labels:
          severity: warning
        annotations:
          summary: "High block processing time"
          description: "95th percentile block processing time is {{ $value }}s"
      
      - alert: PanroHighMemoryUsage
        expr: panro_system_memory_usage_bytes / (1024^3) > 16
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value | humanize }}B"
      
      - alert: PanroHighCPUUsage
        expr: panro_system_cpu_usage_percent > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage"
          description: "CPU usage is {{ $value }}%"
      
      - alert: PanroDiskSpaceLow
        expr: (panro_system_disk_usage_bytes / (1024^3)) / (node_filesystem_size_bytes{mountpoint="/"} / (1024^3)) > 0.85
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Low disk space"
          description: "Disk usage is {{ $value | humanizePercentage }}"
      
      # Validator alerts
      - alert: PanroValidatorDutiesMissed
        expr: increase(panro_validator_duties_missed_total[1h]) > 0
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "Validator duties missed"
          description: "{{ $value }} validator duties missed in the last hour"
      
      - alert: PanroValidatorEffectivenessLow
        expr: panro_validator_effectiveness < 90
        for: 30m
        labels:
          severity: warning
        annotations:
          summary: "Low validator effectiveness"
          description: "Validator effectiveness is {{ $value }}%"
      
      # System alerts
      - alert: PanroServiceDown
        expr: up{job="panro-beacon"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Panro service is down"
          description: "Panro beacon service is not responding"
      
      - alert: PanroAPIUnresponsive
        expr: probe_success{instance=~".*panro.*"} == 0
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Panro API unresponsive"
          description: "Panro API health check failed"
```

### Alertmanager Configuration

```yaml
# alertmanager.yml
global:
  smtp_smarthost: 'localhost:587'
  smtp_from: 'alerts@panro.example.com'
  smtp_auth_username: 'alerts@panro.example.com'
  smtp_auth_password: 'password'

route:
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'web.hook'
  routes:
    - match:
        severity: critical
      receiver: 'critical-alerts'
      group_wait: 10s
      repeat_interval: 5m
    - match:
        severity: warning
      receiver: 'warning-alerts'
      repeat_interval: 30m

receivers:
  - name: 'web.hook'
    webhook_configs:
      - url: 'http://127.0.0.1:5001/'

  - name: 'critical-alerts'
    email_configs:
      - to: 'oncall@panro.example.com'
        subject: '[CRITICAL] Panro Alert: {{ .GroupLabels.alertname }}'
        body: |
          {{ range .Alerts }}
          Alert: {{ .Annotations.summary }}
          Description: {{ .Annotations.description }}
          Labels: {{ range .Labels.SortedPairs }} {{ .Name }}={{ .Value }} {{ end }}
          {{ end }}
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/...'
        channel: '#panro-alerts'
        title: 'Critical Panro Alert'
        text: '{{ range .Alerts }}{{ .Annotations.summary }}{{ end }}'

  - name: 'warning-alerts'
    email_configs:
      - to: 'team@panro.example.com'
        subject: '[WARNING] Panro Alert: {{ .GroupLabels.alertname }}'
        body: |
          {{ range .Alerts }}
          Alert: {{ .Annotations.summary }}
          Description: {{ .Annotations.description }}
          {{ end }}

inhibit_rules:
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname', 'cluster', 'service']
```

## Grafana Dashboards

### Main Dashboard JSON

```json
{
  "dashboard": {
    "id": null,
    "title": "Panro Beacon Node Dashboard",
    "description": "Comprehensive monitoring dashboard for Panro beacon node",
    "tags": ["panro", "beacon", "ethereum"],
    "timezone": "browser",
    "panels": [
      {
        "id": 1,
        "title": "Consensus Status",
        "type": "stat",
        "targets": [
          {
            "expr": "panro_beacon_current_slot",
            "legendFormat": "Current Slot"
          },
          {
            "expr": "panro_beacon_current_epoch",
            "legendFormat": "Current Epoch"
          },
          {
            "expr": "panro_beacon_finalized_epoch",
            "legendFormat": "Finalized Epoch"
          },
          {
            "expr": "panro_beacon_justified_epoch",
            "legendFormat": "Justified Epoch"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0}
      },
      {
        "id": 2,
        "title": "Sync Distance",
        "type": "graph",
        "targets": [
          {
            "expr": "panro_beacon_sync_distance",
            "legendFormat": "Sync Distance (slots)"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0},
        "yAxes": [
          {
            "label": "Slots",
            "min": 0
          }
        ]
      },
      {
        "id": 3,
        "title": "Block Processing",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(panro_beacon_blocks_processed_total[5m])",
            "legendFormat": "Blocks Processed/sec"
          },
          {
            "expr": "rate(panro_beacon_blocks_imported_total[5m])",
            "legendFormat": "Blocks Imported/sec"
          },
          {
            "expr": "rate(panro_beacon_blocks_rejected_total[5m])",
            "legendFormat": "Blocks Rejected/sec"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 8}
      },
      {
        "id": 4,
        "title": "Network Status",
        "type": "graph",
        "targets": [
          {
            "expr": "panro_beacon_peers_connected",
            "legendFormat": "Connected Peers"
          },
          {
            "expr": "rate(panro_beacon_network_bytes_sent_total[5m])",
            "legendFormat": "Bytes Sent/sec"
          },
          {
            "expr": "rate(panro_beacon_network_bytes_received_total[5m])",
            "legendFormat": "Bytes Received/sec"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 8}
      },
      {
        "id": 5,
        "title": "Processing Times",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, rate(panro_beacon_block_processing_seconds_bucket[5m]))",
            "legendFormat": "Block Processing (p50)"
          },
          {
            "expr": "histogram_quantile(0.95, rate(panro_beacon_block_processing_seconds_bucket[5m]))",
            "legendFormat": "Block Processing (p95)"
          },
          {
            "expr": "histogram_quantile(0.99, rate(panro_beacon_block_processing_seconds_bucket[5m]))",
            "legendFormat": "Block Processing (p99)"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 16},
        "yAxes": [
          {
            "label": "Seconds",
            "min": 0
          }
        ]
      },
      {
        "id": 6,
        "title": "System Resources",
        "type": "graph",
        "targets": [
          {
            "expr": "panro_system_memory_usage_bytes / (1024^3)",
            "legendFormat": "Memory Usage (GB)"
          },
          {
            "expr": "panro_system_cpu_usage_percent",
            "legendFormat": "CPU Usage (%)"
          },
          {
            "expr": "panro_system_disk_usage_bytes / (1024^3)",
            "legendFormat": "Disk Usage (GB)"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 16}
      }
    ],
    "time": {
      "from": "now-1h",
      "to": "now"
    },
    "refresh": "10s"
  }
}
```

## Logging Configuration

### Structured Logging Implementation

```rust
use tracing::{info, warn, error, debug, trace};
use tracing_subscriber::{
    fmt, prelude::*, registry::Registry, filter::EnvFilter, Layer
};
use tracing_appender::{non_blocking, rolling};

pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub rotation: LogRotation,
    pub format: LogFormat,
    pub json_output: bool,
}

#[derive(Debug, Clone)]
pub enum LogRotation {
    Never,
    Hourly,
    Daily,
    Weekly,
}

#[derive(Debug, Clone)]
pub enum LogFormat {
    Pretty,
    Compact,
    Json,
}

pub fn init_logging(config: &LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));
    
    let registry = Registry::default().with(filter);
    
    // Console output
    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true);
    
    let registry = registry.with(console_layer);
    
    // File output if configured
    if let Some(file_path) = &config.file_path {
        let file_appender = match config.rotation {
            LogRotation::Never => rolling::never("logs", "panro.log"),
            LogRotation::Hourly => rolling::hourly("logs", "panro.log"),
            LogRotation::Daily => rolling::daily("logs", "panro.log"),
            LogRotation::Weekly => rolling::weekly("logs", "panro.log"),
        };
        
        let (file_writer, _guard) = non_blocking(file_appender);
        
        let file_layer = fmt::layer()
            .with_writer(file_writer)
            .with_ansi(false)
            .json();
        
        let registry = registry.with(file_layer);
        
        tracing::subscriber::set_global_default(registry)?;
    } else {
        tracing::subscriber::set_global_default(registry)?;
    }
    
    Ok(())
}

// Structured logging macros
#[macro_export]
macro_rules! log_block_processed {
    ($slot:expr, $root:expr, $duration:expr) => {
        tracing::info!(
            slot = %$slot,
            block_root = %$root,
            processing_time_ms = $duration.as_millis(),
            "Block processed successfully"
        );
    };
}

#[macro_export]
macro_rules! log_peer_event {
    ($event:expr, $peer_id:expr, $reason:expr) => {
        tracing::info!(
            event = %$event,
            peer_id = %$peer_id,
            reason = %$reason,
            "Peer event"
        );
    };
}

#[macro_export]
macro_rules! log_validator_duty {
    ($duty_type:expr, $slot:expr, $validator_index:expr, $status:expr) => {
        tracing::info!(
            duty_type = %$duty_type,
            slot = %$slot,
            validator_index = %$validator_index,
            status = %$status,
            "Validator duty"
        );
    };
}
```

### ELK Stack Configuration

```yaml
# docker-compose.elk.yml
version: '3.7'

services:
  elasticsearch:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.8.0
    container_name: elasticsearch
    environment:
      - node.name=elasticsearch
      - cluster.name=panro-logs
      - discovery.type=single-node
      - bootstrap.memory_lock=true
      - "ES_JAVA_OPTS=-Xms2g -Xmx2g"
      - xpack.security.enabled=false
    ulimits:
      memlock:
        soft: -1
        hard: -1
    volumes:
      - es-data:/usr/share/elasticsearch/data
    ports:
      - "9200:9200"
    networks:
      - elk

  logstash:
    image: docker.elastic.co/logstash/logstash:8.8.0
    container_name: logstash
    volumes:
      - ./logstash/config/logstash.yml:/usr/share/logstash/config/logstash.yml:ro
      - ./logstash/pipeline:/usr/share/logstash/pipeline:ro
    ports:
      - "5044:5044"
      - "5000:5000/tcp"
      - "5000:5000/udp"
      - "9600:9600"
    environment:
      LS_JAVA_OPTS: "-Xmx1g -Xms1g"
    networks:
      - elk
    depends_on:
      - elasticsearch

  kibana:
    image: docker.elastic.co/kibana/kibana:8.8.0
    container_name: kibana
    ports:
      - "5601:5601"
    environment:
      ELASTICSEARCH_URL: http://elasticsearch:9200
      ELASTICSEARCH_HOSTS: '["http://elasticsearch:9200"]'
    networks:
      - elk
    depends_on:
      - elasticsearch

  filebeat:
    image: docker.elastic.co/beats/filebeat:8.8.0
    container_name: filebeat
    user: root
    volumes:
      - ./filebeat/filebeat.yml:/usr/share/filebeat/filebeat.yml:ro
      - /var/lib/docker/containers:/var/lib/docker/containers:ro
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - panro-logs:/usr/share/filebeat/panro-logs:ro
    networks:
      - elk
    depends_on:
      - logstash

volumes:
  es-data:
  panro-logs:

networks:
  elk:
    driver: bridge
```

## Performance Monitoring

### APM Integration

```rust
use opentelemetry::{
    global, runtime::TokioCurrentThread, sdk::{
        trace::{self, Sampler}, Resource
    }, KeyValue
};
use opentelemetry_jaeger::new_agent_pipeline;
use tracing_opentelemetry::OpenTelemetryLayer;

pub fn init_tracing() -> Result<(), opentelemetry::trace::TraceError> {
    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    
    let tracer = new_agent_pipeline()
        .with_service_name("panro-beacon")
        .with_auto_split_batch(true)
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::TraceIdRatioBased(0.1))
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", "panro-beacon"),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ]))
        )
        .install_batch(TokioCurrentThread)?;
    
    let telemetry = OpenTelemetryLayer::new(tracer);
    
    let subscriber = Registry::default()
        .with(telemetry)
        .with(EnvFilter::from_default_env());
    
    tracing::subscriber::set_global_default(subscriber)?;
    
    Ok(())
}

// Performance instrumentation
#[tracing::instrument(skip(state, block))]
pub async fn process_block_with_tracing(
    state: &mut BeaconState,
    block: &SignedBeaconBlock,
) -> Result<(), BlockProcessingError> {
    let span = tracing::Span::current();
    span.record("block.slot", &block.message().slot().as_u64());
    span.record("block.root", &format!("{:?}", block.canonical_root()));
    
    let start = std::time::Instant::now();
    
    let result = process_block(state, block).await;
    
    let duration = start.elapsed();
    span.record("processing.duration_ms", &duration.as_millis());
    span.record("processing.success", &result.is_ok());
    
    result
}
```

## Health Checks

### Comprehensive Health Check System

```rust
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub overall: HealthState,
    pub checks: HashMap<String, HealthCheck>,
    pub timestamp: u64,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthState {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthState,
    pub message: String,
    pub last_updated: u64,
    pub details: Option<serde_json::Value>,
}

pub struct HealthChecker {
    checks: Vec<Box<dyn HealthCheckProvider + Send + Sync>>,
}

#[async_trait::async_trait]
pub trait HealthCheckProvider {
    async fn check(&self) -> HealthCheck;
    fn name(&self) -> &str;
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
        }
    }
    
    pub fn add_check<T>(&mut self, check: T)
    where
        T: HealthCheckProvider + Send + Sync + 'static,
    {
        self.checks.push(Box::new(check));
    }
    
    pub async fn get_health_status(&self) -> HealthStatus {
        let mut check_results = HashMap::new();
        let mut overall_state = HealthState::Healthy;
        
        for checker in &self.checks {
            let result = checker.check().await;
            
            // Determine overall status
            match result.status {
                HealthState::Critical => overall_state = HealthState::Critical,
                HealthState::Warning if matches!(overall_state, HealthState::Healthy) => {
                    overall_state = HealthState::Warning;
                }
                _ => {}
            }
            
            check_results.insert(result.name.clone(), result);
        }
        
        HealthStatus {
            overall: overall_state,
            checks: check_results,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

// Database health check
pub struct DatabaseHealthCheck {
    db: Arc<Database>,
}

#[async_trait::async_trait]
impl HealthCheckProvider for DatabaseHealthCheck {
    async fn check(&self) -> HealthCheck {
        match self.db.health_check().await {
            Ok(stats) => HealthCheck {
                name: "database".to_string(),
                status: HealthState::Healthy,
                message: "Database is responsive".to_string(),
                last_updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                details: Some(serde_json::to_value(stats).unwrap()),
            },
            Err(e) => HealthCheck {
                name: "database".to_string(),
                status: HealthState::Critical,
                message: format!("Database error: {}", e),
                last_updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                details: None,
            },
        }
    }
    
    fn name(&self) -> &str {
        "database"
    }
}

// Network health check
pub struct NetworkHealthCheck {
    network: Arc<NetworkService>,
    min_peers: usize,
}

#[async_trait::async_trait]
impl HealthCheckProvider for NetworkHealthCheck {
    async fn check(&self) -> HealthCheck {
        let peer_count = self.network.connected_peer_count().await;
        
        let (status, message) = if peer_count >= self.min_peers {
            (HealthState::Healthy, format!("Connected to {} peers", peer_count))
        } else if peer_count > 0 {
            (HealthState::Warning, format!("Only {} peers connected (minimum: {})", peer_count, self.min_peers))
        } else {
            (HealthState::Critical, "No peers connected".to_string())
        };
        
        HealthCheck {
            name: "network".to_string(),
            status,
            message,
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            details: Some(serde_json::json!({
                "peer_count": peer_count,
                "min_peers": self.min_peers
            })),
        }
    }
    
    fn name(&self) -> &str {
        "network"
    }
}
```
