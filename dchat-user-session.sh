#!/bin/bash

# Switch to dchat user and show interactive menu for testing

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${CYAN}=== dchat User Session ===${NC}"
echo ""

# Check if we're dchat user
CURRENT_USER=$(whoami)
if [ "$CURRENT_USER" != "dchat" ]; then
    echo -e "${YELLOW}Current user: $CURRENT_USER${NC}"
    echo -e "${YELLOW}Switching to dchat user...${NC}"
    exec sudo -u dchat -s
fi

echo -e "${GREEN}✓ Logged in as: $(whoami)${NC}"
echo -e "${GREEN}✓ Home directory: $(pwd)${NC}"
echo ""

# Show menu
show_menu() {
    echo -e "${CYAN}=== dchat User Actions ===${NC}"
    echo "1. Check node status"
    echo "2. View recent messages"
    echo "3. Send a message"
    echo "4. Create a channel"
    echo "5. List channels"
    echo "6. View node logs (tail)"
    echo "7. Follow logs (real-time)"
    echo "8. Check wallet balance"
    echo "9. Get connected peers"
    echo "10. Test health endpoint"
    echo "11. Open interactive shell"
    echo "12. Run all diagnostics"
    echo "0. Exit"
    echo ""
}

# Execute actions
execute_action() {
    case $1 in
        1)
            echo -e "${YELLOW}Fetching node status...${NC}"
            curl -s http://localhost:7111/status | jq '.' || echo "Failed to fetch status"
            ;;
        2)
            echo -e "${YELLOW}Fetching recent messages...${NC}"
            curl -s http://localhost:7111/messages?limit=10 | jq '.' || echo "Failed to fetch messages"
            ;;
        3)
            read -p "Enter recipient address: " recipient
            read -p "Enter message: " message
            echo -e "${YELLOW}Sending message...${NC}"
            curl -X POST http://localhost:7111/send \
              -H "Content-Type: application/json" \
              -d "{\"recipient\": \"$recipient\", \"message\": \"$message\"}" | jq '.' || echo "Failed to send"
            ;;
        4)
            read -p "Enter channel name: " name
            read -p "Enter channel description: " desc
            echo -e "${YELLOW}Creating channel...${NC}"
            curl -X POST http://localhost:7111/channels/create \
              -H "Content-Type: application/json" \
              -d "{\"name\": \"$name\", \"description\": \"$desc\", \"is_public\": true}" | jq '.' || echo "Failed to create"
            ;;
        5)
            echo -e "${YELLOW}Listing channels...${NC}"
            curl -s http://localhost:7111/channels | jq '.' || echo "Failed to fetch channels"
            ;;
        6)
            echo -e "${YELLOW}Recent logs (tail 50)...${NC}"
            docker logs dchat-user1 --tail=50 2>&1 || echo "Failed to fetch logs"
            ;;
        7)
            echo -e "${YELLOW}Following logs (press Ctrl+C to stop)...${NC}"
            docker logs -f dchat-user1 2>&1 || echo "Failed to follow logs"
            ;;
        8)
            echo -e "${YELLOW}Checking wallet balance...${NC}"
            curl -s http://localhost:7111/wallet/balance | jq '.' || echo "Failed to fetch balance"
            ;;
        9)
            echo -e "${YELLOW}Getting connected peers...${NC}"
            curl -s http://localhost:7111/peers | jq '.' || echo "Failed to fetch peers"
            ;;
        10)
            echo -e "${YELLOW}Testing health endpoint...${NC}"
            curl -s http://localhost:7111/health | jq '.' || echo "Failed to fetch health"
            ;;
        11)
            echo -e "${YELLOW}Opening interactive shell...${NC}"
            /bin/bash
            ;;
        12)
            echo -e "${YELLOW}=== Running Full Diagnostics ===${NC}"
            echo ""
            
            echo -e "${CYAN}1. Container Status:${NC}"
            docker ps --filter name=dchat-user1 --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}'
            echo ""
            
            echo -e "${CYAN}2. Health Check:${NC}"
            curl -s http://localhost:7111/health | jq '.'
            echo ""
            
            echo -e "${CYAN}3. Node Status:${NC}"
            curl -s http://localhost:7111/status | jq '.'
            echo ""
            
            echo -e "${CYAN}4. Wallet Address:${NC}"
            curl -s http://localhost:7111/wallet/address | jq '.'
            echo ""
            
            echo -e "${CYAN}5. Peer Count:${NC}"
            curl -s http://localhost:7111/peers | jq 'length'
            echo ""
            
            echo -e "${CYAN}6. Recent Messages:${NC}"
            curl -s http://localhost:7111/messages?limit=5 | jq '.'
            echo ""
            
            echo -e "${CYAN}7. Recent Logs:${NC}"
            docker logs dchat-user1 --tail=20 2>&1 | tail -10
            echo ""
            ;;
        0)
            echo -e "${GREEN}Exiting...${NC}"
            return 1
            ;;
        *)
            echo -e "${RED}Invalid choice${NC}"
            ;;
    esac
}

# Main loop
continue=true
while [ "$continue" = true ]; do
    show_menu
    read -p "Enter your choice: " choice
    echo ""
    execute_action "$choice" || continue=false
    echo ""
    if [ "$continue" = true ]; then
        read -p "Press Enter to continue..."
    fi
done

echo -e "${CYAN}=== Session Ended ===${NC}"
