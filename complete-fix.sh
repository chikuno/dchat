#!/bin/bash
# Complete fix script for validator permissions and port 9090

echo "=========================================="
echo "dchat Deployment Fix Script"
echo "=========================================="
echo ""

# Step 1: Kill port 9090
echo "Step 1: Freeing port 9090..."
echo "-------------------------------------------"
sudo fuser -k 9090/tcp 2>/dev/null && echo "✓ Killed process on port 9090" || echo "✓ Port 9090 already free"
docker stop $(docker ps -q --filter "publish=9090") 2>/dev/null && echo "✓ Stopped Docker container on port 9090" || true
echo ""

# Step 2: Stop current deployment
echo "Step 2: Stopping current deployment..."
echo "-------------------------------------------"
cd /opt/dchat || { echo "❌ /opt/dchat not found!"; exit 1; }
docker-compose -f docker-compose-production.yml down 2>/dev/null || docker-compose down 2>/dev/null || true
echo "✓ Stopped all containers"
echo ""

# Step 3: Check validator_keys permissions
echo "Step 3: Checking validator_keys permissions..."
echo "-------------------------------------------"
ls -la validator_keys/ 2>/dev/null || { 
    echo "❌ validator_keys directory not found!"
    echo "Creating validator_keys directory..."
    mkdir -p validator_keys
}
echo ""

# Step 4: Fix permissions
echo "Step 4: Fixing validator_keys permissions..."
echo "-------------------------------------------"
echo "Current permissions:"
ls -la validator_keys/ | head -5

echo ""
echo "Fixing ownership to UID 1000 (dchat user)..."
sudo chown -R 1000:1000 validator_keys/
chmod 755 validator_keys/
chmod 644 validator_keys/*.key 2>/dev/null || echo "⚠️  No key files found to chmod"

echo ""
echo "New permissions:"
ls -la validator_keys/ | head -5
echo "✓ Permissions fixed"
echo ""

# Step 5: Verify port 9090 is free
echo "Step 5: Verifying port 9090 is free..."
echo "-------------------------------------------"
if nc -z localhost 9090 2>/dev/null; then
    echo "❌ Port 9090 is STILL IN USE!"
    echo "Trying harder to free it..."
    sudo lsof -ti :9090 | xargs sudo kill -9 2>/dev/null || true
    sleep 2
    if nc -z localhost 9090 2>/dev/null; then
        echo "❌ FAILED: Port 9090 cannot be freed!"
        echo "Manual intervention required:"
        sudo lsof -i :9090 || sudo ss -tlnp | grep :9090
        exit 1
    else
        echo "✓ Port 9090 is now free"
    fi
else
    echo "✓ Port 9090 is free"
fi
echo ""

# Step 6: Start deployment
echo "Step 6: Starting deployment..."
echo "-------------------------------------------"
docker-compose -f docker-compose-production.yml up -d

echo ""
echo "Waiting 15 seconds for services to initialize..."
sleep 15
echo ""

# Step 7: Check status
echo "Step 7: Checking deployment status..."
echo "-------------------------------------------"
echo ""
echo "Container Status:"
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | grep dchat || docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
echo ""

echo "Validator Logs (validator1):"
docker logs dchat-validator1 --tail=10 2>&1 | grep -E "Error|Permission|✓" || docker logs dchat-validator1 --tail=10
echo ""

# Step 8: Health check
echo "Step 8: Running health checks..."
echo "-------------------------------------------"
sleep 5

echo "Validator1 health:"
curl -s http://localhost:7071/health 2>/dev/null && echo "✓ validator1 healthy" || echo "❌ validator1 unhealthy"

echo "Prometheus health:"
curl -s http://localhost:9090/-/healthy 2>/dev/null && echo "✓ Prometheus healthy" || echo "❌ Prometheus unhealthy"

echo ""
echo "=========================================="
echo "Fix Complete!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "1. Check validator logs: docker logs dchat-validator1 --tail=50"
echo "2. Check all containers: docker ps"
echo "3. Monitor deployment: docker-compose -f docker-compose-production.yml logs -f"
echo ""
