#!/bin/bash
# Script to forcefully free port 9090

echo "=========================================="
echo "Force-Free Port 9090"
echo "=========================================="
echo ""

# Step 1: Kill any process using the port
echo "1. Attempting to kill process on port 9090..."
PID=$(sudo lsof -ti :9090 2>/dev/null)
if [ -n "$PID" ]; then
    echo "   Found PID: $PID"
    sudo kill -9 $PID
    echo "   ✓ Killed process $PID"
else
    echo "   No process found with lsof"
fi

# Alternative method using fuser
sudo fuser -k 9090/tcp 2>/dev/null && echo "   ✓ Killed process with fuser" || echo "   No process found with fuser"

echo ""

# Step 2: Stop any Docker containers using the port
echo "2. Stopping Docker containers on port 9090..."
CONTAINERS=$(docker ps --filter "publish=9090" -q)
if [ -n "$CONTAINERS" ]; then
    echo "   Found containers: $CONTAINERS"
    docker stop $CONTAINERS
    docker rm $CONTAINERS
    echo "   ✓ Stopped and removed containers"
else
    echo "   No Docker containers found"
fi

# Check for Prometheus containers specifically
PROM_CONTAINERS=$(docker ps -a | grep prometheus | awk '{print $1}')
if [ -n "$PROM_CONTAINERS" ]; then
    echo "   Found Prometheus containers: $PROM_CONTAINERS"
    docker rm -f $PROM_CONTAINERS
    echo "   ✓ Force removed Prometheus containers"
fi

echo ""

# Step 3: Clean up Docker networks
echo "3. Cleaning up Docker networks..."
docker network prune -f
echo "   ✓ Pruned unused networks"

echo ""

# Step 4: Verify port is free
echo "4. Verifying port 9090 is now free..."
if nc -z localhost 9090 2>/dev/null; then
    echo "   ❌ Port 9090 is STILL IN USE!"
    echo ""
    echo "   Detailed analysis:"
    sudo lsof -i :9090 || sudo netstat -tlnp | grep :9090 || sudo ss -tlnp | grep :9090
    echo ""
    echo "   MANUAL ACTION REQUIRED!"
    exit 1
else
    echo "   ✅ Port 9090 is now FREE!"
fi

echo ""
echo "=========================================="
echo "Port 9090 is ready for use!"
echo "=========================================="
echo ""
echo "You can now run: docker-compose -f docker-compose-production.yml up -d"
echo ""
