# Deployment and Operations Guide

## Overview

This guide covers deployment strategies, operational procedures, monitoring, and maintenance for Panro beacon chain implementation. It includes production deployment patterns, infrastructure requirements, and operational best practices.

## Deployment Architecture

### Production Infrastructure

```
┌─────────────────────────────────────────────────────────────┐
│                    Production Setup                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ Load        │  │ Beacon      │  │ Validator           │ │
│  │ Balancer    │  │ Node        │  │ Client              │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ Monitoring  │  │ Database    │  │ Backup              │ │
│  │ Stack       │  │ Cluster     │  │ Storage             │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    Network Layer                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ Firewall    │  │ VPN         │  │ DDoS                │ │
│  │ Rules       │  │ Gateway     │  │ Protection          │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## System Requirements

### Minimum Requirements

```yaml
# Development/Testing Environment
hardware:
  cpu: 2 cores (x86_64)
  memory: 4 GB RAM
  storage: 100 GB SSD
  network: 100 Mbps

software:
  os: Ubuntu 20.04+ / CentOS 8+ / macOS 11+
  rust: 1.70+
  dependencies:
    - openssl-dev
    - pkg-config
    - build-essential
```

### Production Requirements

```yaml
# Production Environment
hardware:
  cpu: 8+ cores (x86_64)
  memory: 32+ GB RAM
  storage: 2+ TB NVMe SSD
  network: 1+ Gbps
  redundancy: RAID 1/10 for storage

software:
  os: Ubuntu 22.04 LTS / RHEL 9
  rust: Latest stable
  monitoring:
    - Prometheus
    - Grafana
    - Alertmanager
  backup:
    - Automated backup solution
    - Off-site storage
```

## Docker Deployment

### Dockerfile

```dockerfile
# Multi-stage build for Panro beacon node
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# Build optimized release binary
RUN cargo build --release --bin panro

# Runtime image
FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create panro user
RUN useradd -m -u 1001 panro

# Copy binary
COPY --from=builder /app/target/release/panro /usr/local/bin/panro

# Set permissions
RUN chmod +x /usr/local/bin/panro

# Create data directory
RUN mkdir -p /data && chown panro:panro /data

# Switch to panro user
USER panro

# Expose ports
EXPOSE 9000 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD panro health-check || exit 1

