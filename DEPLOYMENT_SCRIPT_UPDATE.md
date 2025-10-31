# Deployment Script Update Summary

**Updated**: October 31, 2025  
**Script**: `deploy-ubuntu-testnet.sh`  
**Commit**: caadce2

## Overview

The `deploy-ubuntu-testnet.sh` script has been enhanced to include all the latest health check fixes, complete monitoring configuration, and nginx reverse proxy setup. This ensures the deployment script matches all recent improvements from commits 03d044b, 1bdfb87, c04a09a, and 46cf000.

---

## Key Updates

### 1. **Nginx Integration** 🆕
- **Function Added**: `install_nginx()`
- **What it does**:
  - Installs nginx web server
  - Deploys `nginx-testnet.conf` configuration
  - Creates reverse proxy for external access
  - Configures load-balanced health checks
- **New CLI Option**: `--skip-nginx` to optionally skip nginx installation
- **Firewall Updates**: Opens ports 80 (HTTP) and 443 (HTTPS)

**External Access Routes**:
```
http://server-ip/health           → Load-balanced validator health checks
http://server-ip/prometheus/      → Prometheus monitoring UI
http://server-ip/grafana/         → Grafana dashboards
http://server-ip/jaeger/          → Jaeger tracing UI
http://server-ip/api/validators/  → Validator API endpoints
http://server-ip/api/relays/      → Relay API endpoints
```

---

### 2. **Complete Monitoring Configuration**
- **Prometheus Config**: Now creates complete configuration with 14 targets
  - 4 validators: `validator1-4:9090`
  - 7 relays: `relay1:9100, relay2:9100, relay3:9102, relay4:9103, relay5:9104, relay6:9105, relay7:9106`
  - 3 users: `user1:9110, user2:9111, user3:9112`
- **Role Labels**: Added for better metric organization (`role: validator/relay/user`)
- **External Labels**: Added cluster and environment tags

---

### 3. **Enhanced Health Check Testing**
- **Expanded Tests**: Now tests validators, relays, AND users
- **Prometheus Validation**: Checks that Prometheus has 14 active targets
- **Better Reporting**: Shows detailed status for each endpoint

**Test Endpoints**:
```bash
✓ Validator1 (http://localhost:7071/health)
✓ Validator2 (http://localhost:7073/health)
✓ Validator3 (http://localhost:7075/health)
✓ Validator4 (http://localhost:7077/health)
✓ Relay1 (http://localhost:7081/health)
✓ User1 (http://localhost:7111/health)
✓ Prometheus (http://localhost:9095/-/healthy)
✓ Grafana (http://localhost:3000/api/health)
✓ Prometheus has 14 active targets
```

---

### 4. **Documentation Enhancements**
- **Deployment Info**: Saves comprehensive deployment details including:
  - Nginx installation status
  - Health check port configuration
  - Complete Prometheus target list
- **Status Display**: Shows both direct and nginx-proxied access URLs
- **Better Logging**: More informative messages throughout deployment

---

## Usage

### Basic Deployment (Full Installation)
```bash
sudo ./deploy-ubuntu-testnet.sh
```
This will:
1. Update system packages
2. Install Docker and Docker Compose
3. Configure firewall (UFW)
4. **Install and configure nginx** 🆕
5. Validate project structure and keys
6. Build Docker images
7. Deploy 17 containers (4 validators + 7 relays + 3 users + 3 monitoring)
8. Wait for health checks to pass
9. Test all endpoints

### Skip Nginx Installation
```bash
sudo ./deploy-ubuntu-testnet.sh --skip-nginx
```
Use this if you don't want external access via nginx or prefer manual configuration.

### Quick Rebuild (Skip Docker Install and Build)
```bash
sudo ./deploy-ubuntu-testnet.sh --skip-docker --skip-build
```
Use this when you've already built images and just need to restart containers.

### All Options
```bash
sudo ./deploy-ubuntu-testnet.sh \
  --skip-docker        # Skip Docker installation (if already installed)
  --skip-build         # Skip building Docker images (use existing)
  --skip-nginx         # Skip nginx installation and configuration
  --monitoring-only    # Only restart monitoring stack
  --help              # Show help message
```

---

## What Gets Deployed

### Container Architecture (17 Containers)
- **4 Validators**: Consensus nodes
  - Health: port 7071
  - Metrics: port 9090
  - RPC: ports 7071, 7073, 7075, 7077
- **7 Relays**: Message routing nodes
  - Health: port 7081
  - Metrics: ports 9100-9106
  - P2P: ports 7080-7093
- **3 Users**: End-user client nodes
  - Health: port 7111
  - Metrics: ports 9110-9112
  - P2P: ports 7110-7115
- **3 Monitoring**: Observability stack
  - Prometheus: port 9095
  - Grafana: port 3000 (admin/admin)
  - Jaeger: port 16686

### Network Configuration
- **Network Name**: `dchat-testnet`
- **Network Type**: Bridge
- **Health Checks**: All containers use `curl -f http://localhost:PORT/health`
- **Expected Health Status**: All 17 containers should show `(healthy)` status

---

## Post-Deployment Verification

After running the deployment script, verify everything is working:

