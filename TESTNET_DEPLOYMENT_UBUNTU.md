# dchat Testnet Deployment Guide for Ubuntu Server

Complete guide for deploying dchat testnet on a clean Ubuntu server.

## 📋 Prerequisites

### Server Requirements
- **OS**: Ubuntu 20.04 LTS, 22.04 LTS, or 24.04 LTS
- **RAM**: 4GB minimum, 8GB+ recommended
- **Disk**: 50GB minimum, 100GB+ recommended
- **CPU**: 2+ cores
- **Network**: Public IP address with open ports
- **Access**: Root or sudo privileges

### What You Need Locally
1. SSH access to your Ubuntu server
2. This repository cloned or transferred to the server
3. Validator keys generated (if not done yet)

## 🚀 Quick Start (5 Steps)

### Step 1: Connect to Your Server
```bash
ssh root@your-server-ip
# or
ssh your-user@your-server-ip
```

### Step 2: Upload the Repository
```bash
# Option A: Clone from Git
git clone https://github.com/your-org/dchat.git
cd dchat

# Option B: Upload via SCP (from your local machine)
scp -r /path/to/dchat root@your-server-ip:/root/
```

### Step 3: Generate Validator Keys (if not done)
```bash
# On your local Windows machine first:
powershell -ExecutionPolicy Bypass -File generate-validator-keys.ps1

# Then upload to server:
scp -r validator_keys root@your-server-ip:/root/dchat/
```

### Step 4: Make Script Executable
```bash
cd dchat
chmod +x deploy-ubuntu-testnet.sh
```

### Step 5: Run Deployment
```bash
# Full deployment (recommended for first time)
sudo ./deploy-ubuntu-testnet.sh

# Or with options:
sudo ./deploy-ubuntu-testnet.sh --skip-docker    # If Docker already installed
sudo ./deploy-ubuntu-testnet.sh --skip-build     # To save time (use existing images)
```

**The script will take 15-45 minutes depending on your server speed and network.**

## 📊 What Gets Deployed

### Network Topology
```
┌─────────────────────────────────────────────────────┐
│                 dchat Testnet                        │
├─────────────────────────────────────────────────────┤
│                                                      │
│  Validators (4):                                    │
│    ├─ validator1 (7070, 7071, 9090)                │
│    ├─ validator2 (7072, 7073, 9091)                │
│    ├─ validator3 (7074, 7075, 9092)                │
│    └─ validator4 (7076, 7077, 9093)                │
│                                                      │
│  Relays (7):                                        │
│    ├─ relay1 (7080, 7081, 9100)                    │
│    ├─ relay2 (7082, 7083, 9101)                    │
│    ├─ relay3 (7084, 7085, 9102)                    │
│    ├─ relay4 (7086, 7087, 9103)                    │
│    ├─ relay5 (7088, 7089, 9104)                    │
│    ├─ relay6 (7090, 7091, 9105)                    │
│    └─ relay7 (7092, 7093, 9106)                    │
│                                                      │
│  Users (3):                                         │
│    ├─ user1 (7110, 7111, 9110)                     │
│    ├─ user2 (7112, 7113, 9111)                     │
│    └─ user3 (7114, 7115, 9112)                     │
│                                                      │
│  Monitoring:                                        │
│    ├─ Prometheus (9090)                             │
│    ├─ Grafana (3000)                                │
│    └─ Jaeger (16686, 4317, 4318)                   │
│                                                      │
└─────────────────────────────────────────────────────┘
```