# Default command
CMD ["panro", "--config", "/config/panro.toml"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  panro-beacon:
    image: panro/beacon-node:latest
    container_name: panro-beacon
    restart: unless-stopped
    ports:
      - "9000:9000"  # P2P
      - "8080:8080"  # HTTP API
    volumes:
      - ./config:/config:ro
      - panro-data:/data
      - ./logs:/logs
    environment:
      - RUST_LOG=info
      - PANRO_CONFIG_PATH=/config/panro.toml
    networks:
      - panro-network
    depends_on:
      - prometheus
      - grafana
    deploy:
      resources:
        limits:
          memory: 16G
        reservations:
          memory: 8G
    healthcheck:
      test: ["CMD", "panro", "health-check"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s

  panro-validator:
    image: panro/validator:latest
    container_name: panro-validator
    restart: unless-stopped
    volumes:
      - ./config:/config:ro
      - ./keys:/keys:ro
      - validator-data:/data
    environment:
      - RUST_LOG=info
      - PANRO_BEACON_NODE_URL=http://panro-beacon:8080
    networks:
      - panro-network
    depends_on:
      - panro-beacon
    deploy:
      resources:
        limits:
          memory: 4G
        reservations:
          memory: 2G

  prometheus:
    image: prom/prometheus:latest
    container_name: panro-prometheus
    restart: unless-stopped
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/usr/share/prometheus/console_libraries'
      - '--web.console.templates=/usr/share/prometheus/consoles'
      - '--storage.tsdb.retention.time=30d'
      - '--web.enable-lifecycle'
    networks:
      - panro-network

  grafana:
    image: grafana/grafana:latest
    container_name: panro-grafana
    restart: unless-stopped
    ports:
      - "3000:3000"
    volumes:
      - grafana-data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources:ro
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin123
      - GF_USERS_ALLOW_SIGN_UP=false
    networks:
      - panro-network
    depends_on:
      - prometheus

  alertmanager:
    image: prom/alertmanager:latest
    container_name: panro-alertmanager
    restart: unless-stopped
    ports:
      - "9093:9093"
    volumes:
      - ./monitoring/alertmanager.yml:/etc/alertmanager/alertmanager.yml:ro
      - alertmanager-data:/alertmanager
    networks:
      - panro-network

volumes:
  panro-data:
  validator-data:
  prometheus-data:
  grafana-data:
  alertmanager-data:

networks:
  panro-network:
    driver: bridge
```

## Kubernetes Deployment

### Deployment Manifests

```yaml
# beacon-node-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: panro-beacon-node
  namespace: panro
  labels:
    app: panro-beacon-node
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
  selector:
    matchLabels:
      app: panro-beacon-node
  template:
    metadata:
      labels:
        app: panro-beacon-node
    spec:
      containers:
      - name: panro-beacon
        image: panro/beacon-node:v0.1.0
        ports:
        - containerPort: 9000
          name: p2p
        - containerPort: 8080
          name: http-api
        resources:
          requests:
            memory: "8Gi"
            cpu: "2"
          limits:
            memory: "16Gi"
            cpu: "4"
        env:
        - name: RUST_LOG
          value: "info"
        - name: PANRO_CONFIG_PATH
          value: "/config/panro.toml"
        volumeMounts:
        - name: config
          mountPath: /config
          readOnly: true
        - name: data
          mountPath: /data
        - name: keys
          mountPath: /keys
          readOnly: true
        livenessProbe:
          exec:
            command:
            - panro
            - health-check
          initialDelaySeconds: 60
          periodSeconds: 30
          timeoutSeconds: 10
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /eth/v1/node/health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
      volumes:
      - name: config
        configMap:
          name: panro-config
      - name: data
        persistentVolumeClaim:
          claimName: panro-data-pvc
      - name: keys
        secret:
          secretName: panro-keys
          defaultMode: 0400
      nodeSelector:
        panro.io/node-type: beacon
      tolerations:
      - key: "panro.io/dedicated"
        operator: "Equal"
        value: "beacon"
        effect: "NoSchedule"

---
# Service for beacon node
apiVersion: v1
kind: Service
metadata:
  name: panro-beacon-service
  namespace: panro
spec:
  selector:
    app: panro-beacon-node
  ports:
  - name: p2p
    port: 9000
    targetPort: 9000
    protocol: TCP
  - name: http-api
    port: 8080
    targetPort: 8080
    protocol: TCP
  type: LoadBalancer

---
# Persistent Volume Claim
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: panro-data-pvc
  namespace: panro
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 2Ti
  storageClassName: fast-ssd
```

### Helm Chart Structure

```
panro-helm/
├── Chart.yaml
├── values.yaml
├── templates/
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── configmap.yaml
│   ├── secret.yaml
│   ├── ingress.yaml
│   ├── hpa.yaml
│   └── servicemonitor.yaml
└── charts/
```

## Configuration Management

### Production Configuration

```toml
# production.toml
[node]
name = "panro-prod-node-01"
data_dir = "/data"
log_level = "info"
metrics_enabled = true
metrics_address = "0.0.0.0:8081"

[network]
listen_address = "0.0.0.0:9000"
discovery_port = 9001
max_peers = 200
target_peers = 150

# Production-optimized network settings
connection_timeout = "30s"
handshake_timeout = "15s"
max_concurrent_dials = 20

[network.gossip]
mesh_n = 8
mesh_n_low = 6
mesh_n_high = 16
heartbeat_interval = "700ms"

[consensus]
checkpoint_sync_url = "https://checkpoint-sync.example.com"
weak_subjectivity_checkpoint = "0x..."
suggested_fee_recipient = "0x..."

[database]
engine = "rocksdb"
cache_size = "8GB"
max_open_files = 2000
write_buffer_size = "256MB"

[database.compaction]
level0_file_num_compaction_trigger = 4
max_background_compactions = 8
max_background_flushes = 4

[monitoring]
enabled = true
prometheus_address = "0.0.0.0:8081"
log_file = "/logs/panro.log"
log_rotation = "daily"
log_retention_days = 30

[security]
firewall_enabled = true
rate_limiting_enabled = true
ddos_protection_enabled = true
tls_enabled = true
tls_cert_path = "/certs/server.crt"
tls_key_path = "/certs/server.key"
```

## Process Management

### Systemd Service

```ini
# /etc/systemd/system/panro.service
[Unit]
Description=Panro Beacon Node
After=network.target
Wants=network.target

[Service]
Type=exec
User=panro
Group=panro
ExecStart=/usr/local/bin/panro --config /etc/panro/panro.toml
ExecReload=/bin/kill -HUP $MAINPID
KillMode=mixed
KillSignal=SIGINT
TimeoutStopSec=90
Restart=always
RestartSec=10

# Security settings
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/var/lib/panro /var/log/panro

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

# Environment
Environment=RUST_LOG=info
Environment=RUST_BACKTRACE=1

[Install]
WantedBy=multi-user.target
```

### Process Monitoring Script

```bash
#!/bin/bash
# panro-monitor.sh

PANRO_PID_FILE="/var/run/panro/panro.pid"
PANRO_CONFIG="/etc/panro/panro.toml"
PANRO_BINARY="/usr/local/bin/panro"
PANRO_USER="panro"
PANRO_LOG="/var/log/panro/panro.log"

check_process() {
    if [ -f "$PANRO_PID_FILE" ]; then
        PID=$(cat "$PANRO_PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            return 0
        else
            rm -f "$PANRO_PID_FILE"
            return 1
        fi
    else
        return 1
    fi
}

start_panro() {
    if check_process; then
        echo "Panro is already running (PID: $(cat $PANRO_PID_FILE))"
        return 0
    fi
    
    echo "Starting Panro beacon node..."
    sudo -u "$PANRO_USER" "$PANRO_BINARY" \
        --config "$PANRO_CONFIG" \
        --daemon \
        --pid-file "$PANRO_PID_FILE" \
        >> "$PANRO_LOG" 2>&1
    
    if check_process; then
        echo "Panro started successfully (PID: $(cat $PANRO_PID_FILE))"
        return 0
    else
        echo "Failed to start Panro"
        return 1
    fi
}

stop_panro() {
    if ! check_process; then
        echo "Panro is not running"
        return 0
    fi
    
    PID=$(cat "$PANRO_PID_FILE")
    echo "Stopping Panro (PID: $PID)..."
    
    kill -TERM "$PID"
    
    # Wait for graceful shutdown
    for i in {1..30}; do
        if ! ps -p "$PID" > /dev/null 2>&1; then
            rm -f "$PANRO_PID_FILE"
            echo "Panro stopped successfully"
            return 0
        fi
        sleep 1
    done
    
    # Force kill if still running
    echo "Force killing Panro..."
    kill -KILL "$PID"
    rm -f "$PANRO_PID_FILE"
    
    return 0
}

status_panro() {
    if check_process; then
        PID=$(cat "$PANRO_PID_FILE")
        echo "Panro is running (PID: $PID)"
        
        # Check if process is responsive
        if curl -s http://localhost:8080/eth/v1/node/health > /dev/null; then
            echo "Panro is responding to API requests"
        else
            echo "WARNING: Panro is not responding to API requests"
        fi
        
        return 0
    else
        echo "Panro is not running"
        return 1
    fi
}

case "$1" in
    start)
        start_panro
        ;;
    stop)
        stop_panro
        ;;
    restart)
        stop_panro
        sleep 2
        start_panro
        ;;
    status)
        status_panro
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status}"
        exit 1
        ;;
