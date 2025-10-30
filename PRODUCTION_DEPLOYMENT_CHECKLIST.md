# dchat Production Deployment Checklist
## Deployment to rpc.webnetcore.top:8080

### ✅ Pre-Deployment Phase

#### Prerequisites
- [ ] SSH access to remote server
- [ ] Domain `rpc.webnetcore.top` configured and resolving
- [ ] Port 8080 open on firewall (inbound/outbound)
- [ ] Docker and Docker Compose installed on remote server
- [ ] Git installed on remote server
- [ ] Rust toolchain installed (for key generation)
- [ ] Minimum 16GB RAM available (4x validators @ 2GB each)
- [ ] Minimum 100GB disk space available

#### Server Preparation
- [ ] SSH key configured for passwordless login
- [ ] Server running Ubuntu 20.04 LTS or newer
- [ ] System packages updated: `apt update && apt upgrade`
- [ ] Swap space configured (8GB recommended)
- [ ] NTP synchronized: `timedatectl set-ntp true`
- [ ] Firewall rules configured for ports 7070-7087, 9090-9103, 3000, 8080
- [ ] Backup directory created: `/opt/dchat/backups`
- [ ] Monitoring user created (optional): `useradd -s /bin/false dchat`

---

### ✅ Deployment Phase

#### Repository Setup
- [ ] SSH into remote server
- [ ] Clone dchat repository: `git clone https://github.com/chikuno/dchat.git /opt/dchat`
- [ ] Navigate to deployment directory: `cd /opt/dchat`
- [ ] Pull latest changes: `git pull origin main`
- [ ] Verify main branch is checked out

#### Key Generation
- [ ] Create validator_keys directory: `mkdir -p /opt/dchat/validator_keys`
- [ ] Generate 4 validator keys
  ```bash
  cargo run --release --bin key-generator -- -o validator_keys/validator1.key
  cargo run --release --bin key-generator -- -o validator_keys/validator2.key
  cargo run --release --bin key-generator -- -o validator_keys/validator3.key
  cargo run --release --bin key-generator -- -o validator_keys/validator4.key
  ```
- [ ] Set key permissions: `chmod 400 validator_keys/*.key`
- [ ] Backup keys to secure location: `cp -r validator_keys /backup/dchat_validator_keys_$(date +%Y%m%d)`
- [ ] Verify keys exist and readable: `ls -la validator_keys/`

#### Monitoring Configuration
- [ ] Create monitoring directories: `mkdir -p monitoring/{grafana/{datasources,dashboards},prometheus}`
- [ ] Copy monitoring configs from deployment script
- [ ] Verify Prometheus scrape targets configured for all validators and relays
- [ ] Configure Grafana data source pointing to Prometheus
- [ ] Create Grafana dashboards for:
  - [ ] Validator consensus metrics
  - [ ] Relay performance and uptime
  - [ ] Network latency and message throughput
  - [ ] System resource usage (CPU, memory, disk)

#### Docker Image Build
- [ ] Build Docker image: `docker build -t dchat:latest -f Dockerfile .`
- [ ] Verify image built successfully: `docker images | grep dchat`
- [ ] Tag image with version: `docker tag dchat:latest dchat:v1.0`
- [ ] Optional: Push to registry for backups

#### Service Startup
- [ ] Use provided Docker Compose file: `docker-compose-production.yml`
- [ ] Start all services: `docker-compose -f docker-compose-production.yml up -d`
- [ ] Verify containers running: `docker ps | grep dchat`
- [ ] Check container status: `docker-compose -f docker-compose-production.yml ps`
- [ ] All containers should show "Up" status with health "healthy"

#### Service Health Verification
- [ ] Wait 30-60 seconds for services to stabilize
- [ ] Health check endpoint for validator1: `curl http://localhost:7071/health`
- [ ] Health check validator2: `curl http://localhost:7073/health`
- [ ] Health check validator3: `curl http://localhost:7075/health`
- [ ] Health check validator4: `curl http://localhost:7077/health`
- [ ] Health check relay1: `curl http://localhost:7081/health`
- [ ] All health checks should return status "healthy"

#### Prometheus Configuration
- [ ] Access Prometheus: `http://localhost:9090`
- [ ] Verify all 4 validators appear in targets: Status → Targets
- [ ] Verify all 4 relays appear in targets
- [ ] Check for "UP" status (green) for all targets
- [ ] Verify metrics are being scraped (Prometheus → Graph → search `dchat_`)

