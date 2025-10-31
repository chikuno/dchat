# dchat User Interaction Guide

## Overview
This guide explains how to interact with the dchat testnet as a user, check logs, and perform common tasks.

## Prerequisites
- SSH access to server (4.222.211.71)
- Docker containers running on the server
- PowerShell on your local machine

## Quick Start

### 1. Check Container Logs
```powershell
# Check all container logs
.\check-testnet-logs.ps1

# Check specific user with custom tail
.\check-testnet-logs.ps1 -TailLines 100
```

### 2. Login as a User
```powershell
# Login as user1 (default)
.\login-as-user.ps1

# Login as a specific user
.\login-as-user.ps1 -UserNode user2
.\login-as-user.ps1 -UserNode user3
```

## User Node Information

| Node | P2P Port | RPC Port | Metrics Port | Container Name |
|------|----------|----------|--------------|----------------|
| user1 | 7110 | 7111 | 9110 | dchat-user1 |
| user2 | 7112 | 7113 | 9111 | dchat-user2 |
| user3 | 7114 | 7115 | 9112 | dchat-user3 |

## Available User Actions

### 1. Check Node Status
```bash
curl http://localhost:7111/health
curl http://localhost:7111/status
curl http://localhost:7111/ready
```

Returns JSON with:
- Node health status
- Version information
- Timestamp
- Connection status

### 2. View Messages
```bash
# Get recent messages
curl http://localhost:7111/messages?limit=10

# Get messages from specific channel
curl http://localhost:7111/channels/{channel_id}/messages?limit=10
```

### 3. Send a Message
```bash
# Direct message
curl -X POST http://localhost:7111/send \
  -H "Content-Type: application/json" \
  -d '{
    "recipient": "user_address_here",
    "message": "Hello from dchat!"
  }'

# Channel message
curl -X POST http://localhost:7111/channels/{channel_id}/send \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Hello channel!"
  }'
```

### 4. Channel Operations

#### Create Channel
```bash
curl -X POST http://localhost:7111/channels/create \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Channel",
    "description": "A test channel",
    "is_public": true
  }'
```

#### List Channels
```bash
# All channels
curl http://localhost:7111/channels

# My channels
curl http://localhost:7111/channels/mine

# Search channels
curl http://localhost:7111/channels?search=test
```

#### Join Channel
```bash
curl -X POST http://localhost:7111/channels/{channel_id}/join
```

#### Leave Channel
```bash
curl -X POST http://localhost:7111/channels/{channel_id}/leave
```

### 5. Wallet Operations
```bash
# Check balance
curl http://localhost:7111/wallet/balance

# Get wallet address
curl http://localhost:7111/wallet/address

# Send tokens
curl -X POST http://localhost:7111/wallet/send \
  -H "Content-Type: application/json" \
  -d '{
    "recipient": "recipient_address",
    "amount": 100
  }'
```

### 6. Peer Information
```bash
# List connected peers
curl http://localhost:7111/peers

# Peer count
curl http://localhost:7111/peers/count

# Network info
curl http://localhost:7111/network/info
```

## Manual SSH Commands

### Connect to Server
```bash
ssh root@4.222.211.71
```

### Check Container Status
```bash
# All containers
docker ps

# Specific user
docker ps --filter name=dchat-user1

# Health status
docker ps --format 'table {{.Names}}\t{{.Status}}'
```

### View Logs
```bash
# Tail logs
docker logs dchat-user1 --tail=50

# Follow logs (real-time)
docker logs -f dchat-user1

# Since timestamp
docker logs dchat-user1 --since=10m

# With timestamps
docker logs dchat-user1 --timestamps
```

### Execute Commands in Container
```bash
# Health check
docker exec dchat-user1 curl -s http://localhost:7111/health | jq

# Interactive shell
docker exec -it dchat-user1 /bin/bash

# Run dchat CLI (if available)
docker exec dchat-user1 dchat --help
```

### Container Management
```bash
# Restart a user node
docker-compose -f docker-compose-testnet.yml restart user1

# Stop a user node
docker-compose -f docker-compose-testnet.yml stop user1

# Start a user node
docker-compose -f docker-compose-testnet.yml start user1

# Restart all containers
docker-compose -f docker-compose-testnet.yml restart
```

## Testing Scenarios

### Scenario 1: Send Your First Message
1. Check node status: `curl http://localhost:7111/status`
2. Get your wallet address: `curl http://localhost:7111/wallet/address`
3. Get recipient address from user2: `curl http://localhost:7113/wallet/address`
4. Send message from user1 to user2
5. Check messages on user2: `curl http://localhost:7113/messages`

### Scenario 2: Create and Use a Channel
1. Create channel: `POST /channels/create`
2. Get channel ID from response
3. Join channel from user2: `POST /channels/{id}/join`
4. Send message to channel: `POST /channels/{id}/send`
5. View channel messages: `GET /channels/{id}/messages`

### Scenario 3: Test Relay Functionality
1. Send message from user1 to user3 (different networks)
2. Check relay logs: `docker logs dchat-relay1`
3. Verify message delivery
4. Check relay metrics: `curl http://localhost:7081/metrics`

### Scenario 4: Monitor Performance
1. Send multiple messages
2. Check metrics: `curl http://localhost:7111/metrics`
3. View Prometheus: http://4.222.211.71:9095
4. View Grafana: http://4.222.211.71:3000
5. View Jaeger traces: http://4.222.211.71:16686

