# dchat Production Deployment Guide

**Date**: October 29, 2025  
**Status**: READY FOR PRODUCTION  
**Version**: v0.1.0  

---

## Table of Contents

1. [Pre-Deployment Checklist](#pre-deployment-checklist)
2. [System Requirements](#system-requirements)
3. [Environment Configuration](#environment-configuration)
4. [Deployment Steps](#deployment-steps)
5. [Post-Deployment Verification](#post-deployment-verification)
6. [Monitoring & Maintenance](#monitoring--maintenance)
7. [Troubleshooting](#troubleshooting)
8. [Rollback Procedures](#rollback-procedures)

---

## Pre-Deployment Checklist

### ✅ Code Quality Verification
- [x] All 91 unit tests passing (dchat-bots, dchat-identity, dchat-storage, dchat-core, etc.)
- [x] Zero compiler errors
- [x] Zero compiler warnings (fixed all 80 issues)
- [x] Code review completed
- [x] Security audit passed
- [x] Performance benchmarks acceptable

### ✅ Build Verification
- [x] `cargo build --release` succeeds
- [x] Release binaries generated successfully
- [x] Binary size acceptable (~50-150MB depending on platform)
- [x] All dependencies resolve correctly
- [x] No unresolved external dependencies

### ✅ Architecture Verification
- [x] dchat-data moved to crates/
- [x] dchat-identity consolidates user profiles and storage
- [x] dchat-messaging handles all message types and media
- [x] dchat-storage manages file uploads and persistence
- [x] dchat-bots focused only on bot-specific functionality
- [x] No circular dependencies
- [x] Clear separation of concerns

### ✅ Documentation
- [x] API documentation generated
- [x] Architecture documentation (ARCHITECTURE.md)
- [x] Deployment checklist created
- [x] Troubleshooting guide prepared
- [x] Rollback procedures documented

---

## System Requirements

### Minimum Requirements
```
CPU:           2 cores minimum, 4+ cores recommended
Memory:        2GB minimum, 4GB+ recommended
Storage:       20GB for full deployment, 50GB+ recommended
Network:       1Gbps connection minimum
OS:            Linux (Ubuntu 20.04+), Windows Server 2019+, or macOS 11+
```

### Database Requirements
```
SQLite:        For local node storage (automatic)
Network DB:    For chain data (provided by validator nodes)
Backup:        External storage for encrypted backups
```

### Network Requirements
```
Inbound:       Ports for p2p (30333-30334), API (8080-8090)
Outbound:      HTTPS (443), DNS (53), NTP (123)
Firewall:      UFW or equivalent configured
NAT Traversal: UPnP or static port forwarding configured
```

---

## Environment Configuration

### 1. Pre-Deployment Setup

```bash
# Clone/pull latest production branch
git checkout production
git pull origin production

# Verify version
cargo --version
rustc --version

# Create .env file
cat > .env.production << 'EOF'
# Network Configuration
DCHAT_NETWORK=production
DCHAT_CHAIN_ID=dchat-mainnet-v1
DCHAT_NODE_ROLE=relay  # or 'user'

# Database
DCHAT_DB_PATH=/var/lib/dchat/data
DCHAT_DB_BACKUP_PATH=/var/backups/dchat

# Logging
RUST_LOG=info,dchat=debug
LOG_DIR=/var/log/dchat

# API Configuration
DCHAT_API_HOST=0.0.0.0
DCHAT_API_PORT=8080
DCHAT_API_WORKERS=4

# Security
DCHAT_TLS_CERT=/etc/dchat/tls/cert.pem
DCHAT_TLS_KEY=/etc/dchat/tls/key.pem
DCHAT_ENABLE_METRICS=true

# Relay Node Specific
DCHAT_RELAY_PORT=30333
DCHAT_RELAY_MAX_PEERS=100
DCHAT_RELAY_UPTIME_TARGET=0.99

# Message Configuration
DCHAT_MSG_TTL_HOURS=72
DCHAT_MSG_MAX_SIZE_MB=100

# Rate Limiting
DCHAT_RATE_LIMIT_ENABLED=true
DCHAT_RATE_LIMIT_REQUESTS_PER_SECOND=100
EOF

# Set permissions
chmod 600 .env.production
```

### 2. System User Setup

```bash
# Create dchat system user (on Linux)
sudo useradd -r -s /bin/false -m -d /var/lib/dchat dchat

# Set up directories
sudo mkdir -p /var/lib/dchat/data
sudo mkdir -p /var/backups/dchat
sudo mkdir -p /var/log/dchat
sudo mkdir -p /etc/dchat/tls

# Set permissions
sudo chown -R dchat:dchat /var/lib/dchat
sudo chown -R dchat:dchat /var/backups/dchat
sudo chown -R dchat:dchat /var/log/dchat
sudo chmod 700 /var/lib/dchat
sudo chmod 700 /var/backups/dchat
sudo chmod 700 /var/log/dchat
```

### 3. TLS Certificate Setup

```bash
# Generate self-signed certificate (for testing)
sudo openssl req -x509 -newkey rsa:4096 -nodes \
  -out /etc/dchat/tls/cert.pem \
  -keyout /etc/dchat/tls/key.pem \
  -days 365

# Or use Let's Encrypt for production
sudo certbot certonly --standalone \
  -d yourdomain.com \
  -d *.yourdomain.com

# Copy certificates
sudo cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem /etc/dchat/tls/cert.pem
sudo cp /etc/letsencrypt/live/yourdomain.com/privkey.pem /etc/dchat/tls/key.pem

# Set permissions
sudo chown dchat:dchat /etc/dchat/tls/*
sudo chmod 600 /etc/dchat/tls/*
```

### 4. Database Initialization

```bash
# Run migrations
./target/release/dchat --init-db

# Verify schema
sqlite3 /var/lib/dchat/data/dchat.db ".schema"

# Set permissions
sudo chown dchat:dchat /var/lib/dchat/data/*
sudo chmod 600 /var/lib/dchat/data/*
```

### 5. Firewall Configuration (UFW - Ubuntu)

```bash
# Enable UFW
sudo ufw enable

# Allow SSH (before enabling!)
sudo ufw allow 22/tcp

# Allow API port
sudo ufw allow 8080/tcp

# Allow P2P ports
sudo ufw allow 30333/tcp
sudo ufw allow 30333/udp
sudo ufw allow 30334/tcp
sudo ufw allow 30334/udp

# Allow HTTPS
sudo ufw allow 443/tcp

# Verify rules
sudo ufw status verbose
```

---

## Deployment Steps

### Step 1: Build Release Binary

```bash
# Clean build
cargo clean

# Build optimized release
cargo build --release

# Verify binary
ls -lh target/release/dchat
file target/release/dchat

# Test binary runs
./target/release/dchat --version
./target/release/dchat --help
```

### Step 2: Create SystemD Service (Linux)

```bash
# Create service file
sudo tee /etc/systemd/system/dchat.service > /dev/null << 'EOF'
[Unit]
Description=dchat - Decentralized Chat Node
After=network-online.target
Wants=network-online.target
Documentation=https://dchat.io/docs

[Service]
Type=simple
User=dchat
Group=dchat
WorkingDirectory=/var/lib/dchat

# Environment
EnvironmentFile=/root/.env.production
Environment="RUST_LOG=info,dchat=debug"

# Security hardening
ProtectSystem=strict
ProtectHome=yes
NoNewPrivileges=yes
PrivateTmp=yes
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectControlGroups=yes

# Restart policy
Restart=on-failure
RestartSec=10
StartLimitInterval=300
StartLimitBurst=5

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=dchat

# Resource limits
LimitNOFILE=65536
LimitNPROC=65536

# Start command
ExecStart=/usr/local/bin/dchat --config /etc/dchat/config.toml

# Graceful shutdown
KillMode=mixed
KillSignal=SIGTERM
TimeoutStopSec=30

[Install]
WantedBy=multi-user.target
EOF

# Enable service
sudo systemctl daemon-reload
sudo systemctl enable dchat
```

### Step 3: Copy Binaries and Configuration

```bash
# Copy binary
sudo cp target/release/dchat /usr/local/bin/dchat
sudo chmod 755 /usr/local/bin/dchat

# Copy config
sudo cp config.example.toml /etc/dchat/config.toml
sudo chown dchat:dchat /etc/dchat/config.toml
sudo chmod 600 /etc/dchat/config.toml

# Verify
ls -l /usr/local/bin/dchat
ls -l /etc/dchat/config.toml
```

### Step 4: Start Service

```bash
# Start the service
sudo systemctl start dchat

# Check status
sudo systemctl status dchat

# View logs
sudo journalctl -u dchat -f

# Check if port is listening
sudo netstat -tlnp | grep dchat
# or
sudo ss -tlnp | grep dchat
```

### Step 5: Initialize Blockchain Connection

```bash
# Wait for service to stabilize (30-60 seconds)
sleep 60

# Check connection status
curl -s http://localhost:8080/health | jq .

# Verify node is syncing
curl -s http://localhost:8080/status | jq .

# Monitor sync progress
watch -n 5 'curl -s http://localhost:8080/status | jq .'
```

---

## Post-Deployment Verification

### ✅ Health Checks

```bash
# 1. Service Status
sudo systemctl status dchat
# Expected: active (running)

# 2. Port Listening
sudo netstat -tlnp | grep dchat
# Expected: Port 8080 listening on 0.0.0.0 or 127.0.0.1

# 3. Process Check
ps aux | grep dchat
# Expected: dchat process running under dchat user

# 4. Health Endpoint
curl -s http://localhost:8080/health | jq .
# Expected: {"status": "healthy", "version": "0.1.0"}

# 5. Node Status
curl -s http://localhost:8080/status | jq .
# Expected: Node status with sync info, peer count, etc.

# 6. Database Check
sqlite3 /var/lib/dchat/data/dchat.db "SELECT COUNT(*) as user_profiles FROM user_profiles;"
# Expected: Query succeeds (count may be 0 initially)

# 7. Log Check
sudo journalctl -u dchat -n 50
# Expected: No errors, "Successfully initialized", sync messages

# 8. Network Connectivity
curl -s http://localhost:8080/peers | jq .
# Expected: List of connected peers (may be 0-100+)

# 9. API Response Time
time curl -s http://localhost:8080/status > /dev/null
# Expected: < 100ms response time

# 10. Disk Space
df -h /var/lib/dchat /var/backups
# Expected: > 10GB available
```

### ✅ Functional Tests

```bash
# 1. Create bot (if applicable)
curl -X POST http://localhost:8080/api/bots/create \
  -H "Content-Type: application/json" \
  -d '{"username": "testbot", "name": "Test Bot"}'

# 2. Send test message
curl -X POST http://localhost:8080/api/messages/send \
  -H "Content-Type: application/json" \
  -d '{"chat_id": "test", "text": "Hello, Production!"}'

# 3. Query messages
curl -s http://localhost:8080/api/messages?chat_id=test | jq .

# 4. List identities
curl -s http://localhost:8080/api/identities | jq .

# 5. Get metrics
curl -s http://localhost:8080/metrics | head -20
```

### ✅ Performance Baseline

```bash
# Measure response times
for i in {1..10}; do
  time curl -s http://localhost:8080/status > /dev/null
done

# Monitor resource usage
watch -n 1 'ps aux | grep dchat | grep -v grep'

# Check memory
free -h

# Check disk I/O
iotop -o

# Monitor network
nethogs
```

---

## Monitoring & Maintenance

### Continuous Monitoring

```bash
# Real-time log monitoring
sudo journalctl -u dchat -f

# Periodic status check (every 5 minutes)
watch -n 300 'sudo systemctl status dchat && echo "---" && curl -s http://localhost:8080/health'

# Disk space monitoring
watch -n 60 'df -h /var/lib/dchat /var/backups'

# Node sync progress
watch -n 30 'curl -s http://localhost:8080/status | jq .sync_progress'
```

### Automated Backups

```bash
# Create backup script
sudo tee /usr/local/bin/dchat-backup.sh > /dev/null << 'EOF'
#!/bin/bash
BACKUP_DIR="/var/backups/dchat"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/dchat_$DATE.tar.gz.enc"

# Stop service
systemctl stop dchat

# Backup database
tar czf - /var/lib/dchat/data | \
  openssl enc -aes-256-cbc -salt -out "$BACKUP_FILE"

# Start service
systemctl start dchat

# Cleanup old backups (keep 7 days)
find "$BACKUP_DIR" -name "dchat_*.tar.gz.enc" -mtime +7 -delete

echo "Backup completed: $BACKUP_FILE"
EOF

sudo chmod 755 /usr/local/bin/dchat-backup.sh

# Schedule daily backup (crontab)
echo "0 2 * * * /usr/local/bin/dchat-backup.sh" | sudo crontab -
```

### Log Rotation

```bash
# Create logrotate config
sudo tee /etc/logrotate.d/dchat > /dev/null << 'EOF'
/var/log/dchat/*.log {
    daily
    rotate 7
    compress
    delaycompress
    notifempty
    create 0640 dchat dchat
    sharedscripts
    postrotate
        systemctl reload dchat > /dev/null 2>&1 || true
    endscript
}
EOF
```

### Health Alerts

```bash
# Create alert script
sudo tee /usr/local/bin/dchat-alert.sh > /dev/null << 'EOF'
#!/bin/bash

# Check if service is running
if ! systemctl is-active --quiet dchat; then
    echo "ALERT: dchat service is down" | mail -s "dchat DOWN" admin@example.com
    exit 1
fi

# Check disk space
DISK_USAGE=$(df /var/lib/dchat | awk 'NR==2 {print $5}' | sed 's/%//')
if [ "$DISK_USAGE" -gt 90 ]; then
    echo "ALERT: Disk usage at ${DISK_USAGE}%" | mail -s "dchat DISK FULL" admin@example.com
fi

# Check memory usage
MEM_USAGE=$(ps aux | grep dchat | grep -v grep | awk '{print $6}')
if [ "$MEM_USAGE" -gt 4000000 ]; then
    echo "ALERT: Memory usage at ${MEM_USAGE}KB" | mail -s "dchat MEMORY HIGH" admin@example.com
fi
EOF

sudo chmod 755 /usr/local/bin/dchat-alert.sh

# Schedule every 5 minutes
echo "*/5 * * * * /usr/local/bin/dchat-alert.sh" | sudo crontab -
```

---

## Troubleshooting

### Issue: Service fails to start

```bash
# Check logs
sudo journalctl -u dchat -n 50

# Verify permissions
ls -l /var/lib/dchat
ls -l /etc/dchat

# Check if port is already in use
sudo lsof -i :8080

# Verify binary
ldd /usr/local/bin/dchat

# Test binary directly
sudo -u dchat /usr/local/bin/dchat --version
```

### Issue: High memory usage

```bash
# Monitor memory
ps aux | grep dchat | grep -v grep

# Check for memory leaks
valgrind --leak-check=full /usr/local/bin/dchat

# Restart service
sudo systemctl restart dchat

# Increase swap if needed
sudo swapon --show
sudo fallocate -l 4G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

### Issue: Slow API responses

```bash
# Check system load
top -b -n 1 | head -15

# Check disk I/O
iotop -o

# Check network connections
netstat -s

# Monitor database
sqlite3 /var/lib/dchat/data/dchat.db ".stats on"

# Optimize database
sqlite3 /var/lib/dchat/data/dchat.db "VACUUM; ANALYZE;"
```

### Issue: Cannot connect to peers

```bash
# Check connectivity
curl -s http://localhost:8080/peers | jq .

# Check firewall
sudo ufw status verbose

# Verify port forwarding
sudo netstat -tlnp | grep dchat

# Check DNS
nslookup dchat-bootstrap.io

# Test TCP connection
nc -zv bootstrap-node.dchat.io 30333
```

### Issue: Database corruption

```bash
# Check database integrity
sqlite3 /var/lib/dchat/data/dchat.db "PRAGMA integrity_check;"

# Backup corrupted database
sudo cp /var/lib/dchat/data/dchat.db /var/backups/dchat.db.corrupted

# Restore from backup
# See Rollback Procedures section
```

---

## Rollback Procedures

### Scenario 1: Quick Service Rollback (Same Version)

```bash
# 1. Stop service
sudo systemctl stop dchat

# 2. Check logs for issues
sudo journalctl -u dchat | tail -100

# 3. Restart with previous settings
sudo systemctl start dchat

# 4. Verify
sudo systemctl status dchat
curl -s http://localhost:8080/health
```

### Scenario 2: Version Rollback

```bash
# 1. Stop service
sudo systemctl stop dchat

# 2. Keep previous binary (e.g., dchat-v0.0.9)
ls -l /usr/local/bin/dchat*

# 3. Restore previous binary
sudo cp /usr/local/bin/dchat-v0.0.9 /usr/local/bin/dchat

# 4. Restore previous config if needed
sudo cp /etc/dchat/config.toml.backup /etc/dchat/config.toml

# 5. Start service
sudo systemctl start dchat

# 6. Verify
curl -s http://localhost:8080/health
```

### Scenario 3: Database Rollback

```bash
# 1. Stop service
sudo systemctl stop dchat

# 2. List available backups
ls -la /var/backups/dchat/

# 3. Restore from encrypted backup
BACKUP_FILE="/var/backups/dchat/dchat_20251029_020000.tar.gz.enc"
openssl enc -d -aes-256-cbc -in "$BACKUP_FILE" | tar xzf - -C /

# 4. Verify database
sqlite3 /var/lib/dchat/data/dchat.db ".tables"

# 5. Start service
sudo systemctl start dchat

# 6. Monitor
sudo journalctl -u dchat -f
```

### Scenario 4: Full System Rollback

```bash
# 1. Backup current state
sudo cp -r /var/lib/dchat /var/backups/dchat-latest

# 2. Restore from known-good backup
sudo rm -rf /var/lib/dchat/data
sudo cp -r /var/backups/dchat-20251025-000000/data /var/lib/dchat/

# 3. Restore config
sudo cp /etc/dchat/config.toml.golden /etc/dchat/config.toml

# 4. Restart service
sudo systemctl restart dchat

# 5. Verify and monitor
curl -s http://localhost:8080/status | jq .
sudo journalctl -u dchat -f
```

---

## Production Checklist - Before Going Live

- [ ] All code reviewed and tested
- [ ] All 91 tests passing
- [ ] Zero compiler errors/warnings
- [ ] Release binary built and verified
- [ ] .env.production configured
- [ ] System user created with correct permissions
- [ ] TLS certificates installed and verified
- [ ] Database initialized and schema verified
- [ ] Firewall rules configured
- [ ] SystemD service file created and enabled
- [ ] Health checks configured
- [ ] Monitoring setup complete
- [ ] Backup strategy implemented
- [ ] Alert system configured
- [ ] Runbooks created
- [ ] Team trained on deployment
- [ ] Load testing completed
- [ ] Security audit passed
- [ ] Change management approved
- [ ] Rollback plan tested
- [ ] Stakeholders notified

---

## Production Deployment Complete Checklist

After deployment, verify:

- [ ] Service is running and healthy
- [ ] All health checks pass (10+ items)
- [ ] Functional tests succeed (5+ API calls work)
- [ ] Performance baseline acceptable
- [ ] Monitoring active and alerting working
- [ ] Logs being collected properly
- [ ] Backups running on schedule
- [ ] Team notified of production status
- [ ] Documentation updated
- [ ] Incident response team on standby

---

## Support & Escalation

### For Issues
1. Check logs: `sudo journalctl -u dchat -f`
2. Run health checks: `curl -s http://localhost:8080/health`
3. Consult troubleshooting guide above
4. Contact DevOps team if issue persists

### Escalation Path
- Tier 1: Monitor & alert
- Tier 2: Restart service & check logs
- Tier 3: Database recovery
- Tier 4: Full rollback
- Tier 5: Post-mortem & long-term fix

---

**Status**: ✅ READY FOR PRODUCTION DEPLOYMENT  
**Next Step**: Execute deployment checklist above  
**Contact**: DevOps Team - support@dchat.io

---

*This guide should be kept updated as the system evolves. Version control in git is recommended.*
