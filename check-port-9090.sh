#!/bin/bash
# Script to check what's using port 9090 on Linux

echo "=========================================="
echo "Port 9090 Investigation"
echo "=========================================="
echo ""

echo "1. Checking what process is using port 9090..."
echo ""

# Method 1: lsof (most detailed)
if command -v lsof >/dev/null 2>&1; then
    echo "Using lsof:"
    sudo lsof -i :9090 || echo "   No process found with lsof"
    echo ""
fi

# Method 2: netstat
if command -v netstat >/dev/null 2>&1; then
    echo "Using netstat:"
    sudo netstat -tlnp | grep :9090 || echo "   No process found with netstat"
    echo ""
fi

# Method 3: ss (modern alternative)
if command -v ss >/dev/null 2>&1; then
    echo "Using ss:"
    sudo ss -tlnp | grep :9090 || echo "   No process found with ss"
    echo ""
fi

# Method 4: Check Docker containers
echo "2. Checking Docker containers..."
echo ""
docker ps --format "table {{.Names}}\t{{.Ports}}" | grep 9090 || echo "   No Docker containers using 9090"
echo ""

# Method 5: Check if port is actually in use
echo "3. Testing port availability..."
if nc -z localhost 9090 2>/dev/null; then
    echo "   ⚠️  Port 9090 is IN USE"
else
    echo "   ✓ Port 9090 is AVAILABLE"
fi
echo ""

# Method 6: Check for zombie Docker networks
echo "4. Checking Docker networks and endpoints..."
docker network inspect bridge 2>/dev/null | grep -A 10 "9090" || echo "   No bridge network conflicts"
echo ""

# Method 7: Check system services
echo "5. Checking systemd services on port 9090..."
sudo systemctl list-units --type=service --state=running | grep -E "prometheus|metrics" || echo "   No Prometheus-related services found"
echo ""

echo "=========================================="
echo "Suggested Solutions:"
echo "=========================================="
echo ""
echo "If a process is found:"
echo "  1. Kill it: sudo kill -9 <PID>"
echo "  2. Or use: sudo fuser -k 9090/tcp"
echo ""
echo "If Docker container found:"
echo "  1. Stop it: docker stop <container-name>"
echo "  2. Remove it: docker rm <container-name>"
echo ""
echo "If systemd service found:"
echo "  1. Stop it: sudo systemctl stop <service-name>"
echo "  2. Disable it: sudo systemctl disable <service-name>"
echo ""
echo "If nothing found but port still allocated:"
echo "  1. Reboot the system: sudo reboot"
echo "  2. Or change Prometheus port in docker-compose-production.yml"
echo ""
