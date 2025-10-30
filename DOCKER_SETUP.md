# Docker Configuration & Setup Guide for dchat Testnet

## Overview

This guide walks you through configuring and deploying the dchat testnet using Docker and Docker Compose. The infrastructure includes:

- **3 Relay Nodes** (containerized, auto-bootstrapping)
- **PostgreSQL** database (persistent storage)
- **Prometheus** metrics collection (9093)
- **Grafana** visualization dashboard (3000)
- **Jaeger** distributed tracing (16686)

## Prerequisites

✅ **Already Completed**:
- Docker Desktop v4.49.0 installed
- WSL-2 backend configured
- `Dockerfile` (multi-stage build)
- `docker-compose.yml` (full stack)
- `testnet-config.toml` (relay configuration)
- `testnet-deploy.ps1` (PowerShell script)

## Quick Start (5 minutes)

### 1. Start Docker Desktop

**Windows 11**:
```powershell
# Option A: GUI - Click "Docker Desktop" from Start menu or taskbar
# Option B: CLI - Start the service
Start-Service Docker-Desktop
```

**Verify Docker is running**:
```powershell
docker --version
docker ps
```

Expected output:
```
Docker version 4.49.0, build (some-build-id)
CONTAINER ID   IMAGE     COMMAND   CREATED   STATUS    PORTS     NAMES
(empty table - no containers running yet)
```

### 2. Verify Docker Configuration Files

All required files should exist:
```powershell
# Check structure
Test-Path "docker-compose.yml"        # Should be True
Test-Path "Dockerfile"                 # Should be True
Test-Path "testnet-config.toml"        # Should be True
Test-Path "monitoring\prometheus.yml"  # Should be True
```

### 3. Create Relay Configuration Files

Copy the testnet configuration to each relay node:

```powershell
# Create config directory
New-Item -ItemType Directory -Path "config" -Force | Out-Null

# Copy configuration for each relay
Copy-Item "testnet-config.toml" "config\relay1.toml" -Force
Copy-Item "testnet-config.toml" "config\relay2.toml" -Force
Copy-Item "testnet-config.toml" "config\relay3.toml" -Force

# Verify
ls config\relay*.toml
```

### 4. Deploy Full Stack

```powershell
# Navigate to project root
cd c:\Users\USER\dchat

# Pull latest images and start all services
docker-compose up -d

# This will:
# 1. Create dchat-network
# 2. Start PostgreSQL (wait ~10 sec for init)
# 3. Build relay image from Dockerfile (first run ~5-10 min)
# 4. Start relay1 (bootstrap node)
# 5. Start relay2 & relay3 (bootstrap from relay1)
# 6. Start Prometheus (scrape metrics)
# 7. Start Grafana (fetch from Prometheus)
# 8. Start Jaeger (collect traces)
```

**Estimated time**: 
- First deployment: ~10-15 minutes (building relay image)
- Subsequent deployments: ~30 seconds (images cached)

### 5. Verify All Services are Running

```powershell
# Check service status
docker-compose ps

# Expected output:
# NAME              STATUS              PORTS
# dchat-postgres    Up (healthy)        5432
# dchat-relay1      Up (healthy)        0.0.0.0:7070->7070/tcp, 9090->9090/tcp
# dchat-relay2      Up (healthy)        0.0.0.0:7072->7072/tcp, 9091->9091/tcp
# dchat-relay3      Up (healthy)        0.0.0.0:7074->7074/tcp, 9092->9092/tcp
# dchat-prometheus  Up                  0.0.0.0:9093->9090/tcp
# dchat-grafana     Up                  0.0.0.0:3000->3000/tcp
# dchat-jaeger      Up                  0.0.0.0:16686->16686/tcp

# Check individual service health
docker healthcheck dchat-relay1
docker healthcheck dchat-postgres
```

### 6. Access Monitoring Dashboards

Open in browser (or use PowerShell curl):

```powershell
# Relay1 Health Check
$response = curl http://localhost:8080/health
Write-Host "Relay1 Health: $response"

# Prometheus (raw metrics)
Start-Process "http://localhost:9093"

# Grafana (dashboards)
Start-Process "http://localhost:3000"
# Login: admin / admin

# Jaeger (tracing)
Start-Process "http://localhost:16686"
```

### 7. Monitor Relay Logs