esac

exit $?
```

## Backup and Recovery

### Automated Backup Script

```bash
#!/bin/bash
# backup-panro.sh

BACKUP_DIR="/backup/panro"
DATA_DIR="/var/lib/panro"
CONFIG_DIR="/etc/panro"
RETENTION_DAYS=30
BACKUP_PREFIX="panro-backup"
COMPRESSION="gzip"
ENCRYPTION_KEY="/etc/panro/backup-encryption.key"

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Generate timestamp
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_NAME="${BACKUP_PREFIX}_${TIMESTAMP}"
BACKUP_PATH="${BACKUP_DIR}/${BACKUP_NAME}"

echo "Starting backup: $BACKUP_NAME"

# Stop Panro for consistent backup
echo "Stopping Panro for backup..."
systemctl stop panro

# Create backup
echo "Creating backup archive..."
tar -czf "${BACKUP_PATH}.tar.gz" \
    -C "$(dirname "$DATA_DIR")" "$(basename "$DATA_DIR")" \
    -C "$(dirname "$CONFIG_DIR")" "$(basename "$CONFIG_DIR")"

# Encrypt backup if key exists
if [ -f "$ENCRYPTION_KEY" ]; then
    echo "Encrypting backup..."
    gpg --symmetric --cipher-algo AES256 \
        --passphrase-file "$ENCRYPTION_KEY" \
        --batch --quiet \
        "${BACKUP_PATH}.tar.gz"
    rm "${BACKUP_PATH}.tar.gz"
    FINAL_BACKUP="${BACKUP_PATH}.tar.gz.gpg"
