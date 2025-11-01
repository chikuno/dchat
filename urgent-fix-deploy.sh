#!/bin/bash
# Urgent Fix Deployment Script for Linux VM
# Fixes: 1) Health check endpoint, 2) Gossipsub mesh configuration

set -e  # Exit on error

echo "🚀 dchat Urgent Fix Deployment"
echo "================================"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SERVER="4.221.211.71"
KEY="$HOME/Downloads/anacreon.pem"
REMOTE_USER="azureuser"
REMOTE_DIR="~/chain/dchat"

# Check if key exists
if [ ! -f "$KEY" ]; then
    echo -e "${RED}❌ SSH key not found at: $KEY${NC}"
    echo "Please set the correct path to your SSH key"
    exit 1
fi

# Step 1: Build Docker image locally
echo -e "${YELLOW}📦 Step 1: Building Rust project...${NC}"
cargo build --release
if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Build failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Build successful${NC}"
echo ""

# Step 2: Build Docker image
echo -e "${YELLOW}🐳 Step 2: Building Docker image...${NC}"
docker build -t dchat:latest .
if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Docker build failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Docker image built${NC}"
echo ""

# Step 3: Save Docker image
echo -e "${YELLOW}💾 Step 3: Saving Docker image...${NC}"
docker save dchat:latest | gzip > dchat-latest.tar.gz
if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Docker save failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Image saved to dchat-latest.tar.gz${NC}"
echo ""

# Step 4: Upload to server
echo -e "${YELLOW}⬆️  Step 4: Uploading to server...${NC}"
scp -i "$KEY" \
    dchat-latest.tar.gz \
    docker-compose-testnet.yml \
    "$REMOTE_USER@$SERVER:$REMOTE_DIR/"
if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Upload failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Files uploaded${NC}"
echo ""

# Step 5: Deploy on server
echo -e "${YELLOW}🚀 Step 5: Deploying on server...${NC}"
ssh -i "$KEY" "$REMOTE_USER@$SERVER" << 'ENDSSH'
cd ~/chain/dchat
echo '📥 Loading Docker image...'
gunzip -c dchat-latest.tar.gz | sudo docker load
echo '🛑 Stopping existing containers...'
sudo docker-compose -f docker-compose-testnet.yml down
echo '🧹 Cleaning up old containers...'
sudo docker system prune -f
echo '🚀 Starting updated stack...'
sudo docker-compose -f docker-compose-testnet.yml up -d
echo '✅ Deployment complete!'
echo ''
echo '📊 Container status:'
sudo docker ps --format 'table {{.Names}}\t{{.Status}}' | grep dchat | head -15
echo ''
echo '⏳ Waiting 30s for services to stabilize...'
sleep 30
echo ''
echo '🏥 Health check status:'
for i in 1 2 3 4; do
    echo -n "validator$i: "
    curl -s -f http://localhost:$((9089+i))/health > /dev/null 2>&1 && echo "✅ healthy" || echo "❌ unhealthy"
done
ENDSSH

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Deployment failed!${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}✅ Urgent fixes deployed successfully!${NC}"
echo ""
echo -e "${CYAN}🔍 Fixes applied:${NC}"
echo -e "  1. Health server now binds to 0.0.0.0:8080 (accessible from Docker)"
echo -e "  2. All health checks updated to port 8080"
echo -e "  3. Extended gossipsub mesh formation wait: 30s + conditional 10s"
echo -e "  4. Added retry logic: 3 attempts with 2s delays for message publishing"
echo -e "  5. Added gossipsub subscription/mesh logging for diagnostics"
echo -e "  6. Dynamic mesh status checking before publishing"
echo ""
echo -e "${YELLOW}📝 To verify:${NC}"
echo -e "  ssh -i '$KEY' $REMOTE_USER@$SERVER 'sudo docker ps'"
echo ""
echo -e "${YELLOW}🔬 To test user messaging:${NC}"
echo -e "  ssh -i '$KEY' $REMOTE_USER@$SERVER 'sudo docker logs dchat-user1-test --tail 20'"
echo ""

# Cleanup
echo -e "${YELLOW}🧹 Cleaning up local tar.gz...${NC}"
rm -f dchat-latest.tar.gz
echo -e "${GREEN}✅ Cleanup complete${NC}"