```powershell
# Watch relay1 bootstrap (shows peer discovery)
docker logs -f dchat-relay1 | Select-String -Pattern "peer|connected|bootstrap"

# Watch relay2 connecting to relay1
docker logs -f dchat-relay2

# Watch all relays together
docker-compose logs -f relay1 relay2 relay3
```

---

## Deployment Verification Checklist

### ✅ Docker Daemon
- [ ] Docker Desktop application is running
- [ ] `docker ps` returns empty table (no error)
- [ ] `docker version` shows v4.49.0

### ✅ Configuration Files
- [ ] `docker-compose.yml` exists and valid YAML
- [ ] `Dockerfile` exists with multi-stage build
- [ ] `testnet-config.toml` exists in root
- [ ] `config/relay{1,2,3}.toml` exist (copies of testnet-config.toml)
- [ ] `monitoring/prometheus.yml` exists

### ✅ All Services Running
```powershell
docker-compose ps | grep -E "Up|healthy"
# Should show 7 services all "Up"
```

| Service | Port | Expected Status |
|---------|------|-----------------|
| PostgreSQL | 5432 | (healthy) |
| relay1 | 7070, 9090 | (healthy) |
| relay2 | 7072, 9091 | (healthy) |
| relay3 | 7074, 9092 | (healthy) |
| Prometheus | 9093 | Up |
| Grafana | 3000 | Up |
| Jaeger | 16686 | Up |

### ✅ Network Connectivity

```powershell
# Test relay1 health endpoint
curl http://localhost:8080/health

# Expected output:
# {"status":"healthy","version":"0.1.0","peers":2}

# Test relay ports (should accept connections)
Test-NetConnection localhost -Port 7070 -WarningAction SilentlyContinue
Test-NetConnection localhost -Port 7072 -WarningAction SilentlyContinue
Test-NetConnection localhost -Port 7074 -WarningAction SilentlyContinue
```

### ✅ Metrics Collection

```powershell
# Verify Prometheus can scrape relay metrics
curl http://localhost:9093/api/v1/query?query=up

# Expected output:
# {"status":"success","data":{"resultType":"vector","result":[
#   {"metric":{"job":"dchat-relay1"},"value":[<timestamp>,"1"]},
#   {"metric":{"job":"dchat-relay2"},"value":[<timestamp>,"1"]},
#   {"metric":{"job":"dchat-relay3"},"value":[<timestamp>,"1"]}
# ]}}
```

### ✅ Grafana Dashboard

```powershell
# Login to Grafana
Start-Process "http://localhost:3000"
# Username: admin
# Password: admin

# Once logged in:
# 1. Home > Dashboards > dchat-overview (should exist)
# 2. Should show relay node status, message counts, peer connections
# 3. All graphs should have data (red/pink lines = healthy)
```

---

## Common Operations

### Start Testnet

```powershell
# Start all services in background
docker-compose up -d

# Or with logging
docker-compose up
```

### Stop Testnet

```powershell
# Stop all services (containers remain)
docker-compose stop

# Stop and remove containers
docker-compose down
```

### Restart Services

```powershell
# Restart specific service
docker-compose restart relay1

# Restart all services
docker-compose restart
```

### Clean Reset

```powershell
# Remove all containers, networks, volumes (DELETES DATA)
docker-compose down -v

# Start fresh
docker-compose up -d
```

### View Logs

```powershell
# Real-time logs from relay1
docker-compose logs -f relay1

# Last 50 lines
docker-compose logs --tail 50

# With timestamps
docker-compose logs -f --timestamps
```

### Inspect Services

```powershell
# See all running processes
docker-compose ps

# Inspect network
docker network ls
docker network inspect dchat-network

# Check volume sizes
docker volume ls
docker volume inspect dchat_relay1_data
```

---

## Troubleshooting

### Issue: "Docker daemon is not running"

**Solution**:
```powershell
# Start Docker Desktop
Start-Service Docker-Desktop
# Wait 2-3 minutes for daemon to initialize

# Verify
docker ps
```

### Issue: "docker-compose command not found"

**Solution**: Docker Desktop includes docker-compose. Verify installation:
```powershell
docker-compose --version
# Should show: Docker Compose version 2.x.x
```

### Issue: "Port already in use" (e.g., port 3000)

**Solution**:
```powershell
# Find process using port 3000
Get-NetStatistics -LocalPort 3000 -Protocol TCP

# Kill process or use different port
# Edit docker-compose.yml: "3001:3000" (external:internal)
```