else
    FINAL_BACKUP="${BACKUP_PATH}.tar.gz"
fi

# Restart Panro
echo "Restarting Panro..."
systemctl start panro

# Verify backup
if [ -f "$FINAL_BACKUP" ]; then
    BACKUP_SIZE=$(du -h "$FINAL_BACKUP" | cut -f1)
    echo "Backup completed successfully: $FINAL_BACKUP ($BACKUP_SIZE)"
else
    echo "ERROR: Backup failed!"
    exit 1
fi

# Clean up old backups
echo "Cleaning up old backups (retention: $RETENTION_DAYS days)..."
find "$BACKUP_DIR" -name "${BACKUP_PREFIX}_*" -mtime +$RETENTION_DAYS -delete

# Upload to remote storage (optional)
if [ -n "$REMOTE_BACKUP_URL" ]; then
    echo "Uploading backup to remote storage..."
    rsync -avz "$FINAL_BACKUP" "$REMOTE_BACKUP_URL/"
fi

echo "Backup process completed successfully"
```

### Recovery Procedures

```bash
#!/bin/bash
# restore-panro.sh

BACKUP_FILE="$1"
RESTORE_DIR="/var/lib/panro-restore"
DATA_DIR="/var/lib/panro"
CONFIG_DIR="/etc/panro"
ENCRYPTION_KEY="/etc/panro/backup-encryption.key"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup-file>"
    exit 1
fi

if [ ! -f "$BACKUP_FILE" ]; then
    echo "ERROR: Backup file not found: $BACKUP_FILE"
    exit 1
fi

echo "Starting restore from: $BACKUP_FILE"

# Stop Panro
echo "Stopping Panro service..."
systemctl stop panro

# Backup current data
echo "Backing up current data..."
mv "$DATA_DIR" "${DATA_DIR}.bak.$(date +%Y%m%d_%H%M%S)"

# Create restore directory
mkdir -p "$RESTORE_DIR"