### 1. Check Container Health
```bash
docker ps --format "table {{.Names}}\t{{.Status}}"
```
**Expected**: All containers show `(healthy)` status after 60 seconds.

### 2. Verify Prometheus Targets
```bash
curl -s http://localhost:9095/api/v1/targets | jq '.data.activeTargets | length'
```
**Expected**: 14 (4 validators + 7 relays + 3 users)

### 3. Test Health Endpoints
```bash
# Validators
curl http://localhost:7071/health  # validator1
curl http://localhost:7073/health  # validator2
curl http://localhost:7075/health  # validator3
curl http://localhost:7077/health  # validator4

# Relays
curl http://localhost:7081/health  # relay1

# Users
curl http://localhost:7111/health  # user1
```

### 4. Test Nginx Reverse Proxy (if installed)
```bash
SERVER_IP=$(hostname -I | awk '{print $1}')

curl http://$SERVER_IP/health
curl http://$SERVER_IP/prometheus/
curl http://$SERVER_IP/grafana/
curl http://$SERVER_IP/jaeger/
```

### 5. Access Monitoring UIs
- **Grafana**: http://server-ip:3000 (or http://server-ip/grafana/)
  - Username: admin
  - Password: admin
- **Prometheus**: http://server-ip:9095 (or http://server-ip/prometheus/)
- **Jaeger**: http://server-ip:16686 (or http://server-ip/jaeger/)

---

## Troubleshooting

### Containers Show "unhealthy"
This should NOT happen with the updated script. If it does:
1. Check logs: `docker logs validator1` (or any container name)
2. Verify health endpoint: `docker exec validator1 curl -f http://localhost:7071/health`
3. Check if health server is running: `docker exec validator1 netstat -tlnp | grep 7071`

### Prometheus Has Wrong Target Count
Expected: 14 targets  
If you see fewer:
1. Check prometheus.yml: `cat monitoring/prometheus.yml`
2. Restart Prometheus: `docker restart prometheus`
3. Check targets: http://localhost:9095/targets

### Nginx Not Working
1. Test nginx config: `sudo nginx -t`
2. Check nginx status: `sudo systemctl status nginx`
3. View nginx logs: `sudo tail -f /var/log/nginx/error.log`
4. Verify config: `cat /etc/nginx/sites-enabled/dchat-testnet`

---

## Files Modified/Created

### Modified by Script
- `/etc/nginx/sites-available/dchat-testnet` - nginx configuration (from nginx-testnet.conf)
- `/etc/nginx/sites-enabled/dchat-testnet` - nginx symlink
- `monitoring/prometheus.yml` - Complete Prometheus config (if missing)
- `monitoring/grafana/datasources/prometheus.yml` - Grafana datasource

### Created by Script
- `DEPLOYMENT_INFO.txt` - Deployment summary with health check details
- `stop-testnet.sh` - Quick stop script
- `start-testnet.sh` - Quick start script
- `logs-testnet.sh` - View logs script
- `status-testnet.sh` - Check status script
- `/var/log/dchat-deployment.log` - Full deployment log

---

## Integration with Recent Fixes

This deployment script update aligns with all recent health check and monitoring fixes:

### Commit 03d044b
✅ Dockerfile includes curl for health checks

### Commit 1bdfb87
✅ All docker-compose commands use `--health-addr` and `--metrics-addr` parameters  
✅ Complete Prometheus configuration with 14 targets  
✅ Proper port mapping for validators, relays, and users

### Commit c04a09a & 46cf000
✅ Comprehensive documentation matching QUICK_FIX_GUIDE.md and HEALTH_CHECK_FIX_SUMMARY.md

### nginx-testnet.conf
✅ Nginx configuration deployed and enabled  
✅ Load-balanced health checks  
✅ External access to monitoring stack

---

## Expected Results

After running `sudo ./deploy-ubuntu-testnet.sh`:

✅ **17 containers running**  
✅ **All containers healthy** (not "unhealthy")  
✅ **14 Prometheus targets active**  
✅ **All health endpoints responding**  
✅ **Nginx reverse proxy configured** (unless `--skip-nginx`)  
✅ **External monitoring access working**  

**Deployment Time**: 10-20 minutes (mostly Docker image building)

---

## Next Steps

After successful deployment:

1. **Test Message Flow**: Send messages between user nodes
2. **Monitor Metrics**: Check Grafana dashboards for node performance
3. **Verify Consensus**: Ensure all validators are producing blocks
4. **Load Testing**: Use test scripts to simulate network load
5. **Optional SSL**: Configure Let's Encrypt for HTTPS access

For SSL setup:
```bash
sudo apt-get install certbot python3-certbot-nginx
sudo certbot --nginx -d your-domain.com
# Then uncomment SSL server block in /etc/nginx/sites-available/dchat-testnet
```

---

## Summary

The `deploy-ubuntu-testnet.sh` script now provides:
- **Complete automation** from fresh Ubuntu to running testnet
- **Proper health checks** with correct port configuration
- **Full monitoring stack** with 14 Prometheus targets
- **External access** via nginx reverse proxy
- **Comprehensive testing** and validation
- **Detailed documentation** and status reporting

This ensures consistent, reliable deployments that match all recent improvements and best practices. 🚀