### Issue: Relay nodes won't start / "Exit code 1"

**Causes & Solutions**:
```powershell
# Check logs
docker logs dchat-relay1

# Common issues:
# 1. Config file not found → Verify config/relay1.toml exists
# 2. Port conflict → Check ports 7070, 7072, 7074 not in use
# 3. Database not ready → Wait 10 sec, restart relay nodes

# Solution: Full reset
docker-compose down -v
docker-compose up -d
```

### Issue: Grafana shows "No data"

**Causes & Solutions**:
```powershell
# 1. Prometheus not scraping metrics
curl http://localhost:9093/targets
# Should show all 3 relays with status "UP"

# 2. Datasource misconfigured
# In Grafana UI: Configuration > Data Sources > Prometheus
# Verify URL is "http://prometheus:9090" (docker network name)

# 3. Relays not exporting metrics
docker logs dchat-relay1 | grep -i "metrics\|prometheus"
```

### Issue: PostgreSQL won't start / "Permission denied"

**Solution**:
```powershell
# Remove stuck volumes
docker volume rm dchat_postgres_data

# Restart
docker-compose up -d postgres
```

### Issue: "No space left on device"

**Solution**:
```powershell
# Clean up unused Docker objects
docker system prune -a

# Check disk space
Get-PSDrive C
```

---

## Advanced Configuration

### Using Alternative Configuration

To use a custom configuration instead of testnet-config.toml:

```powershell
# Edit config/relay1.toml
notepad config\relay1.toml

# Changes take effect on restart:
docker-compose restart relay1
```

### Scaling to More Relays

To deploy 5 relays instead of 3:

```powershell
# Modify docker-compose.yml
notepad docker-compose.yml

# Add relay4, relay5 services with new ports:
# relay4: 7076, 9094
# relay5: 7078, 9095

# Update prometheus.yml with new targets
notepad monitoring\prometheus.yml

# Restart
docker-compose down
docker-compose up -d
```

### Connecting External Clients

To connect user nodes to the testnet:

```powershell
# Get relay1 peer ID
docker logs dchat-relay1 | grep -i "peer\|identity"

# Use relay1 as bootstrap:
# Relay1 P2P: localhost:7070
# Relay2 P2P: localhost:7072
# Relay3 P2P: localhost:7074

# In user client config:
# [network]
# bootstrap_peers = ["/ip4/127.0.0.1/tcp/7070/p2p/<relay1-peer-id>"]
```

---

## Performance Tuning

### Metrics Collection

For high-volume testing, adjust Prometheus scrape interval:

```yaml
# monitoring/prometheus.yml
global:
  scrape_interval: 5s   # Default 15s, increase for high volume
  evaluation_interval: 5s
```

### Database Connection Pooling

Edit testnet-config.toml:

```toml
[database]
max_connections = 50     # Increase for high concurrency
min_connections = 5
connection_timeout_secs = 10
```

### Network Tuning

```toml
[network]
max_connections = 500    # Increase peer connections
inbound_timeout_secs = 30
outbound_timeout_secs = 10
```

---

## Next Steps

After successful deployment:

1. **Load Testing**: Run user node clients against testnet
2. **Monitoring**: Set up Grafana alerts for relay health
3. **Backup**: Configure automated database backups
4. **Scaling**: Deploy to Kubernetes for production
5. **Documentation**: Update OPERATIONAL_GUIDE.md with actual metrics

---

## Support & Debugging

### Enable Debug Logging

```powershell
# Set environment variable for relay1
docker-compose down
$env:RUST_LOG = "debug"
docker-compose up -d
docker logs -f dchat-relay1
```

### Collect System Information

```powershell
# For bug reports
docker version
docker-compose version
docker ps
docker-compose ps
docker logs dchat-relay1 > relay1.log
docker logs dchat-postgres > postgres.log
Get-SystemInfo
Get-ComputerInfo | select-object CsSystemType
```

### Network Diagnostics

```powershell
# Test network connectivity
docker exec dchat-relay1 curl http://relay2:9091
docker exec dchat-relay2 curl http://relay1:9090

# Check DNS resolution
docker exec dchat-relay1 nslookup relay1
docker exec dchat-relay1 nslookup postgres
```

---

**Last Updated**: 2024
**Status**: Production Ready ✅