## Troubleshooting

### Container Not Responding
```bash
# Check if container is running
docker ps | grep dchat-user1

# Check health status
docker inspect dchat-user1 | grep -A5 Health

# Restart container
docker-compose -f docker-compose-testnet.yml restart user1

# Check logs for errors
docker logs dchat-user1 --tail=100 | grep -i error
```

### Health Check Failing
```bash
# Test health endpoint manually
docker exec dchat-user1 curl -v http://localhost:7111/health

# Check if port is listening
docker exec dchat-user1 netstat -tlnp | grep 7111

# Check firewall
sudo ufw status | grep 7111
```

### Connection Issues
```bash
# Check peer connections
docker exec dchat-user1 curl http://localhost:7111/peers

# Check network connectivity
docker network inspect dchat-testnet

# Ping another container
docker exec dchat-user1 ping dchat-validator1
```

### Database Issues
```bash
# Check database size
docker exec dchat-user1 du -sh /data/*.db

# Check database integrity
docker exec dchat-user1 sqlite3 /data/dchat.db "PRAGMA integrity_check;"

# Backup database
docker cp dchat-user1:/data/dchat.db ./backup-user1.db
```

## Performance Monitoring

### Check Resource Usage
```bash
# Container stats
docker stats dchat-user1 --no-stream

# All containers
docker stats --no-stream

# Detailed info
docker exec dchat-user1 top -b -n 1
```

### Check Metrics
```bash
# Prometheus metrics
curl http://localhost:7111/metrics

# Parsed metrics (if prometheus client available)
curl http://localhost:7111/metrics | grep dchat_messages_sent
```

### Network Statistics
```bash
# Network connections
docker exec dchat-user1 netstat -an | grep ESTABLISHED

# Bandwidth usage (if iftop installed)
docker exec dchat-user1 iftop -t -s 1
```

## Advanced Usage

### Multi-User Testing Script
```bash
#!/bin/bash
# test-multi-user.sh

# Get addresses
ADDR1=$(curl -s http://localhost:7111/wallet/address | jq -r .address)
ADDR2=$(curl -s http://localhost:7113/wallet/address | jq -r .address)
ADDR3=$(curl -s http://localhost:7115/wallet/address | jq -r .address)

echo "User1: $ADDR1"
echo "User2: $ADDR2"
echo "User3: $ADDR3"

# Send messages
curl -X POST http://localhost:7111/send \
  -H "Content-Type: application/json" \
  -d "{\"recipient\": \"$ADDR2\", \"message\": \"Hello from User1\"}"

curl -X POST http://localhost:7113/send \
  -H "Content-Type: application/json" \
  -d "{\"recipient\": \"$ADDR3\", \"message\": \"Hello from User2\"}"

curl -X POST http://localhost:7115/send \
  -H "Content-Type: application/json" \
  -d "{\"recipient\": \"$ADDR1\", \"message\": \"Hello from User3\"}"

# Check messages
echo "User1 messages:"
curl -s http://localhost:7111/messages?limit=5 | jq

echo "User2 messages:"
curl -s http://localhost:7113/messages?limit=5 | jq

echo "User3 messages:"
curl -s http://localhost:7115/messages?limit=5 | jq
```

### Channel Testing Script
```bash
#!/bin/bash
# test-channels.sh

# Create channel
CHANNEL=$(curl -s -X POST http://localhost:7111/channels/create \
  -H "Content-Type: application/json" \
  -d '{"name": "Test Channel", "description": "Testing", "is_public": true}' | jq -r .channel_id)

echo "Created channel: $CHANNEL"

# Join from other users
curl -X POST http://localhost:7113/channels/$CHANNEL/join
curl -X POST http://localhost:7115/channels/$CHANNEL/join

# Send messages
curl -X POST http://localhost:7111/channels/$CHANNEL/send \
  -H "Content-Type: application/json" \
  -d '{"message": "Message from User1"}'

curl -X POST http://localhost:7113/channels/$CHANNEL/send \
  -H "Content-Type: application/json" \
  -d '{"message": "Message from User2"}'

# View messages
curl -s http://localhost:7111/channels/$CHANNEL/messages | jq
```

## Security Notes

1. **Never expose RPC ports directly to the internet** - Use nginx reverse proxy
2. **Always use HTTPS** for external access - Configured via nginx
3. **Rotate keys regularly** - Follow key rotation procedures
4. **Monitor suspicious activity** - Check logs regularly
5. **Keep containers updated** - Rebuild images with latest security patches

## Useful Links

- **Prometheus**: http://4.222.211.71:9095
- **Grafana**: http://4.222.211.71:3000 (default: admin/admin)
- **Jaeger**: http://4.222.211.71:16686
- **HTTPS RPC**: https://rpc.webnetcore.top/health

## Next Steps

1. Test basic message sending between users
2. Create and test channels
3. Monitor performance metrics
4. Test relay functionality
5. Verify end-to-end encryption
6. Test account recovery mechanisms
7. Simulate network failures
8. Load test with multiple concurrent users

For more information, see:
- `ARCHITECTURE.md` - System architecture
- `API_SPECIFICATION.md` - Complete API reference
- `docker-compose-testnet.yml` - Container configuration
