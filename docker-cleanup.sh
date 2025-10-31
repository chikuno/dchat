#!/bin/bash
# Docker Cleanup Script - Remove everything except dchat image
# WARNING: This will destroy all containers, volumes, and non-dchat images!

set -e

echo "========================================"
echo "Docker Cleanup Script"
echo "========================================"
echo ""
echo "This will remove:"
echo "  ✗ All containers (running and stopped)"
echo "  ✗ All volumes"
echo "  ✗ All networks (except default)"
echo "  ✗ All images (except dchat:*)"
echo ""
echo "This will preserve:"
echo "  ✓ dchat:latest and dchat:* images"
echo ""
read -p "Are you sure? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "Aborted."
    exit 0
fi

echo ""
echo "Starting cleanup..."
echo ""

# Step 1: Stop all running containers
echo "1. Stopping all running containers..."
if [ "$(docker ps -q)" ]; then
    docker stop $(docker ps -q)
    echo "   ✓ Stopped all containers"
else
    echo "   ℹ No running containers"
fi

# Step 2: Remove all containers
echo ""
echo "2. Removing all containers..."
if [ "$(docker ps -aq)" ]; then
    docker rm -f $(docker ps -aq)
    echo "   ✓ Removed all containers"
else
    echo "   ℹ No containers to remove"
fi

# Step 3: Remove all volumes
echo ""
echo "3. Removing all volumes..."
if [ "$(docker volume ls -q)" ]; then
    docker volume rm $(docker volume ls -q) 2>/dev/null || echo "   ⚠ Some volumes may be in use"
    echo "   ✓ Removed volumes"
else
    echo "   ℹ No volumes to remove"
fi

# Step 4: Remove all networks (except default ones)
echo ""
echo "4. Removing custom networks..."
CUSTOM_NETWORKS=$(docker network ls --filter type=custom -q)
if [ -n "$CUSTOM_NETWORKS" ]; then
    docker network rm $CUSTOM_NETWORKS 2>/dev/null || echo "   ⚠ Some networks may be in use"
    echo "   ✓ Removed custom networks"
else
    echo "   ℹ No custom networks to remove"
fi

# Step 5: Remove all images except dchat
echo ""
echo "5. Removing images (preserving dchat:*)..."
# Get all images except those with 'dchat' in the name
IMAGES_TO_REMOVE=$(docker images --format "{{.Repository}}:{{.Tag}}" | grep -v "^dchat:" || true)
if [ -n "$IMAGES_TO_REMOVE" ]; then
    echo "$IMAGES_TO_REMOVE" | xargs -r docker rmi -f 2>/dev/null || echo "   ⚠ Some images may be in use"
    echo "   ✓ Removed non-dchat images"
else
    echo "   ℹ No images to remove"
fi

# Step 6: Prune system
echo ""
echo "6. Pruning Docker system..."
docker system prune -f --volumes
echo "   ✓ System pruned"

# Step 7: Show what's left
echo ""
echo "========================================"
echo "Cleanup Complete!"
echo "========================================"
echo ""
echo "Remaining Docker resources:"
echo ""
echo "Images:"
docker images
echo ""
echo "Containers:"
docker ps -a
echo ""
echo "Volumes:"
docker volume ls
echo ""
echo "Networks:"
docker network ls
echo ""
echo "✅ Cleanup finished successfully!"
