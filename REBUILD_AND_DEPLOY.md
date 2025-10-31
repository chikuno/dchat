# Rebuild and Redeploy Instructions

## Problem
The Docker image on the server was built **before** we added `curl` to the runtime stage. The containers can't run health checks because curl is missing.

## Solution: Rebuild Image on Server

SSH into your server (rpc.webnetcore.top) and run these commands:

```bash
# 1. Navigate to project directory
cd /opt/dchat

# 2. Pull latest Dockerfile changes (with curl added)
git pull origin main

# 3. Stop all running containers
docker-compose -f docker-compose-testnet.yml down

# 4. Rebuild the image with curl included
docker-compose -f docker-compose-testnet.yml build --no-cache

# 5. Start the testnet
docker-compose -f docker-compose-testnet.yml up -d

# 6. Wait for services to initialize (30 seconds)
sleep 30

# 7. Check container health
docker ps --format "table {{.Names}}\t{{.Status}}"

# 8. Run test suite
sudo ./test-deployment.sh
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
