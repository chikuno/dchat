# dchat Production Deployment Guide
## Deploying to rpc.webnetcore.top:8080

This guide provides complete step-by-step instructions for deploying the dchat testnet to your production server.

---

## 📋 Table of Contents

1. [Quick Start](#quick-start)
2. [Pre-Deployment Checklist](#pre-deployment-checklist)
3. [Step-by-Step Deployment](#step-by-step-deployment)
4. [Verification & Testing](#verification--testing)
5. [Monitoring & Observability](#monitoring--observability)
6. [Troubleshooting](#troubleshooting)
7. [Operations & Maintenance](#operations--maintenance)

---

## 🚀 Quick Start

### For Experienced Operators (5-minute setup)

**IMPORTANT: Run build initialization first** to ensure all dependencies are correct:

```bash
# SSH into remote server
ssh user@rpc.webnetcore.top

# Clone repository
git clone https://github.com/chikuno/dchat.git /opt/dchat
cd /opt/dchat


# install nightly 

rustup install nightly

rustup override set nightly

cargo --version
rustc --version


# Initialize build environment (REQUIRED - fixes Rust version issues)
chmod +x scripts/build-init.sh
./scripts/build-init.sh

# Create keys directory
mkdir -p validator_keys





#generate bin 

cargo run --bin dchat


# Generate keys
cargo run --release --bin key-generator -- -o validator_keys/validator1.key
cargo run --release --bin key-generator -- -o validator_keys/validator2.key
cargo run --release --bin key-generator -- -o validator_keys/validator3.key
cargo run --release --bin key-generator -- -o validator_keys/validator4.key

# Build and deploy
DOCKER_BUILDKIT=1 docker build -t dchat:latest 

            or 

docker build -t dchat:latest .  ## ( copy with the dot)

docker-compose -f docker-compose-production.yml up -d

# Verify
docker ps
curl http://localhost:7071/health
```

### For First-Time Deployers

Follow the [Step-by-Step Deployment](#step-by-step-deployment) section below.

---

## 📋 Pre-Deployment Checklist

### Server Requirements

- **OS**: Ubuntu 20.04 LTS or newer
- **CPU**: 8+ cores (4 vCPU minimum, 8 vCPU recommended)
- **RAM**: 16GB+ (8GB minimum for 4 validators)
- **Disk**: 100GB+ SSD (500GB recommended for growth)
- **Network**: 1Gbps+ connection with static IP
- **Ports Open**: 7070-7087 (internal), 8080 (external RPC), 9090, 3000, 16686 (monitoring)

### Access & Credentials

```bash
# Verify SSH access
ssh -i ~/.ssh/your_key user@rpc.webnetcore.top

# Check domain resolution
nslookup rpc.webnetcore.top
# Should resolve to your server IP

# Verify ports open
telnet rpc.webnetcore.top 8080
# Should connect (Ctrl+] then quit to exit)
```

### System Preparation

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install Docker (if not already installed)
sudo apt install -y docker.io docker-compose

# Add current user to docker group
sudo usermod -aG docker $USER
newgrp docker

# Verify Docker installation
docker --version
docker-compose --version

# Install additional tools
sudo apt install -y git curl jq nginx

# Configure firewall (UFW)
sudo ufw allow 22/tcp        # SSH
sudo ufw allow 8080/tcp      # RPC (public)
sudo ufw allow 9090/tcp      # Prometheus
sudo ufw allow 3000/tcp      # Grafana
sudo ufw allow 16686/tcp     # Jaeger
sudo ufw enable

# Create deployment directory
sudo mkdir -p /opt/dchat
sudo chown $USER:$USER /opt/dchat
```

---

## 📦 Step-by-Step Deployment

### Step 1: Clone Repository

```bash
cd /opt/dchat
git clone https://github.com/chikuno/dchat.git .
git pull origin main

# Verify you're on the right branch
git branch -a
git log --oneline -5
```

### Step 1.5: Initialize Build Environment (REQUIRED)

**This step is CRITICAL** - it fixes the `edition2024` Rust compatibility error:

```bash
# Run the build initialization script
cd /opt/dchat

# On Linux/macOS
chmod +x scripts/build-init.sh
./scripts/build-init.sh

# On Windows (PowerShell)
powershell -ExecutionPolicy Bypass -File scripts/build-init.ps1
```

This script will:
- ✅ Update Rust to version 1.82 (required for the project)
- ✅ Install system dependencies (OpenSSL, SQLite, etc.)
- ✅ Clear cargo cache to prevent stale versions
- ✅ Update all dependencies to compatible versions
- ✅ Test the build configuration

**Expected output:**
```
✅ Rust toolchain set to 1.82
✅ Library build successful
Ready to build dchat! 🚀
```

### Step 2: Generate Validator Keys

**IMPORTANT**: Keep these keys secure. They control your validators.

```bash
# Create keys directory
mkdir -p /opt/dchat/validator_keys
cd /opt/dchat

# Generate 4 validator keys
for i in {1..4}; do
    cargo run --release --bin key-generator -- -o validator_keys/validator$i.key -t ed25519
    chmod 400 validator_keys/validator$i.key
done

# Verify keys generated
ls -la validator_keys/

# Create encrypted backup
gpg --encrypt validator_keys/*.key
# or
tar czf validator_keys.tar.gz validator_keys/
scp validator_keys.tar.gz backup_server:/backups/
```

### Step 3: Prepare Monitoring Configuration

```bash
# Create monitoring directories
mkdir -p monitoring/{prometheus,grafana/{datasources,dashboards}}

# Copy Prometheus config (see below)
cat > monitoring/prometheus.yml << 'EOF'
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'validators'
    static_configs:
      - targets: ['validator1:9090', 'validator2:9090', 'validator3:9090', 'validator4:9090']
  
  - job_name: 'relays'
    static_configs:
      - targets: ['relay1:9100', 'relay2:9100', 'relay3:9100', 'relay4:9100']
  
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']
EOF

# Copy Grafana datasource
mkdir -p monitoring/grafana/datasources
cat > monitoring/grafana/datasources/prometheus.yml << 'EOF'
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: true
EOF
```

### Step 4: Build Docker Image

```bash
cd /opt/dchat

# Build the image (this may take 5-10 minutes)
docker build -t dchat:latest -f Dockerfile .

# Tag with version
docker tag dchat:latest dchat:v1.0

# Verify build
docker images | grep dchat
# Output: dchat  latest  <IMAGE_ID>  <TIME>  <SIZE>
```

### Step 5: Start Services with Docker Compose

```bash
cd /opt/dchat

# Start all services
docker-compose -f docker-compose-production.yml up -d

# Watch startup progress
docker-compose -f docker-compose-production.yml logs -f --tail=50

# Check service status
docker-compose -f docker-compose-production.yml ps

# Expected output:
# NAME                STATUS          PORTS
# dchat-validator1    Up (healthy)    0.0.0.0:7070->7070/tcp, ...
# dchat-validator2    Up (healthy)    0.0.0.0:7072->7070/tcp, ...
# dchat-validator3    Up (healthy)    0.0.0.0:7074->7070/tcp, ...
# dchat-validator4    Up (healthy)    0.0.0.0:7076->7070/tcp, ...
# dchat-relay1        Up (healthy)    0.0.0.0:7080->7080/tcp, ...
# dchat-relay2        Up (healthy)    0.0.0.0:7082->7080/tcp, ...
# dchat-relay3        Up (healthy)    0.0.0.0:7084->7080/tcp, ...
# dchat-relay4        Up (healthy)    0.0.0.0:7086->7080/tcp, ...
# dchat-prometheus    Up              0.0.0.0:9090->9090/tcp
# dchat-grafana       Up              0.0.0.0:3000->3000/tcp
# dchat-jaeger        Up              0.0.0.0:16686->16686/tcp
```

### Step 6: Configure Nginx Reverse Proxy

```bash
# Create Nginx configuration
sudo tee /etc/nginx/sites-available/dchat << 'EOF'
upstream validators {
    least_conn;
    server localhost:7071 weight=1 max_fails=2 fail_timeout=30s;
    server localhost:7073 weight=1 max_fails=2 fail_timeout=30s;
    server localhost:7075 weight=1 max_fails=2 fail_timeout=30s;
    server localhost:7077 weight=1 max_fails=2 fail_timeout=30s;
}

server {
    listen 8080;
    server_name rpc.webnetcore.top;
    
    location / {
        proxy_pass http://validators;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header Host $host;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
    }
}
EOF

# Enable the site
sudo ln -s /etc/nginx/sites-available/dchat /etc/nginx/sites-enabled/ 2>/dev/null || true

# Test configuration
sudo nginx -t

# Reload Nginx
sudo systemctl reload nginx

# Verify
curl http://localhost:8080/health
```

### Step 7: Setup SSL/TLS (Optional but Recommended)

```bash
# Install Certbot
sudo apt install -y certbot python3-certbot-nginx

# Obtain certificate
sudo certbot certonly --standalone -d rpc.webnetcore.top

# Update Nginx to use SSL
sudo certbot --nginx -d rpc.webnetcore.top

# Verify SSL
curl -I https://rpc.webnetcore.top:8080/health
```

---

## ✅ Verification & Testing

### Health Checks

```bash
# Check all validators
for i in 1 2 3 4; do
    port=$((7070 + (i-1)*2))
    echo "Validator $i (port $port):"
    curl -s http://localhost:$port/health | jq .
done

# Check all relays
for i in 1 2 3 4; do
    port=$((7080 + (i-1)*2))
    echo "Relay $i (port $port):"
    curl -s http://localhost:$port/health | jq .
done

# Check RPC endpoint (through proxy)
curl -s http://localhost:8080/health | jq .
```

### Consensus Verification

```bash
# Check consensus status
curl -s http://localhost:7071/chain/consensus-status | jq .

# Expected output (4/4 validators voting):
# {
#   "voting_count": 4,
#   "total_validators": 4,
#   "finalized": true,
#   "block_number": 123
# }

# Monitor block production (should see new block every 3 seconds)
watch -n 1 'curl -s http://localhost:7071/chain/latest-block | jq .block_number'
```

### Message Propagation Test

```bash
# Send test message through relay
curl -X POST http://localhost:7081/message \
  -H "Content-Type: application/json" \
  -d '{
    "from": "test_user",
    "to": "recipient",
    "content": "Hello from dchat!"
  }'

# Verify message appears on chat chain
curl -s http://localhost:7071/chain/messages | jq '.[-1]'
```

### External Access Test

```bash
# From your local machine, test external access
curl -I http://rpc.webnetcore.top:8080/health

# Should return: HTTP/1.1 200 OK
```

### Performance Baseline

```bash
# Measure RPC response time
time curl -s http://localhost:8080/health > /dev/null
# Target: < 100ms

# Check message throughput
for i in {1..100}; do
    curl -X POST http://localhost:7081/message \
      -H "Content-Type: application/json" \
      -d "{\"content\": \"test-$i\"}" 2>/dev/null
done

# Monitor in Grafana: Should see throughput > 1000 msg/sec
```

---

## 📊 Monitoring & Observability

### Prometheus Monitoring

Access: **http://localhost:9090**

```bash
# View targets
curl -s http://localhost:9090/api/v1/targets | jq '.data.activeTargets[] | {labels: .labels, health: .health}'

# Query metrics
curl -s 'http://localhost:9090/api/v1/query?query=dchat_block_height' | jq '.data.result'

# Check available metrics
curl -s http://localhost:9090/metrics | grep dchat_
```

**Key Metrics to Monitor**:
- `dchat_block_height`: Latest block number
- `dchat_consensus_voters`: Number of active validators
- `dchat_message_latency`: Message propagation time
- `dchat_validator_uptime`: Validator availability
- `dchat_relay_throughput`: Messages relayed per second

### Grafana Dashboards

Access: **http://localhost:3000** (default: admin/admin)

**Change password immediately**:
1. Login with admin/admin
2. Profile → Change Password
3. Set new password

**Import Dashboards**:
1. Click "+" → Import
2. Import dashboard ID for dchat (if available) or create custom
3. Common dashboard IDs:
   - 12114: Node Exporter
   - 1860: Node Exporter Full
   - 3662: Prometheus

**Recommended Custom Dashboards**:
- Validator Consensus Status (voting power, finalization)
- Relay Performance (throughput, latency)
- Network Topology (peer connections)
- System Resources (CPU, memory, disk usage)

### Jaeger Distributed Tracing

Access: **http://localhost:16686**

```bash
# View available services
curl -s http://localhost:16686/api/services

# View traces for specific service
curl -s 'http://localhost:16686/api/traces?service=dchat-validator1'
```

---

## 🔧 Troubleshooting

### Service Won't Start

```bash
# Check logs
docker-compose -f docker-compose-production.yml logs validator1

# Common issues:
# "Address already in use" → Check what's using the port
lsof -i :7070
# Kill the process
kill -9 <PID>

# Restart services
docker-compose -f docker-compose-production.yml restart validator1
```

### Health Check Failing

```bash
# Debug health endpoint
docker exec dchat-validator1 curl -v http://localhost:7071/health

# Check validator logs for errors
docker logs dchat-validator1 --tail=50 | grep ERROR

# Verify network connectivity
docker exec dchat-validator1 ping dchat-validator2
docker exec dchat-validator1 ping dchat-relay1
```

### Consensus Not Finalizing

```bash
# Check consensus status
curl -s http://localhost:7071/chain/consensus-status | jq '.'

# Ensure all 4 validators are voting
# If < 4: Restart failed validators
docker-compose -f docker-compose-production.yml restart validator1 validator2 validator3 validator4

# Check if validators are peers
curl -s http://localhost:7071/peers | jq '.peers | length'
# Should be 3-4 (all other validators)
```

### High Resource Usage

```bash
# Check container stats
docker stats --no-stream

# If > 80% CPU:
# 1. Check for long-running queries
curl -s http://localhost:7071/metrics | grep duration_seconds

# 2. Check for network flooding
docker logs dchat-relay1 | grep "message queue"

# 3. Restart container to clear state
docker-compose -f docker-compose-production.yml restart relay1
```

### Disk Space Issues

```bash
# Check disk usage
df -h

# List container volumes
docker volume ls

# Find largest volumes
du -sh /var/lib/docker/volumes/*/

# Clean up old Docker data
docker system prune -a
# WARNING: This deletes stopped containers and unused images

# Archive old data
docker exec dchat-validator1 tar czf /var/lib/dchat/archive.tar.gz /var/lib/dchat/data/
docker cp dchat-validator1:/var/lib/dchat/archive.tar.gz ./archive/
```

### Network Connectivity Issues

```bash
# Test inter-container connectivity
docker exec dchat-validator1 ping dchat-validator2
docker exec dchat-validator1 ping dchat-relay1

# Check network status
docker network inspect dchat-testnet

# Restart networking
docker-compose -f docker-compose-production.yml down
docker-compose -f docker-compose-production.yml up -d
```

---

## 🛠️ Operations & Maintenance

### Daily Operations

```bash
# Morning checks
curl -s http://localhost:7071/health | jq '.status'
docker-compose -f docker-compose-production.yml ps
curl -s http://localhost:9090/api/v1/targets | jq '.data.activeTargets[] | .health' | sort | uniq -c

# Monitor block production
watch -n 5 'curl -s http://localhost:7071/chain/latest-block | jq .block_number'

# Check for errors
docker-compose -f docker-compose-production.yml logs --since 1h | grep ERROR
```

### Weekly Maintenance

```bash
# Backup data
tar czf /backup/dchat-$(date +%Y%m%d).tar.gz /opt/dchat/validator_keys /opt/dchat/monitoring

# Clean logs
docker system prune -f

# Update monitoring
docker pull prom/prometheus:latest
docker pull grafana/grafana:latest
docker-compose -f docker-compose-production.yml up -d
```

### Restart Procedures

```bash
# Graceful restart (preserves state)
docker-compose -f docker-compose-production.yml restart validator1

# Hard restart (clears memory)
docker-compose -f docker-compose-production.yml down validator1
docker-compose -f docker-compose-production.yml up -d validator1

# Full restart (all services)
docker-compose -f docker-compose-production.yml down
docker-compose -f docker-compose-production.yml up -d
```

### Backup & Recovery

```bash
# Create backup
tar czf /backup/dchat-backup-$(date +%Y%m%d-%H%M%S).tar.gz \
  /opt/dchat/validator_keys \
  /opt/dchat/monitoring \
  /var/lib/docker/volumes/validator*_data

# Restore from backup
tar xzf /backup/dchat-backup-20240101-120000.tar.gz -C /

# Verify restoration
docker ps -a
curl -s http://localhost:7071/health | jq '.'
```

---

## 📞 Support & Resources

### Documentation
- **ARCHITECTURE.md**: System design and components
- **docker-compose-production.yml**: Service configuration
- **PRODUCTION_DEPLOYMENT_CHECKLIST.md**: Detailed checklist

### Monitoring Commands

```bash
# Real-time dashboard
./health-dashboard.ps1 -Continuous

# Export metrics to JSON
./health-dashboard.ps1 -ExportFormat json

# View logs with filtering
docker-compose -f docker-compose-production.yml logs --filter "source=stdout" -f
```

### Emergency Contacts
- On-call: [Your contact info]
- Escalation: [Your manager]
- Status Page: https://status.dchat.io

---

## 🎯 Success Criteria

Your deployment is successful when:

✅ All 4 validators are healthy and voting
✅ All 4 relays are operational
✅ RPC endpoint accessible at `http://rpc.webnetcore.top:8080`
✅ Prometheus scraping all targets
✅ Grafana dashboards displaying metrics
✅ New blocks produced every 3 seconds
✅ Message latency < 1 second
✅ No errors in logs
✅ Monitoring endpoints accessible
✅ Backup system operational

---

**Deployment Date**: ____________
**Deployed By**: ________________
**Last Updated**: _______________

For questions or issues, refer to the PRODUCTION_DEPLOYMENT_CHECKLIST.md or contact your operations team.