# Decrypt if needed
if [[ "$BACKUP_FILE" == *.gpg ]]; then
    if [ ! -f "$ENCRYPTION_KEY" ]; then
        echo "ERROR: Encryption key not found: $ENCRYPTION_KEY"
        exit 1
    fi
    
    echo "Decrypting backup..."
    DECRYPTED_FILE="${BACKUP_FILE%.gpg}"
    gpg --decrypt --passphrase-file "$ENCRYPTION_KEY" \
        --batch --quiet \
        --output "$DECRYPTED_FILE" \
        "$BACKUP_FILE"
    EXTRACT_FILE="$DECRYPTED_FILE"
else
    EXTRACT_FILE="$BACKUP_FILE"
fi

# Extract backup
echo "Extracting backup..."
tar -xzf "$EXTRACT_FILE" -C "$RESTORE_DIR"

# Restore data
echo "Restoring data..."
mv "${RESTORE_DIR}/$(basename "$DATA_DIR")" "$DATA_DIR"
chown -R panro:panro "$DATA_DIR"

# Verify restoration
if [ -d "$DATA_DIR" ]; then
    echo "Data restoration completed successfully"
else
    echo "ERROR: Data restoration failed!"
    exit 1
fi

# Clean up
rm -rf "$RESTORE_DIR"
if [ -f "$DECRYPTED_FILE" ]; then
    rm "$DECRYPTED_FILE"
fi

# Start Panro
echo "Starting Panro service..."
systemctl start panro

# Wait for startup
echo "Waiting for Panro to start..."
sleep 10

# Verify service is running
if systemctl is-active --quiet panro; then
    echo "Panro service is running"
    
    # Check API responsiveness
    if curl -s http://localhost:8080/eth/v1/node/health > /dev/null; then
        echo "Panro is responding to API requests"
        echo "Restore completed successfully!"
    else
        echo "WARNING: Panro is not responding to API requests"
        echo "Check logs: journalctl -u panro -f"
    fi
else
    echo "ERROR: Panro service failed to start"
    echo "Check logs: journalctl -u panro -f"
    exit 1
fi
```

## Security Hardening

### Firewall Configuration

```bash
#!/bin/bash
# setup-firewall.sh

# Basic firewall setup for Panro beacon node

# Reset iptables
iptables -F
iptables -X
iptables -t nat -F
iptables -t nat -X
iptables -t mangle -F
iptables -t mangle -X

# Default policies
iptables -P INPUT DROP
iptables -P FORWARD DROP
iptables -P OUTPUT ACCEPT

# Allow loopback
iptables -A INPUT -i lo -j ACCEPT
iptables -A OUTPUT -o lo -j ACCEPT

# Allow established connections
iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT

# Allow SSH (adjust port as needed)
iptables -A INPUT -p tcp --dport 22 -m conntrack --ctstate NEW,ESTABLISHED -j ACCEPT

# Allow Panro P2P (port 9000)
iptables -A INPUT -p tcp --dport 9000 -m conntrack --ctstate NEW,ESTABLISHED -j ACCEPT
iptables -A INPUT -p udp --dport 9000 -j ACCEPT

# Allow HTTP API (restrict to specific IPs in production)
iptables -A INPUT -p tcp --dport 8080 -s 10.0.0.0/8 -j ACCEPT
iptables -A INPUT -p tcp --dport 8080 -s 172.16.0.0/12 -j ACCEPT
iptables -A INPUT -p tcp --dport 8080 -s 192.168.0.0/16 -j ACCEPT

# Allow monitoring (Prometheus)
iptables -A INPUT -p tcp --dport 8081 -s 10.0.0.0/8 -j ACCEPT

# Rate limiting for new connections
iptables -A INPUT -p tcp --dport 9000 -m conntrack --ctstate NEW -m limit --limit 25/minute --limit-burst 100 -j ACCEPT

# DDoS protection
iptables -A INPUT -p tcp --syn -m limit --limit 1/s --limit-burst 3 -j ACCEPT
iptables -A INPUT -p icmp --icmp-type echo-request -m limit --limit 1/s -j ACCEPT

# Log dropped packets
iptables -A INPUT -m limit --limit 5/min -j LOG --log-prefix "iptables denied: " --log-level 7