#### Grafana Setup
- [ ] Access Grafana: `http://localhost:3000`
- [ ] Login with default credentials: admin/admin
- [ ] Change admin password immediately
- [ ] Configure data source (should be auto-configured):
  - [ ] Configuration → Data Sources
  - [ ] Verify Prometheus data source is active
  - [ ] Test connection successful
- [ ] Import dashboards:
  - [ ] Create dashboard for validator metrics
  - [ ] Create dashboard for relay metrics
  - [ ] Create dashboard for network topology
  - [ ] Create dashboard for system resources

#### Jaeger Tracing Setup
- [ ] Access Jaeger UI: `http://localhost:16686`
- [ ] Verify service list populated
- [ ] Check for traces from validators and relays
- [ ] Configure trace retention (14 days recommended)

---

### ✅ Nginx Reverse Proxy Configuration

#### Install Nginx (on remote server)
- [ ] Install Nginx: `sudo apt install nginx`
- [ ] Create configuration directory: `sudo mkdir -p /etc/nginx/conf.d`
- [ ] Copy Nginx config from deployment script to `/etc/nginx/sites-available/dchat`
- [ ] Enable site: `sudo ln -s /etc/nginx/sites-available/dchat /etc/nginx/sites-enabled/`
- [ ] Test configuration: `sudo nginx -t`
- [ ] Reload Nginx: `sudo systemctl reload nginx`

#### SSL/TLS Configuration (Recommended)
- [ ] Install Certbot: `sudo apt install certbot python3-certbot-nginx`
- [ ] Generate certificate: `sudo certbot certonly --standalone -d rpc.webnetcore.top`
- [ ] Update Nginx config with SSL settings
- [ ] Configure certificate auto-renewal: `sudo systemctl enable certbot.timer`

#### Reverse Proxy Verification
- [ ] Test RPC access through reverse proxy: `curl http://rpc.webnetcore.top:8080/health`
- [ ] Monitor Nginx logs: `tail -f /var/log/nginx/dchat_access.log`
- [ ] Monitor Nginx errors: `tail -f /var/log/nginx/dchat_error.log`
- [ ] Load test with multiple concurrent requests
- [ ] Verify rate limiting is working

---

### ✅ Network & Consensus Verification

#### Validator Consensus
- [ ] Query validator1 peer count: `curl http://localhost:7071/peers | jq '.peer_count'`
- [ ] Query validator2 peer count: `curl http://localhost:7073/peers | jq '.peer_count'`
- [ ] Query validator3 peer count: `curl http://localhost:7075/peers | jq '.peer_count'`
- [ ] Query validator4 peer count: `curl http://localhost:7077/peers | jq '.peer_count'`
- [ ] All validators should show 3-4 peer connections (other validators)

#### Block Production
- [ ] Query latest block: `curl http://localhost:7071/chain/latest-block`
- [ ] Verify block contains transactions from all chains
- [ ] Monitor block production rate: `curl http://localhost:7071/chain/blocks?limit=10`
- [ ] Verify block time is ~3 seconds (configured in environment)
- [ ] Check for missing blocks or gaps

#### Message Propagation
- [ ] Send test message through relay: `curl -X POST http://localhost:7081/message -d '...'`
- [ ] Verify message appears on chat chain
- [ ] Check message ordering on-chain
- [ ] Measure message propagation latency (should be <1s)

#### Cross-Chain Bridge
- [ ] Query bridge status: `curl http://localhost:7071/bridge/status`
- [ ] Verify both chains are synchronized
- [ ] Send test cross-chain transaction
- [ ] Verify transaction atomicity

---

### ✅ Monitoring & Alerting

#### Prometheus Alerts
- [ ] Configure alert rules for:
  - [ ] Validator down (3 consecutive failed health checks)
  - [ ] High CPU usage (>80% per container)
  - [ ] High memory usage (>90% per container)
  - [ ] Disk space low (<20% free)
  - [ ] Block production stopped (no new blocks for 5 minutes)
  - [ ] Peer disconnections (consensus < 3/4)

#### Alertmanager Setup (Optional)
- [ ] Configure Alertmanager to receive alerts
- [ ] Setup notification channels:
  - [ ] Email notifications for critical alerts
  - [ ] Slack/Discord webhooks
  - [ ] PagerDuty for on-call alerts
