#!/bin/bash
# Test user node messaging after urgent fix

set -e

# Configuration
SERVER="${1:-4.221.211.71}"
KEY="${2:-$HOME/Downloads/anacreon.pem}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m'

echo -e "${CYAN}🧪 Testing dchat User Node Messaging${NC}"
echo -e "${CYAN}=====================================${NC}"
echo ""

# Test 1: Check health endpoints
echo -e "${YELLOW}🏥 Test 1: Checking health endpoints...${NC}"
echo -e "${WHITE}Validators:${NC}"
ssh -i "$KEY" azureuser@"$SERVER" 'for i in 1 2 3 4; do echo -n "validator$i: "; curl -s -f http://localhost:$((9089+i))/health > /dev/null 2>&1 && echo "✅" || echo "❌"; done'
echo ""
echo -e "${WHITE}Relays (sample):${NC}"
ssh -i "$KEY" azureuser@"$SERVER" 'for i in 1 2 3; do echo -n "relay$i: "; curl -s -f http://localhost:$((9099+i))/health > /dev/null 2>&1 && echo "✅" || echo "❌"; done'
echo ""

# Test 2: Check container health status
echo -e "${YELLOW}🩺 Test 2: Docker health status...${NC}"
ssh -i "$KEY" azureuser@"$SERVER" 'sudo docker ps --format "table {{.Names}}\t{{.Status}}" | grep dchat | head -15'
echo ""

# Test 3: Check user node logs
echo -e "${YELLOW}📋 Test 3: User node logs (checking for errors)...${NC}"
echo -e "${CYAN}--- User1 (last 20 lines) ---${NC}"
ssh -i "$KEY" azureuser@"$SERVER" 'sudo docker logs dchat-user1-test --tail 20 2>&1'
echo ""
echo -e "${CYAN}--- User2 (last 20 lines) ---${NC}"
ssh -i "$KEY" azureuser@"$SERVER" 'sudo docker logs dchat-user2-test --tail 20 2>&1'
echo ""

# Test 4: Check for InsufficientPeers error
echo -e "${YELLOW}🔍 Test 4: Checking for InsufficientPeers errors...${NC}"
USER1_ERROR=$(ssh -i "$KEY" azureuser@"$SERVER" 'sudo docker logs dchat-user1-test 2>&1 | grep -c "InsufficientPeers" || echo 0')
USER2_ERROR=$(ssh -i "$KEY" azureuser@"$SERVER" 'sudo docker logs dchat-user2-test 2>&1 | grep -c "InsufficientPeers" || echo 0')

if [ "$USER1_ERROR" -eq 0 ] && [ "$USER2_ERROR" -eq 0 ]; then
    echo -e "${GREEN}✅ No InsufficientPeers errors found!${NC}"
else
    echo -e "${RED}❌ Found InsufficientPeers errors: user1=$USER1_ERROR, user2=$USER2_ERROR${NC}"
fi
echo ""

# Test 5: Check relay message stats
echo -e "${YELLOW}📊 Test 5: Relay message statistics...${NC}"
ssh -i "$KEY" azureuser@"$SERVER" 'sudo docker logs dchat-relay1 2>&1 | grep "bandwidth" | tail -1'
echo ""

# Test 6: Check validator block production
echo -e "${YELLOW}⛓️  Test 6: Validator block production...${NC}"
ssh -i "$KEY" azureuser@"$SERVER" 'sudo docker logs dchat-validator1 --tail 5 2>&1 | grep "block"'
echo ""

# Test 7: Count healthy containers
echo -e "${YELLOW}📈 Test 7: Health status summary...${NC}"
HEALTHY=$(ssh -i "$KEY" azureuser@"$SERVER" 'sudo docker ps --format "{{.Status}}" | grep -c "healthy" || echo 0')
UNHEALTHY=$(ssh -i "$KEY" azureuser@"$SERVER" 'sudo docker ps --format "{{.Status}}" | grep -c "unhealthy" || echo 0')
TOTAL=$(ssh -i "$KEY" azureuser@"$SERVER" 'sudo docker ps --format "{{.Names}}" | grep -c "dchat" || echo 0')

echo -e "Total containers: $TOTAL"
echo -e "${GREEN}Healthy: $HEALTHY${NC}"
echo -e "${RED}Unhealthy: $UNHEALTHY${NC}"
echo ""

# Summary
echo -e "${GREEN}✅ Tests complete!${NC}"
echo ""
echo -e "${CYAN}🔍 Key indicators to check:${NC}"
echo -e "  ✓ Health endpoints should return status"
echo -e "  ✓ Container health should be 'healthy' (not 'unhealthy')"
echo -e "  ✓ User nodes should NOT show 'InsufficientPeers' error"
echo -e "  ✓ Validators should be producing blocks"
echo ""

if [ "$UNHEALTHY" -gt 0 ]; then
    echo -e "${YELLOW}⚠️  Warning: Some containers are still unhealthy${NC}"
    echo -e "   Run: ssh -i '$KEY' azureuser@$SERVER 'sudo docker ps' for details"
fi
