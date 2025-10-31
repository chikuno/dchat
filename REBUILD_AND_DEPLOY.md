# Rebuild and Redeploy Instructions

## Problems Fixed

1. ✅ **Missing curl**: Added curl to Dockerfile runtime stage for health checks
2. ✅ **Health server on wrong port**: Added `--health-addr 0.0.0.0:XXXX` to all commands
3. ✅ **Incomplete Prometheus config**: Updated to scrape all 17 containers (4 validators + 7 relays + 3 users)
4. ✅ **No external access**: Created nginx reverse proxy configuration

## Solution: Rebuild and Deploy on Server

SSH into your server (rpc.webnetcore.top) and run these commands:

```bash
# 1. Navigate to project directory
cd /opt/dchat

# 2. Pull latest changes (Dockerfile, docker-compose, monitoring configs, nginx)
git pull origin main

# 3. Stop all running containers
docker-compose -f docker-compose-testnet.yml down

# 4. Rebuild the image with all fixes
docker-compose -f docker-compose-testnet.yml build --no-cache

# 5. Start the testnet
docker-compose -f docker-compose-testnet.yml up -d

# 6. Wait for services to initialize (60 seconds for all containers)
sleep 60

# 7. Check container health
docker ps --format "table {{.Names}}\t{{.Status}}"

# 8. Run test suite
sudo ./test-deployment.sh
```

## Optional: Setup Nginx Reverse Proxy

For external access to Prometheus, Grafana, and Jaeger:

```bash
# 1. Install nginx if not already installed
sudo apt-get update && sudo apt-get install -y nginx

# 2. Copy nginx configuration
sudo cp /opt/dchat/nginx-testnet.conf /etc/nginx/sites-available/dchat-testnet

# 3. Enable the site
sudo ln -s /etc/nginx/sites-available/dchat-testnet /etc/nginx/sites-enabled/

# 4. Test configuration
sudo nginx -t

# 5. Restart nginx
sudo systemctl restart nginx

# 6. Test external access
curl http://rpc.webnetcore.top/health
curl http://rpc.webnetcore.top/prometheus/
curl http://rpc.webnetcore.top/grafana/
curl http://rpc.webnetcore.top/jaeger/
```

## Firewall Configuration

Ensure the following ports are open:

```bash
# Allow HTTP/HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Allow Docker swarm (if used)
sudo ufw allow 2377/tcp
sudo ufw allow 7946/tcp
sudo ufw allow 7946/udp
sudo ufw allow 4789/udp

# Check firewall status
sudo ufw status
```

## What Changed

In `Dockerfile`, we added curl to the runtime stage:

```dockerfile
# Before (missing curl):
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    libc6 \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# After (with curl for healthchecks):
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    libc6 \
    curl \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
```

## Expected Results

After rebuilding:
- ✅ All containers should show "healthy" status
- ✅ Health endpoints should respond (ports 7071, 7073, 7075, 7077)
- ✅ Prometheus should be accessible on port 9095
- ✅ Grafana should be accessible on port 3000
- ✅ Validators should start producing blocks

## Troubleshooting

If containers are still unhealthy after rebuild:

```bash
# Check specific container logs
docker logs dchat-validator1 --tail=100

# Test health endpoint manually
docker exec dchat-validator1 curl -f http://localhost:7071/health

# Verify curl is installed in container
docker exec dchat-validator1 which curl
# Should output: /usr/bin/curl

# Check if dchat process is running
docker exec dchat-validator1 ps aux | grep dchat
```

## Quick Verification Commands

```bash
# All containers healthy?
docker ps --filter "health=unhealthy"
# Should return empty list

# Test validator endpoint from host
curl http://localhost:7071/health
# Should return: {"status":"healthy"}

# Check Prometheus targets
curl http://localhost:9095/api/v1/targets | jq '.data.activeTargets | length'
# Should show number of active targets

# View validator logs
docker logs dchat-validator1 --tail=50
```

## Time Estimate
- Image rebuild: ~5-10 minutes (depending on server speed)
- Container startup: ~30 seconds
- Total: ~10-15 minutes

## Notes
- The `--no-cache` flag ensures a clean rebuild with the new Dockerfile
- Validator keys are preserved in `./validator_keys/` directory
- Data volumes are not deleted, so existing chain state is preserved