# Save rules
iptables-save > /etc/iptables/rules.v4

echo "Firewall configured successfully"
```

### SSL/TLS Configuration

```bash
#!/bin/bash
# setup-ssl.sh

CERT_DIR="/etc/panro/certs"
DOMAIN="beacon.example.com"

# Create certificate directory
mkdir -p "$CERT_DIR"

# Generate private key
openssl genrsa -out "$CERT_DIR/server.key" 4096

# Generate certificate signing request
openssl req -new -key "$CERT_DIR/server.key" -out "$CERT_DIR/server.csr" \
    -subj "/C=US/ST=State/L=City/O=Organization/CN=$DOMAIN"

# Generate self-signed certificate (replace with proper CA-signed cert in production)
openssl x509 -req -days 365 -in "$CERT_DIR/server.csr" \
    -signkey "$CERT_DIR/server.key" -out "$CERT_DIR/server.crt"

# Set proper permissions
chmod 600 "$CERT_DIR/server.key"
chmod 644 "$CERT_DIR/server.crt"
chown -R panro:panro "$CERT_DIR"

echo "SSL certificates generated in $CERT_DIR"
```

## Troubleshooting

### Common Issues and Solutions

```bash
# Performance troubleshooting script
#!/bin/bash

echo "=== Panro Performance Diagnostics ==="

# Check system resources
echo "1. System Resources:"
echo "CPU Usage:"
top -bn1 | grep "Cpu(s)" | sed "s/.*, *\([0-9.]*\)%* id.*/\1/" | awk '{print 100 - $1"%"}'

echo "Memory Usage:"
free -h

echo "Disk Usage:"
df -h /var/lib/panro

echo "Disk I/O:"
iostat -x 1 1

# Check network
echo "2. Network Status:"
echo "Network connections:"
netstat -tuln | grep -E "(9000|8080|8081)"

echo "Peer count:"
curl -s http://localhost:8080/eth/v1/node/peer_count | jq '.data.connected'

# Check process
echo "3. Process Status:"
echo "Panro process:"
ps aux | grep panro | grep -v grep

echo "Open files:"
lsof -p $(pgrep panro) | wc -l

# Check logs for errors
echo "4. Recent Errors:"
journalctl -u panro --since "1 hour ago" | grep -i error | tail -10

# Database stats
echo "5. Database Status:"
du -sh /var/lib/panro/db

# API health check
echo "6. API Health:"
curl -s http://localhost:8080/eth/v1/node/health || echo "API not responding"

echo "=== Diagnostics Complete ==="
```

### Log Analysis Tools

```bash
#!/bin/bash
# analyze-logs.sh

LOG_FILE="/var/log/panro/panro.log"
ANALYSIS_PERIOD="24h"

echo "=== Panro Log Analysis (Last $ANALYSIS_PERIOD) ==="

# Error summary
echo "1. Error Summary:"
journalctl -u panro --since "$ANALYSIS_PERIOD" | grep -i error | \
    awk '{print $6" "$7" "$8}' | sort | uniq -c | sort -nr | head -10

# Warning summary
echo "2. Warning Summary:"
journalctl -u panro --since "$ANALYSIS_PERIOD" | grep -i warn | \
    awk '{print $6" "$7" "$8}' | sort | uniq -c | sort -nr | head -10

# Connection issues
echo "3. Connection Issues:"
journalctl -u panro --since "$ANALYSIS_PERIOD" | grep -i "connection\|peer" | \
    grep -E "(failed|error|timeout)" | wc -l

# Sync status
echo "4. Sync Status:"
journalctl -u panro --since "$ANALYSIS_PERIOD" | grep -i "sync" | tail -5

# Performance metrics
echo "5. Performance Indicators:"
journalctl -u panro --since "$ANALYSIS_PERIOD" | grep -E "(slot|epoch|finalized)" | tail -10

echo "=== Analysis Complete ==="
```
