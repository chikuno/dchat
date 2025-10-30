# dchat Operational Guide

**Version**: 1.0  
**Last Updated**: October 28, 2025  
**Target Audience**: System administrators, DevOps engineers, node operators

---

## Table of Contents
1. [System Overview](#system-overview)
2. [Prerequisites](#prerequisites)
3. [Installation](#installation)
4. [Node Types & Operations](#node-types--operations)
5. [Configuration](#configuration)
6. [Monitoring & Observability](#monitoring--observability)
7. [Backup & Recovery](#backup--recovery)
8. [Upgrades & Maintenance](#upgrades--maintenance)
9. [Troubleshooting](#troubleshooting)
10. [Security Best Practices](#security-best-practices)
11. [Performance Tuning](#performance-tuning)
12. [Disaster Recovery](#disaster-recovery)

---

## System Overview

### Architecture
dchat operates on a **dual-chain architecture**:
- **Chat Chain**: Identity, messaging, channels, governance, reputation
- **Currency Chain**: Payments, staking, rewards, economics

### Node Types
1. **Relay Nodes**: Forward messages between peers, earn rewards via proof-of-delivery
2. **Validator Nodes**: Validate transactions on chat/currency chains, participate in consensus
3. **User Clients**: End-user messaging clients, connect to relay network
4. **Bootstrap Nodes**: Well-known entry points for peer discovery

### Components
- **P2P Network**: DHT discovery (Kademlia), gossip protocol, NAT traversal
- **Cryptography**: Noise Protocol (XX handshake), Ed25519 signatures, key rotation
- **Storage**: Local SQLite, encrypted backups, message expiration
- **Bridge**: Cross-chain atomic transactions between chat and currency chains
- **Observability**: Prometheus metrics (port 9090), health checks (port 8080)

---

## Prerequisites

### System Requirements

#### Relay Node
- **CPU**: 2 cores minimum, 4 cores recommended
- **RAM**: 4GB minimum, 8GB recommended
- **Storage**: 50GB SSD (100GB for high-traffic relays)
- **Network**: Public IP, 100 Mbps bandwidth, ports 7070 (P2P) and 8080 (HTTP)
- **OS**: Linux (Ubuntu 22.04 LTS, Debian 12), macOS 12+, Windows Server 2022

#### Validator Node
- **CPU**: 4 cores minimum, 8 cores recommended
- **RAM**: 16GB minimum, 32GB recommended
- **Storage**: 500GB NVMe SSD (chain state grows ~1GB/month)
- **Network**: Public IP, 1 Gbps bandwidth, low latency (<50ms to peers)
- **OS**: Linux (Ubuntu 22.04 LTS recommended)

#### User Client
- **CPU**: 1 core
- **RAM**: 512MB
- **Storage**: 1GB
- **Network**: Any (NAT traversal included)
- **OS**: Linux, macOS, Windows, Android, iOS

### Software Dependencies
- **Rust**: 1.70+ (for building from source)
- **SQLite**: 3.35+ (embedded, included)
- **OpenSSL**: 1.1.1+ (for TLS)
- **Docker**: 20.10+ (for containerized deployments)
- **Kubernetes**: 1.25+ (for orchestrated deployments)

---

## Installation

### From Pre-Built Binaries (Recommended)

#### Linux/macOS
```bash
# Download latest release
curl -LO https://github.com/dchat/dchat/releases/latest/download/dchat-linux-x86_64.tar.gz

# Extract
tar -xzf dchat-linux-x86_64.tar.gz

# Install to /usr/local/bin
sudo mv dchat /usr/local/bin/
sudo chmod +x /usr/local/bin/dchat

# Verify installation
dchat --version
```

#### Windows
```powershell
# Download from GitHub releases
Invoke-WebRequest -Uri https://github.com/dchat/dchat/releases/latest/download/dchat-windows-x86_64.zip -OutFile dchat.zip

# Extract
Expand-Archive -Path dchat.zip -DestinationPath C:\Program Files\dchat

# Add to PATH
$env:Path += ";C:\Program Files\dchat"

# Verify
dchat --version
```

### From Source

```bash
# Clone repository
git clone https://github.com/dchat/dchat.git
cd dchat

# Build release binary
cargo build --release

# Binary located at target/release/dchat
./target/release/dchat --version

# Install system-wide (optional)
sudo cp target/release/dchat /usr/local/bin/
```

### Docker

```bash
# Pull official image
docker pull dchat/relay:latest

# Run relay node
docker run -d \
  --name dchat-relay \
  -p 7070:7070 \
  -p 8080:8080 \
  -p 9090:9090 \
  -v /var/dchat:/data \
  dchat/relay:latest
```

### Kubernetes (Helm)

```bash
# Add dchat Helm repository
helm repo add dchat https://dchat.github.io/charts
helm repo update

# Install relay node
helm install dchat-relay dchat/relay \
  --set replicaCount=3 \
  --set service.type=LoadBalancer \
  --set persistence.size=100Gi

# Check status
kubectl get pods -l app=dchat-relay
```

---

## Node Types & Operations

### 1. Relay Node

**Purpose**: Forward messages between peers, earn rewards

#### Quick Start
```bash
# Generate identity for relay
dchat keygen --output /etc/dchat/relay-identity.json

# Run relay node
dchat relay \
  --identity /etc/dchat/relay-identity.json \
  --listen 0.0.0.0:7070 \
  --bootstrap /ip4/seed1.dchat.org/tcp/7070/p2p/12D3KooWABC... \
  --bootstrap /ip4/seed2.dchat.org/tcp/7070/p2p/12D3KooWXYZ... \
  --data-dir /var/dchat/relay \
  --log-level info \
  --json-logs
```

#### Configuration File (`relay-config.toml`)
```toml
[node]
identity_file = "/etc/dchat/relay-identity.json"
data_dir = "/var/dchat/relay"
log_level = "info"
json_logs = true

[network]
listen_address = "0.0.0.0:7070"
max_peers = 50
bootstrap_nodes = [
    "/ip4/seed1.dchat.org/tcp/7070/p2p/12D3KooWABC...",
    "/ip4/seed2.dchat.org/tcp/7070/p2p/12D3KooWXYZ..."
]

[nat]
enable_upnp = true
enable_stun = true
stun_server = "stun.l.google.com:19302"
enable_turn = true
turn_server = "turn.dchat.org:3478"
turn_username = "relay1"
turn_password = "secret"

[relay]
enable = true
max_queue_size = 10000
delivery_timeout = 300  # 5 minutes
proof_of_delivery = true

[storage]
backend = "sqlite"
db_path = "/var/dchat/relay/relay.db"
max_message_age = 604800  # 7 days in seconds
pruning_interval = 3600   # 1 hour

[observability]
metrics_port = 9090
health_port = 8080
tracing_enabled = true
```

#### Run with Config File
```bash
dchat relay --config /etc/dchat/relay-config.toml
```

#### Systemd Service (`/etc/systemd/system/dchat-relay.service`)
```ini
[Unit]
Description=dchat Relay Node
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=dchat
Group=dchat
ExecStart=/usr/local/bin/dchat relay --config /etc/dchat/relay-config.toml
Restart=on-failure
RestartSec=10s
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=full
ProtectHome=true
ReadWritePaths=/var/dchat/relay

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
# Create dchat user
sudo useradd -r -s /bin/false dchat
sudo mkdir -p /var/dchat/relay
sudo chown dchat:dchat /var/dchat/relay

# Install service
sudo systemctl daemon-reload
sudo systemctl enable dchat-relay
sudo systemctl start dchat-relay

# Check status
sudo systemctl status dchat-relay
journalctl -u dchat-relay -f
```

---

### 2. Validator Node

**Purpose**: Validate transactions, participate in consensus

#### Quick Start
```bash
# Generate validator identity
dchat keygen --output /etc/dchat/validator-identity.json

# Run validator
dchat validator \
  --identity /etc/dchat/validator-identity.json \
  --chain-id dchat-mainnet-1 \
  --listen 0.0.0.0:26656 \
  --rpc-listen 0.0.0.0:26657 \
  --data-dir /var/dchat/validator \
  --bootstrap /ip4/validator1.dchat.org/tcp/26656 \
  --stake 1000000  # Minimum stake: 1M tokens
```

#### Configuration File (`validator-config.toml`)
```toml
[validator]
identity_file = "/etc/dchat/validator-identity.json"
chain_id = "dchat-mainnet-1"
data_dir = "/var/dchat/validator"
moniker = "my-validator"

[network]
listen_address = "0.0.0.0:26656"
rpc_listen_address = "0.0.0.0:26657"
max_peers = 100
bootstrap_validators = [
    "/ip4/validator1.dchat.org/tcp/26656",
    "/ip4/validator2.dchat.org/tcp/26656"
]

[consensus]
timeout_propose = 3000    # 3 seconds
timeout_prevote = 1000    # 1 second
timeout_precommit = 1000  # 1 second
timeout_commit = 5000     # 5 seconds
create_empty_blocks = false
max_block_size = 10485760  # 10MB

[staking]
initial_stake = 1000000  # 1M tokens
min_self_delegation = 100000
unbonding_time = 1209600  # 14 days in seconds

[slashing]
downtime_jail_duration = 600  # 10 minutes
double_sign_jail_duration = 86400  # 24 hours
missed_blocks_window = 10000
min_signed_per_window = 0.5  # 50%

[database]
backend = "rocksdb"  # or "sqlite"
path = "/var/dchat/validator/chain.db"
cache_size = 2048  # MB
```

#### Systemd Service
```ini
[Unit]
Description=dchat Validator Node
After=network.target

[Service]
Type=simple
User=dchat
Group=dchat
ExecStart=/usr/local/bin/dchat validator --config /etc/dchat/validator-config.toml
Restart=on-failure
RestartSec=5s
LimitNOFILE=65536

# Security
NoNewPrivileges=true
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

---

### 3. User Client

**Purpose**: End-user messaging application

#### Quick Start
```bash
# Generate user identity
dchat keygen --output ~/.dchat/identity.json

# Run user client (interactive mode)
dchat user \
  --identity ~/.dchat/identity.json \
  --bootstrap /ip4/seed1.dchat.org/tcp/7070/p2p/12D3KooWABC...
```

#### Configuration File (`~/.dchat/config.toml`)
```toml
[user]
identity_file = "~/.dchat/identity.json"
data_dir = "~/.dchat/data"
log_level = "warn"

[network]
bootstrap_nodes = [
    "/ip4/seed1.dchat.org/tcp/7070/p2p/12D3KooWABC...",
    "/ip4/seed2.dchat.org/tcp/7070/p2p/12D3KooWXYZ..."
]
max_relay_connections = 5

[nat]
enable_upnp = true
enable_stun = true

[storage]
local_db = "~/.dchat/data/messages.db"
max_local_messages = 10000
message_retention_days = 30

[backup]
enable = true
backup_interval = 86400  # Daily
encrypted_backup = true
backup_path = "~/.dchat/backups"

[privacy]
enable_onion_routing = true
cover_traffic = false  # Disable to save bandwidth
```

#### Desktop Application (GUI)
```bash
# Run with GUI (if built with UI features)
dchat user --gui
```

---

### 4. Bootstrap Node

**Purpose**: Well-known entry points for peer discovery

Bootstrap nodes are relay nodes with publicly advertised addresses:

```bash
# Run bootstrap relay with fixed identity
dchat relay \
  --identity /etc/dchat/bootstrap-identity.json \
  --listen 0.0.0.0:7070 \
  --public-address /ip4/203.0.113.10/tcp/7070 \
  --data-dir /var/dchat/bootstrap \
  --log-level info
```

Publish your bootstrap node multiaddr:
```
/ip4/203.0.113.10/tcp/7070/p2p/12D3KooWABC...
```

---

## Configuration

### Configuration File Locations

| Node Type | Linux/macOS | Windows |
|-----------|-------------|---------|
| **Relay** | `/etc/dchat/relay-config.toml` | `C:\Program Files\dchat\relay-config.toml` |
| **Validator** | `/etc/dchat/validator-config.toml` | `C:\Program Files\dchat\validator-config.toml` |
| **User** | `~/.dchat/config.toml` | `%USERPROFILE%\.dchat\config.toml` |

### Environment Variables

All configuration options can be overridden via environment variables:

```bash
export DCHAT_IDENTITY_FILE=/path/to/identity.json
export DCHAT_DATA_DIR=/var/dchat
export DCHAT_LOG_LEVEL=debug
export DCHAT_LISTEN_ADDRESS=0.0.0.0:7070
export DCHAT_BOOTSTRAP_NODES="/ip4/seed1.dchat.org/tcp/7070/p2p/12D3KooWABC..."
export DCHAT_ENABLE_UPNP=true
export DCHAT_MAX_PEERS=50
```

### Configuration Validation

```bash
# Validate config file syntax
dchat relay --config /etc/dchat/relay-config.toml --validate

# Check configuration and exit
dchat relay --config /etc/dchat/relay-config.toml --dry-run
```

### Secure Key Management

#### AWS KMS Integration
```bash
# Use AWS KMS for key storage
dchat relay \
  --hsm \
  --kms-key-id arn:aws:kms:us-east-1:123456789012:key/abc123 \
  --aws-region us-east-1
```

#### Hardware Security Module (HSM)
```bash
# Use PKCS#11 HSM
dchat validator \
  --hsm \
  --pkcs11-library /usr/lib/libpkcs11.so \
  --pkcs11-slot 0 \
  --pkcs11-pin-file /etc/dchat/hsm-pin.txt
```

---

## Monitoring & Observability

### Health Checks

#### HTTP Endpoints
```bash
# Liveness probe (is the process running?)
curl http://localhost:8080/health
# Response: {"status":"ok","version":"1.0.0","timestamp":1698518400}

# Readiness probe (is it ready to serve traffic?)
curl http://localhost:8080/ready
# Response: {"ready":true,"connected_peers":45}

# Detailed status
curl http://localhost:8080/status
# Response: JSON with peer count, message queue size, uptime, etc.
```

#### Command-Line Health Check
```bash
dchat health --url http://localhost:8080/health
```

### Prometheus Metrics

Metrics are exposed on port **9090** at `/metrics`:

```bash
curl http://localhost:9090/metrics
```

#### Key Metrics

**Relay Node**:
- `dchat_relay_messages_forwarded_total` (counter)
- `dchat_relay_delivery_success_rate` (gauge, 0-1)
- `dchat_relay_queue_size` (gauge)
- `dchat_relay_uptime_seconds` (gauge)
- `dchat_network_connected_peers` (gauge)
- `dchat_network_bandwidth_bytes_total` (counter, labels: direction=in/out)

**Validator Node**:
- `dchat_validator_blocks_validated_total` (counter)
- `dchat_validator_block_time_seconds` (histogram)
- `dchat_validator_missed_blocks_total` (counter)
- `dchat_validator_stake_amount` (gauge)
- `dchat_chain_height` (gauge)

**User Client**:
- `dchat_messages_sent_total` (counter)
- `dchat_messages_received_total` (counter)
- `dchat_message_latency_seconds` (histogram)

### Prometheus Configuration (`prometheus.yml`)

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'dchat-relay'
    static_configs:
      - targets:
        - relay1.dchat.org:9090
        - relay2.dchat.org:9090
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance

  - job_name: 'dchat-validator'
    static_configs:
      - targets:
        - validator1.dchat.org:9090

  - job_name: 'dchat-user'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app]
        action: keep
        regex: dchat-user
```

### Grafana Dashboards

Pre-built dashboards are available in `monitoring/dashboards/`:

1. **Relay Node Overview** (`relay-overview.json`)
   - Message throughput
   - Delivery success rate
   - Peer connections
   - Queue sizes

2. **Validator Performance** (`validator-performance.json`)
   - Block validation time
   - Missed blocks
   - Stake amount
   - Uptime

3. **Network Health** (`network-health.json`)
   - Total active peers
   - DHT performance
   - NAT traversal success rates
   - Gossip propagation latency

#### Import Dashboards
```bash
# Using Grafana API
curl -X POST \
  -H "Authorization: Bearer ${GRAFANA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d @monitoring/dashboards/relay-overview.json \
  http://grafana.example.com/api/dashboards/db
```

### Alerting

#### PagerDuty Integration (`monitoring/alert-rules.yaml`)
```yaml
groups:
  - name: dchat-critical
    interval: 30s
    rules:
      - alert: RelayNodeDown
        expr: up{job="dchat-relay"} == 0
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Relay node {{ $labels.instance }} is down"
          description: "Relay has been unreachable for 5 minutes"

      - alert: HighMessageQueueSize
        expr: dchat_relay_queue_size > 5000
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High message queue on {{ $labels.instance }}"
          description: "Queue size: {{ $value }}"

      - alert: ValidatorMissedBlocks
        expr: rate(dchat_validator_missed_blocks_total[5m]) > 0.1
        for: 15m
        labels:
          severity: critical
        annotations:
          summary: "Validator {{ $labels.instance }} missing blocks"
          description: "Missed block rate: {{ $value | humanizePercentage }}"
```

### Distributed Tracing

dchat supports **OpenTelemetry** for distributed tracing:

```toml
# relay-config.toml
[tracing]
enabled = true
exporter = "jaeger"  # or "zipkin", "otlp"
jaeger_endpoint = "http://jaeger-collector:14268/api/traces"
service_name = "dchat-relay"
sample_rate = 0.1  # Sample 10% of traces
```

View traces in Jaeger UI:
```bash
docker run -d --name jaeger \
  -p 16686:16686 \
  -p 14268:14268 \
  jaegertracing/all-in-one:latest
```

Navigate to: `http://localhost:16686`

---

## Backup & Recovery

### Automatic Backups

#### Relay Node Backup
```bash
# Backup relay database
dchat database backup \
  --db-path /var/dchat/relay/relay.db \
  --output /backup/relay-$(date +%Y%m%d).db.gz \
  --compress
```

#### Scheduled Backups (Cron)
```bash
# /etc/cron.d/dchat-backup
0 2 * * * dchat /usr/local/bin/dchat database backup --db-path /var/dchat/relay/relay.db --output /backup/relay-$(date +\%Y\%m\%d).db.gz --compress
```

### Identity Backup

```bash
# Backup identity (CRITICAL - keep offline!)
cp /etc/dchat/relay-identity.json /backup/relay-identity-$(date +%Y%m%d).json.gpg
gpg --encrypt --recipient admin@example.com /backup/relay-identity-*.json
shred -u /backup/relay-identity-*.json  # Securely delete unencrypted copy
```

### Disaster Recovery

#### Full Chain State Replay
```bash
# Replay entire chain from genesis
dchat validator \
  --replay-from-genesis \
  --genesis-file /backup/genesis.json \
  --data-dir /var/dchat/validator-recovery
```

#### Restore from Snapshot
```bash
# Download latest snapshot
wget https://snapshots.dchat.org/latest.tar.gz

# Extract and restore
tar -xzf latest.tar.gz -C /var/dchat/validator/

# Restart validator
sudo systemctl restart dchat-validator
```

### Encrypted Cloud Backup

```toml
# config.toml
[backup]
enable = true
backend = "s3"  # or "gcs", "azure", "ipfs"
s3_bucket = "dchat-backups"
s3_region = "us-east-1"
encryption = "aes256"
encryption_key_file = "/etc/dchat/backup-key.txt"
schedule = "0 2 * * *"  # Daily at 2 AM
retention_days = 30
```

---

## Upgrades & Maintenance

### Software Upgrades

#### Zero-Downtime Upgrade (Relay Nodes)
```bash
# Download new version
wget https://github.com/dchat/dchat/releases/download/v1.1.0/dchat-linux-x86_64.tar.gz
tar -xzf dchat-linux-x86_64.tar.gz

# Validate new binary
./dchat --version
./dchat relay --config /etc/dchat/relay-config.toml --validate

# Graceful restart
sudo systemctl reload dchat-relay  # Sends SIGHUP, waits for connections to drain
```

#### Validator Upgrade (Coordinated)
Validator upgrades require coordination to avoid downtime:

1. **Announce upgrade** in governance (24-hour notice)
2. **Coordinate upgrade block height** (e.g., block 1,000,000)
3. **All validators upgrade at the same time**
4. **Use upgrade handler**:

```bash
# Automated upgrade at specific block height
dchat validator \
  --upgrade-name v1.1.0 \
  --upgrade-height 1000000 \
  --upgrade-binary /opt/dchat/dchat-v1.1.0
```

### Configuration Updates

```bash
# Reload config without restarting
sudo systemctl reload dchat-relay

# Validate before applying
dchat relay --config /etc/dchat/relay-config.toml --validate
```

### Database Migrations

```bash
# Check current schema version
dchat database version --db-path /var/dchat/relay/relay.db

# Apply pending migrations
dchat database migrate --db-path /var/dchat/relay/relay.db

# Rollback last migration (if needed)
dchat database rollback --db-path /var/dchat/relay/relay.db --steps 1
```

### Pruning Old Data

```bash
# Prune messages older than 30 days
dchat database prune \
  --db-path /var/dchat/relay/relay.db \
  --older-than 30d \
  --dry-run  # Preview what will be deleted

# Execute pruning
dchat database prune --db-path /var/dchat/relay/relay.db --older-than 30d
```

---

## Troubleshooting

### Common Issues

#### 1. Relay Node Not Receiving Connections

**Symptoms**: `dchat_network_connected_peers = 0`, no incoming messages

**Diagnosis**:
```bash
# Check if port 7070 is open
sudo netstat -tulpn | grep 7070

# Test connectivity from outside
telnet your-public-ip 7070

# Check firewall
sudo ufw status
sudo iptables -L -n | grep 7070
```

**Solutions**:
```bash
# Open port in firewall
sudo ufw allow 7070/tcp

# Enable UPnP (if behind NAT)
# Edit relay-config.toml:
[nat]
enable_upnp = true

# Configure port forwarding manually (router)
# Forward external 7070 -> internal 7070

# Use STUN/TURN fallback
[nat]
enable_stun = true
stun_server = "stun.l.google.com:19302"
enable_turn = true
turn_server = "turn.dchat.org:3478"
```

---

#### 2. High Memory Usage

**Symptoms**: OOM kills, `dchat_memory_usage_bytes` increasing

**Diagnosis**:
```bash
# Check memory usage
ps aux | grep dchat
pmap $(pgrep dchat) | tail -1

# Check message queue size
curl http://localhost:8080/status | jq .queue_size
```

**Solutions**:
```toml
# Reduce max peers
[network]
max_peers = 25  # Default: 50

# Reduce message queue size
[relay]
max_queue_size = 5000  # Default: 10000

# Enable aggressive pruning
[storage]
max_message_age = 86400  # 1 day instead of 7
pruning_interval = 1800  # 30 minutes instead of 1 hour

# Limit database cache
[database]
cache_size = 256  # MB (default: 2048)
```

---

#### 3. Validator Missing Blocks

**Symptoms**: `dchat_validator_missed_blocks_total` increasing, slashing warnings

**Diagnosis**:
```bash
# Check validator logs
journalctl -u dchat-validator -n 100

# Check network latency
ping validator1.dchat.org
mtr validator1.dchat.org

# Check system resources
top
iostat -x 1
```

**Solutions**:
```bash
# Increase hardware resources (CPU, RAM, disk IOPS)

# Reduce consensus timeouts (if network is fast)
[consensus]
timeout_propose = 2000  # 2s instead of 3s
timeout_prevote = 500   # 500ms instead of 1s

# Connect to more peers
[network]
max_peers = 150  # Default: 100

# Use faster database backend
[database]
backend = "rocksdb"  # Instead of sqlite
cache_size = 4096  # Increase cache
```

---

#### 4. Messages Not Delivering

**Symptoms**: Messages stuck in queue, `dchat_relay_delivery_success_rate < 0.9`

**Diagnosis**:
```bash
# Check relay logs
journalctl -u dchat-relay | grep "delivery failed"

# Check peer connectivity
curl http://localhost:8080/status | jq .peers

# Test DHT lookup
dchat network peer-lookup --peer-id 12D3KooWABC...
```

**Solutions**:
```toml
# Increase delivery timeout
[relay]
delivery_timeout = 600  # 10 minutes instead of 5

# Enable more aggressive retries
[relay]
max_delivery_attempts = 5  # Default: 3
retry_backoff_seconds = 10  # Default: 30

# Use more bootstrap nodes
[network]
bootstrap_nodes = [
    "/ip4/seed1.dchat.org/tcp/7070/p2p/12D3KooWABC...",
    "/ip4/seed2.dchat.org/tcp/7070/p2p/12D3KooWXYZ...",
    "/ip4/seed3.dchat.org/tcp/7070/p2p/12D3KooWDEF..."
]
```

---

#### 5. Database Corruption

**Symptoms**: Crashes with "database is malformed", SQLite errors

**Diagnosis**:
```bash
# Check database integrity
sqlite3 /var/dchat/relay/relay.db "PRAGMA integrity_check;"
```

**Solutions**:
```bash
# Restore from backup
sudo systemctl stop dchat-relay
cp /backup/relay-20251027.db.gz /var/dchat/relay/
gunzip /var/dchat/relay/relay-20251027.db.gz
mv /var/dchat/relay/relay-20251027.db /var/dchat/relay/relay.db
sudo systemctl start dchat-relay

# Rebuild from scratch (last resort)
sudo systemctl stop dchat-relay
rm /var/dchat/relay/relay.db
dchat database migrate --db-path /var/dchat/relay/relay.db
sudo systemctl start dchat-relay
```

---

### Debug Logging

Enable verbose logging for troubleshooting:

```bash
# Temporary debug logging
dchat relay --log-level debug

# Or via environment variable
export DCHAT_LOG_LEVEL=debug
dchat relay

# Trace specific modules
export RUST_LOG=dchat_network=trace,dchat_relay=debug
dchat relay
```

---

## Security Best Practices

### 1. Identity Protection
- **Never share** identity JSON files (`identity.json`)
- Store identity on **encrypted volumes**
- Use **HSM/KMS** for validator keys
- Backup identity to **offline storage** (USB, paper wallet)

### 2. Network Security
- **Firewall**: Only expose necessary ports (7070, 8080, 9090)
- **TLS**: Use TLS for HTTP endpoints (`--tls-cert`, `--tls-key`)
- **DDoS Protection**: Use Cloudflare, fail2ban, rate limiting
- **VPN**: Connect validators via private VPN mesh

### 3. Access Control
```bash
# Restrict file permissions
sudo chmod 600 /etc/dchat/relay-identity.json
sudo chown dchat:dchat /etc/dchat/relay-identity.json

# Restrict HTTP endpoints to localhost
[observability]
metrics_address = "127.0.0.1:9090"  # Not 0.0.0.0
health_address = "127.0.0.1:8080"

# Use reverse proxy (nginx) for public access
# with authentication
```

### 4. Regular Security Updates
```bash
# Subscribe to security announcements
# https://github.com/dchat/dchat/security/advisories

# Check for updates weekly
curl -s https://api.github.com/repos/dchat/dchat/releases/latest | jq .tag_name
```

---

## Performance Tuning

### System-Level Tuning

#### Linux Kernel Parameters (`/etc/sysctl.conf`)
```bash
# Increase file descriptor limit
fs.file-max = 2097152

# Network tuning
net.core.somaxconn = 65535
net.ipv4.tcp_max_syn_backlog = 8096
net.ipv4.ip_local_port_range = 1024 65535
net.core.netdev_max_backlog = 16384

# Memory tuning
vm.swappiness = 10
vm.dirty_ratio = 15
vm.dirty_background_ratio = 5

# Apply changes
sudo sysctl -p
```

#### Increase File Descriptors
```bash
# /etc/security/limits.conf
dchat soft nofile 65536
dchat hard nofile 65536
```

### Application-Level Tuning

#### Relay Node Performance
```toml
[network]
max_peers = 100  # Increase for high-traffic relays
connection_pool_size = 200

[relay]
worker_threads = 8  # Match CPU core count
max_queue_size = 20000

[database]
cache_size = 4096  # MB (increase for high message volume)
write_buffer_size = 64  # MB
max_open_files = 1000
```

#### Validator Node Performance
```toml
[database]
backend = "rocksdb"  # Faster than SQLite
cache_size = 8192  # MB
write_buffer_size = 128  # MB
compaction_threads = 4

[consensus]
create_empty_blocks = false  # Save resources
max_block_size = 20971520  # 20MB (if network allows)
```

### Benchmarking

```bash
# Run performance benchmarks
cargo bench --package dchat-network

# Relay performance test
dchat benchmark relay \
  --duration 60s \
  --message-rate 1000  # messages per second

# Validator performance test
dchat benchmark validator \
  --duration 60s \
  --transaction-rate 500
```

---

## Disaster Recovery

### Scenario 1: Single Node Failure

**Recovery**:
1. Restore identity from backup
2. Restore database from latest snapshot
3. Restart node
4. Wait for peer discovery and sync

```bash
# Restore identity
gpg --decrypt /backup/relay-identity.json.gpg > /etc/dchat/relay-identity.json

# Restore database
cp /backup/relay-latest.db /var/dchat/relay/relay.db

# Restart
sudo systemctl start dchat-relay
```

---

### Scenario 2: Network Partition

**Detection**:
- Gossip messages not propagating
- Validator block heights diverge
- `dchat_network_partition_detected = 1`

**Recovery**:
1. Wait for partition to heal (automatic)
2. If persistent, activate bridge nodes
3. Manual peer connections

```bash
# Manually connect to specific peers
dchat network connect --peer-id 12D3KooWABC... --address /ip4/203.0.113.10/tcp/7070

# Force bridge activation
dchat network activate-bridge --partition-id A --partition-id B
```

---

### Scenario 3: Consensus Stall (Validators)

**Symptoms**: Block height not increasing, validators stuck

**Recovery**:
1. Check if 2/3+ validators are online
2. Coordinate validator restarts
3. If stalled >1 hour, trigger recovery leader election

```bash
# Initiate recovery (requires 2/3+ validators)
dchat validator recover \
  --initiate-leader-election \
  --min-validators 5
```

---

### Scenario 4: Data Corruption (Chain State)

**Recovery**:
1. Download trusted snapshot
2. Verify integrity (checksum, signature)
3. Restore and replay

```bash
# Download snapshot
wget https://snapshots.dchat.org/chain-state-20251028.tar.gz
wget https://snapshots.dchat.org/chain-state-20251028.tar.gz.sig

# Verify signature
gpg --verify chain-state-20251028.tar.gz.sig

# Restore
sudo systemctl stop dchat-validator
tar -xzf chain-state-20251028.tar.gz -C /var/dchat/validator/
sudo systemctl start dchat-validator
```

---

## Appendix

### Ports Reference

| Port | Protocol | Purpose |
|------|----------|---------|
| **7070** | TCP | P2P communication (libp2p) |
| **8080** | HTTP | Health checks, status API |
| **9090** | HTTP | Prometheus metrics |
| **26656** | TCP | Validator P2P (Tendermint-style) |
| **26657** | HTTP | Validator RPC |
| **3478** | UDP/TCP | TURN relay (NAT traversal) |

### Command Reference

```bash
# Node Operations
dchat relay --config <file>           # Run relay node
dchat validator --config <file>       # Run validator node
dchat user --config <file>            # Run user client
dchat health --url <url>              # Check node health

# Identity Management
dchat keygen --output <file>          # Generate new identity
dchat keygen --burner --output <file> # Generate burner identity
dchat identity verify --file <file>   # Verify identity format

# Database Operations
dchat database migrate --db-path <path>    # Apply migrations
dchat database backup --db-path <path> --output <file>  # Backup
dchat database restore --db-path <path> --input <file>  # Restore
dchat database prune --db-path <path> --older-than <duration>  # Prune old data
dchat database version --db-path <path>   # Show schema version

# Network Diagnostics
dchat network peer-list               # List connected peers
dchat network peer-lookup --peer-id <id>  # Lookup peer by ID
dchat network connect --peer-id <id> --address <multiaddr>  # Connect to peer
dchat network stats                   # Show network statistics

# Benchmarking
dchat benchmark relay --duration <time>   # Relay performance test
dchat benchmark validator --duration <time>  # Validator performance test
```

### Useful Links
- **GitHub**: https://github.com/dchat/dchat
- **Documentation**: https://docs.dchat.org
- **Discord**: https://discord.gg/dchat
- **Forum**: https://forum.dchat.org
- **Status Page**: https://status.dchat.org

---

**End of Operational Guide**

For additional support, consult:
- `ARCHITECTURE.md` - System design
- `IMPLEMENTATION_STATUS.md` - Feature status
- `SECURITY_MODEL.md` - Security assumptions
- `API_SPECIFICATION.md` - API contracts
- `TROUBLESHOOTING.md` - Extended troubleshooting guide

**Version**: 1.0  
**Last Updated**: October 28, 2025