- [ ] Test alert delivery with test alert

#### Log Aggregation (Optional)
- [ ] Configure centralized logging (ELK, Loki, Datadog)
- [ ] Export logs from all containers
- [ ] Set up log-based alerts for errors/warnings

---

### ✅ Performance Tuning

#### System Optimization
- [ ] Configure kernel parameters for networking:
  ```bash
  sudo sysctl -w net.core.rmem_max=134217728
  sudo sysctl -w net.core.wmem_max=134217728
  sudo sysctl -w net.ipv4.tcp_rmem="4096 87380 67108864"
  sudo sysctl -w net.ipv4.tcp_wmem="4096 65536 67108864"
  ```
- [ ] Increase file descriptor limits: `ulimit -n 100000`
- [ ] Configure connection tracking: `sysctl -w net.netfilter.nf_conntrack_max=500000`

#### Docker Resource Management
- [ ] Verify memory limits per container: 2GB for validators, 1.5GB for relays
- [ ] Verify CPU limits per container: 1.5 cores for validators, 1 core for relays
- [ ] Monitor actual usage vs limits: `docker stats`
- [ ] Configure swap limits to prevent swapping

#### Database Optimization
- [ ] Configure RocksDB cache size: 1GB per validator
- [ ] Optimize write buffer size based on workload
- [ ] Enable compression for historical data

---

### ✅ Security Hardening

#### Access Control
- [ ] Restrict SSH access to whitelisted IPs
- [ ] Disable password authentication (use SSH keys only)
- [ ] Configure fail2ban to prevent brute-force attacks
- [ ] Set up firewall rules to restrict port access:
  - [ ] Port 8080: Allow from public (RPC endpoint)
  - [ ] Ports 7070-7087: Allow internal only
  - [ ] Ports 9090-9103: Allow internal only
  - [ ] Port 3000: Allow internal only

#### Data Protection
- [ ] Encrypt validator keys at rest: `gpg --encrypt validator_keys/*`
- [ ] Store validator keys in secure backup location (off-server)
- [ ] Enable file-level encryption for validator data directories
- [ ] Regular backup schedule: daily snapshots + off-site backup

#### Network Security
- [ ] Enable DDoS protection via reverse proxy
- [ ] Configure rate limiting at Nginx level
- [ ] Monitor for unusual traffic patterns
- [ ] Configure IDS/IPS (optional): Suricata or Snort

#### Container Security
- [ ] Use non-root user inside containers
- [ ] Enable AppArmor or SELinux profiles
- [ ] Scan Docker image for vulnerabilities: `trivy image dchat:latest`
- [ ] Keep base image updated

---

### ✅ Backup & Disaster Recovery

#### Backup Strategy
- [ ] Create daily backup script using provided backup.sh
- [ ] Schedule backup via cron: `0 2 * * * /opt/dchat/backups/backup.sh`
- [ ] Backup includes:
  - [ ] Validator keys (encrypted)
  - [ ] Block data
  - [ ] State snapshots
  - [ ] Configuration files
- [ ] Verify backup integrity: `tar -tzf /backup/dchat-backup-*.tar.gz`

#### Off-Site Backup
- [ ] Upload daily backups to secure storage:
  - [ ] AWS S3 with encryption
  - [ ] Azure Blob Storage with encryption
  - [ ] Google Cloud Storage
- [ ] Retain 30 days of daily backups + weekly archives
- [ ] Test restore from off-site backup monthly

#### Disaster Recovery Testing
- [ ] Document recovery procedure in recovery.sh
- [ ] Monthly disaster recovery drill:
  - [ ] Simulate validator failure
  - [ ] Restore from backup
  - [ ] Verify consensus resumes
  - [ ] Verify no data loss
- [ ] Test with different failure scenarios

---

### ✅ Operational Runbooks

#### Daily Operations
- [ ] Morning: Check Grafana dashboard for anomalies
- [ ] Monitor block production (should have new block every 3 seconds)
- [ ] Monitor validator peer connections (should maintain 3 peers minimum)
- [ ] Check backup completion logs
- [ ] Verify no alert threshold violations

#### Weekly Maintenance
- [ ] Review logs for errors/warnings
- [ ] Check disk usage and clean old logs if needed
- [ ] Verify backup integrity and restoration test
- [ ] Performance baseline comparison

