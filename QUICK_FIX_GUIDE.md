# Quick Deployment Fix Guide

## 🚨 Current Issue
Containers running but showing "unhealthy" status because:
1. Health server was on port 8080 (default) but health checks expected port 7071/7081/7111
2. Commands didn't specify `--health-addr` parameter
3. Prometheus not configured to scrape all containers

## ✅ What Was Fixed

### 1. Docker Compose Commands Updated
All validator/relay/user commands now include:
- `--health-addr 0.0.0.0:XXXX` (matches RPC port)
- `--metrics-addr 0.0.0.0:XXXX` (explicit metrics port)

**Example** (validator1):
```bash
# Before:
command: validator --key /validator_keys/validator1.key --chain-rpc http://chain-rpc:26657 --stake 10000 --producer

# After:
command: validator --key /validator_keys/validator1.key --chain-rpc http://chain-rpc:26657 --stake 10000 --producer --health-addr 0.0.0.0:7071 --metrics-addr 0.0.0.0:9090
```

### 2. Prometheus Configuration Updated
`monitoring/prometheus.yml` now scrapes:
- **validators**: 4 nodes on port 9090
- **relays**: 7 nodes on ports 9100-9106
- **users**: 3 nodes on ports 9110-9112

### 3. Nginx Reverse Proxy Created
`nginx-testnet.conf` provides external access to:
- Health checks: http://rpc.webnetcore.top/health
- Prometheus: http://rpc.webnetcore.top/prometheus/
- Grafana: http://rpc.webnetcore.top/grafana/
- Jaeger: http://rpc.webnetcore.top/jaeger/

## 📋 Deployment Steps on rpc.webnetcore.top

```bash
# 1. SSH to server
ssh user@rpc.webnetcore.top

# 2. Navigate to dchat directory
cd /opt/dchat

# 3. Pull latest fixes from GitHub
git pull origin main

# 4. Stop current containers
docker-compose -f docker-compose-testnet.yml down

# 5. Rebuild image (includes curl for health checks)
docker-compose -f docker-compose-testnet.yml build --no-cache

# 6. Start all containers
docker-compose -f docker-compose-testnet.yml up -d

# 7. Wait for initialization (60 seconds)
sleep 60

# 8. Check container health - should ALL show "healthy"
docker ps --format "table {{.Names}}\t{{.Status}}"
```

## ✅ Verification Commands

```bash
# 1. Check all containers are healthy
docker ps --filter "health=unhealthy"
# Should return: EMPTY (no unhealthy containers)

# 2. Test health endpoints
curl http://localhost:7071/health  # validator1
curl http://localhost:7081/health  # relay1
curl http://localhost:7111/health  # user1

# 3. Check Prometheus targets
curl http://localhost:9095/api/v1/targets | jq '.data.activeTargets | length'
# Should return: 14 (4 validators + 7 relays + 3 users)

# 4. Run full test suite
sudo ./test-deployment.sh
# Expected: 43/43 tests passed
```

## 🔧 Optional: Setup Nginx (for external access)

```bash
# Install nginx
sudo apt-get update && sudo apt-get install -y nginx

# Copy and enable configuration
sudo cp nginx-testnet.conf /etc/nginx/sites-available/dchat-testnet
sudo ln -s /etc/nginx/sites-available/dchat-testnet /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx

# Open firewall
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Test external access
curl http://rpc.webnetcore.top/health
```

## 📊 Expected Results After Fix

### Container Status
```
NAME                 STATUS
dchat-validator1     Up X minutes (healthy)
dchat-validator2     Up X minutes (healthy)
dchat-validator3     Up X minutes (healthy)
dchat-validator4     Up X minutes (healthy)
dchat-relay1         Up X minutes (healthy)
dchat-relay2         Up X minutes (healthy)
dchat-relay3         Up X minutes (healthy)
dchat-relay4         Up X minutes (healthy)
dchat-relay5         Up X minutes (healthy)
dchat-relay6         Up X minutes (healthy)
dchat-relay7         Up X minutes (healthy)
dchat-user1          Up X minutes (healthy)
dchat-user2          Up X minutes (healthy)
dchat-user3          Up X minutes (healthy)
dchat-prometheus     Up X minutes (healthy)
dchat-grafana        Up X minutes (healthy)
dchat-jaeger         Up X minutes (healthy)
```

### Health Endpoint Response
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "timestamp": "2025-10-31T..."
}
```

### Prometheus Targets
- 4 validator targets (port 9090)
- 7 relay targets (ports 9100-9106)
- 3 user targets (ports 9110-9112)
- 1 prometheus target (port 9090)
- **Total: 15 active targets**

### Test Suite Results
```
Tests Passed:  43
Tests Failed:  0
✓ All tests passed
```

## 🐛 Troubleshooting

### If containers still unhealthy:
```bash
# Check logs
docker logs dchat-validator1 --tail=100

# Verify health endpoint inside container
docker exec dchat-validator1 curl -f http://localhost:7071/health

# Verify curl is installed
docker exec dchat-validator1 which curl
# Should output: /usr/bin/curl
```

### If Prometheus shows no targets:
```bash
# Check Prometheus logs
docker logs dchat-prometheus --tail=50

# Verify Prometheus config was mounted
docker exec dchat-prometheus cat /etc/prometheus/prometheus.yml

# Check if containers can resolve each other
docker exec dchat-prometheus ping -c 1 validator1
```

### If health endpoint returns 404:
```bash
# Verify health server is listening
docker exec dchat-validator1 netstat -tlnp | grep 7071

# Check if health server started
docker logs dchat-validator1 | grep "Health server listening"
```

## 📞 Support

If issues persist after following this guide:
1. Capture logs: `docker logs dchat-validator1 > validator1.log`
2. Check system resources: `free -h && df -h`
3. Review docker-compose file: `cat docker-compose-testnet.yml | grep -A5 "validator1:"`
4. Check git commit: `git log --oneline -1`

Current commit should be: **1bdfb87** (fix: Complete health check and monitoring configuration)

---

**Time Estimate**: 10-15 minutes (mostly rebuild time)
**Risk Level**: Low (no data loss, can rollback with `git checkout <old-commit>`)
**Expected Outcome**: All 17 containers healthy, full monitoring operational