### Ports Opened on Firewall
- **22**: SSH (essential - don't block!)
- **7070-7077**: Validator nodes (P2P)
- **7080-7093**: Relay nodes (P2P)
- **7110-7115**: User nodes (P2P)
- **3000**: Grafana dashboard
- **9090**: Prometheus
- **16686**: Jaeger tracing UI

## 🔍 Post-Deployment Verification

### 1. Check Container Status
```bash
./status-testnet.sh
# Or manually:
docker ps
```

All containers should show as "Up" and "healthy" status.

### 2. Check Logs
```bash
# All containers
./logs-testnet.sh

# Specific container
./logs-testnet.sh validator1

# Last 100 lines
docker logs --tail 100 dchat-validator1
```

### 3. Test Network Endpoints

```bash
# Check validator health
curl http://localhost:7071/health
curl http://localhost:7073/health

# Check Prometheus
curl http://localhost:9090/-/healthy

# Check Grafana
curl http://localhost:3000/api/health
```

### 4. Access Web Interfaces

Open in your browser:
- **Grafana**: `http://your-server-ip:3000`
  - Username: `admin`
  - Password: `admin` (change on first login)

- **Prometheus**: `http://your-server-ip:9090`

- **Jaeger**: `http://your-server-ip:16686`

## 🎮 Management Commands

### Start/Stop/Restart
```bash
# Stop testnet
./stop-testnet.sh

# Start testnet
./start-testnet.sh

# Restart specific service
docker compose -f docker-compose-testnet.yml -p dchat-testnet restart validator1

# Restart all
docker compose -f docker-compose-testnet.yml -p dchat-testnet restart
```

### View Logs
```bash
# All services
./logs-testnet.sh

# Specific service
./logs-testnet.sh relay1

# Follow logs in real-time
docker compose -f docker-compose-testnet.yml -p dchat-testnet logs -f validator1
```

### Check Resource Usage
```bash
# Container stats
docker stats

# Disk usage
docker system df

# Network info
docker network inspect dchat-testnet
```

### Clean Up
```bash
# Stop and remove containers (data preserved)
docker compose -f docker-compose-testnet.yml -p dchat-testnet down

# Stop and remove containers + volumes (DELETES ALL DATA)
docker compose -f docker-compose-testnet.yml -p dchat-testnet down -v

# Remove unused images
docker image prune -a
```

## 🧪 Testing the Network

### Test Message Propagation
```bash
# Connect to user1
docker exec -it dchat-user1 bash

# Inside container, send a test message
dchat send --to user2@dchat.local --message "Hello from user1!"

# Exit and connect to user2
exit
docker exec -it dchat-user2 bash

# Check received messages
dchat inbox
```

### Test Validator Consensus
```bash
# Check validator sync status
curl http://localhost:7071/status | jq

# Check block height across validators
for port in 7071 7073 7075 7077; do
  echo "Validator on port $port:"
  curl -s http://localhost:$port/status | jq '.block_height'
done
```

### Load Test Relay Network
```bash
# Use the benchmarking tools
docker exec -it dchat-relay1 bash
dchat benchmark --duration 60 --messages 1000
```

## 🔧 Troubleshooting

### Container Won't Start
```bash
# Check logs for errors
docker logs dchat-validator1

# Check if port is already in use
sudo netstat -tulpn | grep 7070

# Recreate container
docker compose -f docker-compose-testnet.yml -p dchat-testnet up -d --force-recreate validator1
```

### Out of Disk Space
```bash
# Check disk usage
df -h

# Clean Docker cache
docker system prune -a --volumes

# Remove old logs
rm -rf testnet-logs/*.log
```

### Containers Keep Restarting
```bash
# Check recent container logs
docker logs --tail 200 dchat-validator1

# Common issues:
# - Missing validator keys
# - Insufficient RAM
# - Disk full
# - Port conflicts

# Check system resources
free -h
df -h
docker stats --no-stream
```

### Network Connectivity Issues
```bash
# Check firewall
sudo ufw status

# Check Docker network
docker network inspect dchat-testnet

# Test container connectivity
docker exec -it dchat-user1 ping validator1

# Check DNS resolution
docker exec -it dchat-user1 nslookup validator1
```

### Can't Access Web UIs
```bash
# Check if ports are listening
sudo netstat -tulpn | grep -E ':(3000|9090|16686)'

# Check firewall
sudo ufw status | grep -E '(3000|9090|16686)'

# Check container status
docker ps | grep -E '(grafana|prometheus|jaeger)'

# Test locally first
curl http://localhost:3000
curl http://localhost:9090
curl http://localhost:16686
```

## 🔐 Security Hardening (Production)

### 1. Change Default Passwords
```bash
# Grafana: Change admin password via UI or:
docker exec -it dchat-grafana grafana-cli admin reset-admin-password newpassword
```

### 2. Restrict Firewall Access
```bash
# Only allow specific IPs to access monitoring
sudo ufw delete allow 3000
sudo ufw allow from YOUR_IP to any port 3000

# Or use SSH tunneling instead
ssh -L 3000:localhost:3000 user@server-ip
# Then access via http://localhost:3000 on your local machine
```

### 3. Enable TLS/SSL
```bash
# Use nginx or Caddy as reverse proxy
sudo apt install nginx
# Configure SSL certificates (Let's Encrypt recommended)
```

### 4. Secure Validator Keys
```bash
# Ensure proper permissions
chmod 600 validator_keys/*.key
chown root:root validator_keys/*.key

# Consider using hardware security modules (HSM) for production
```

### 5. Regular Updates
```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Update Docker images
docker compose -f docker-compose-testnet.yml pull
docker compose -f docker-compose-testnet.yml up -d
```

## 📈 Monitoring and Metrics

### Grafana Dashboards
After logging into Grafana (http://your-ip:3000):

1. **Import dchat Dashboard**:
   - Go to Dashboards → Import
   - Upload `monitoring/grafana/dashboards/dchat-overview.json`

2. **Key Metrics to Watch**:
   - Message throughput (messages/second)
   - P2P connection count
   - Block production rate
   - Validator consensus status
   - Relay node uptime
   - Memory/CPU usage per container

### Prometheus Queries
Useful queries to run in Prometheus (http://your-ip:9090):

```promql
# Message rate across all nodes
rate(dchat_messages_total[5m])

# Validator block height
dchat_block_height{role="validator"}

# Relay uptime percentage
avg_over_time(up{job="dchat-relays"}[1h]) * 100

# Network latency (p95)
histogram_quantile(0.95, rate(dchat_network_latency_seconds_bucket[5m]))
```

### Alert Setup
Create alerts for:
- Container down/unhealthy
- Disk space < 10%
- Memory usage > 90%
- Validator out of sync
- Relay node offline > 5 minutes

## 🔄 Upgrading

### Update dchat Version
```bash
# Pull latest code
git pull origin main

# Rebuild images
docker compose -f docker-compose-testnet.yml build

# Restart with new images
docker compose -f docker-compose-testnet.yml up -d
```

### Rolling Update (Zero Downtime)
```bash
# Update one validator at a time
for i in 1 2 3 4; do
  docker compose -f docker-compose-testnet.yml up -d --no-deps --build validator$i
  sleep 30  # Wait for sync
done
```

## 📚 Additional Resources

- **Architecture**: See `ARCHITECTURE.md`
- **API Documentation**: See `API_SPECIFICATION.md`
- **Governance**: See `GOVERNANCE_QUICK_REF.md`
- **Marketplace**: See `MARKETPLACE_QUICK_REF.md`
- **Docker Setup**: See `DOCKER_QUICK_REF.md`

## 💬 Support

- **Issues**: Report bugs on GitHub Issues
- **Logs**: Always check `/var/log/dchat-deployment.log`
- **Community**: Join our Discord/Telegram

## 📝 Deployment Checklist

- [ ] Ubuntu server provisioned (4GB+ RAM, 50GB+ disk)
- [ ] SSH access configured
- [ ] Validator keys generated and uploaded
- [ ] Firewall ports opened (or will be auto-configured)
- [ ] Deployment script executed successfully
- [ ] All containers running and healthy
- [ ] Web interfaces accessible
- [ ] Test message sent between users
- [ ] Monitoring dashboards configured
- [ ] Passwords changed from defaults
- [ ] Backup strategy established
- [ ] Documentation reviewed

---

**Deployment Time**: ~15-45 minutes  
**Last Updated**: October 31, 2025  
**Version**: dchat v0.1.0 Testnet