#### Monthly Tasks
- [ ] Full system security audit
- [ ] Dependency and package updates
- [ ] SSL certificate renewal (if expiring soon)
- [ ] Disaster recovery drill
- [ ] Capacity planning review

#### Incident Response
- [ ] Validator crash: Restart container and check logs
  ```bash
  docker-compose -f docker-compose-production.yml restart validator1
  docker logs dchat-validator1 --tail 100
  ```
- [ ] Network partition: Check peer connections and restart relays
  ```bash
  docker-compose -f docker-compose-production.yml restart relay1 relay2 relay3 relay4
  ```
- [ ] Consensus stuck: Restore from latest snapshot
  ```bash
  /opt/dchat/backups/recover.sh /backup/dchat-backup-latest.tar.gz
  ```
- [ ] Out of disk: Archive old blocks and restart services

---

### ✅ Post-Deployment Verification

#### Final Validation Checklist
- [ ] All 4 validators running and healthy
- [ ] All 4 relays running and healthy
- [ ] RPC endpoint accessible from external network: `curl http://rpc.webnetcore.top:8080/health`
- [ ] Prometheus scraping all targets successfully
- [ ] Grafana dashboard showing metrics
- [ ] Jaeger receiving traces from services
- [ ] Nginx reverse proxy working correctly
- [ ] SSL/TLS certificate valid and configured
- [ ] Rate limiting active and effective
- [ ] Backup system operational and tested

#### Performance Baseline
- [ ] RPC response time: < 100ms (p95)
- [ ] Block production: Consistent 3-second interval
- [ ] Message propagation: < 1 second end-to-end
- [ ] Validator consensus: 4/4 nodes agreeing on canonical chain
- [ ] Network throughput: > 1000 msg/sec
- [ ] CPU usage: 30-50% per validator
- [ ] Memory usage: 60-70% of allocated (1.2GB of 2GB)
- [ ] Disk I/O: < 100 IOPS during normal operation

#### Documentation
- [ ] Document server IP address and access credentials (secure location)
- [ ] Create runbook for common operational tasks
- [ ] Document incident response procedures
- [ ] Create monitoring alert escalation process
- [ ] Maintain topology diagram (validators, relays, monitoring)
- [ ] Record validator key backup locations (encrypted, off-site)

---

### ✅ Sign-Off

#### Deployment Approval
- [ ] Technical lead: _____________________ Date: _______
- [ ] Operations lead: ____________________ Date: _______
- [ ] Security lead: _____________________ Date: _______

#### Production Status
- [ ] Date deployed: _____________________
- [ ] Version deployed: __________________
- [ ] Deployment completed by: ___________
- [ ] Production ready: [ ] YES [ ] NO

---

## 📋 Quick Reference Commands

### Service Management
```bash
# Start all services
docker-compose -f docker-compose-production.yml up -d

# Stop all services
docker-compose -f docker-compose-production.yml down

# View service status
docker-compose -f docker-compose-production.yml ps

# View logs for specific service
docker logs dchat-validator1 -f

# Restart specific service
docker-compose -f docker-compose-production.yml restart validator1
```

### Health Checks
```bash
# Check all validators
for i in 1 2 3 4; do echo "Validator $i:"; curl http://localhost:$((7070 + i*2))/health; done

# Check all relays
for i in 1 2 3 4; do echo "Relay $i:"; curl http://localhost:$((7080 + i*2))/health; done

# Check consensus
curl http://localhost:7071/chain/consensus-status | jq
```

### Monitoring
```bash
# View Prometheus targets
curl http://localhost:9090/api/v1/targets

# View Prometheus metrics
curl http://localhost:9090/metrics | grep dchat_

# View container resource usage
docker stats --no-stream
```

### Backup & Recovery
```bash
# Create backup
/opt/dchat/backups/backup.sh

# List backups
ls -lah /opt/dchat/backups/

# Restore from backup
/opt/dchat/backups/recover.sh /opt/dchat/backups/dchat-backup-YYYYMMDD_HHMMSS.tar.gz
```

### Troubleshooting
```bash
# View recent errors
docker-compose -f docker-compose-production.yml logs --tail=50 | grep ERROR

# Check network connectivity between containers
docker exec dchat-validator1 ping dchat-relay1

# Verify block production
curl http://localhost:7071/chain/blocks | jq '.blocks | length'
```

---

**Deployment Date**: ________________
**Deployed By**: ____________________
**Last Updated**: ___________________
