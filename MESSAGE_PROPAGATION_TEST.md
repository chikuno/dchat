# Message Propagation Testing Guide

## Current Testnet Status
- ✅ 4 Validators: Producing blocks (height ~38)
- ✅ 7 Relays: Healthy and routing
- ⚠️ 3 Users: Restarting (non-interactive mode limitation)

## Problem Identified
User nodes fail with `InsufficientPeers` error in non-interactive mode because they immediately try to publish a test message but haven't discovered relay peers yet.

## Solution Options

### Option 1: Modify User Nodes (Recommended)
Remove `--non-interactive` flag and let nodes run interactively:
```bash
# Change from:
command: user --username user1 --non-interactive

# To:
command: user --username user1
```

Then nodes will:
- Wait for peer discovery
- Establish connections to relays
- Accept interactive commands via stdin

### Option 2: Test via Direct Commands
Use docker exec to send messages directly:

```powershell
# Send test message from user1 through relay1
docker exec dchat-user1 dchat send-message --to user2 --text "Hello from testnet!"

# Query message history
docker exec dchat-user1 dchat message-history --limit 10

# Check peer connections
docker exec dchat-relay1 dchat peer-list
```

### Option 3: Enable Logging for Analysis
```powershell
# View relay routing logs
docker logs dchat-relay1 --follow

# View validator consensus logs
docker logs dchat-validator1 --follow

# Check network connections
docker exec dchat-relay1 netstat -an | grep 7070
```

## Quick Verification Commands

### Check Validator Consensus
```powershell
docker logs dchat-validator1 | Select-String "block|height"
```

### Check Relay Connectivity
```powershell
docker logs dchat-relay1 | Select-String "peer|DHT|bootstrap"
```

### Monitor Message Flow
```powershell
# Terminal 1: Watch relay logs
docker logs dchat-relay1 --follow

# Terminal 2: Watch user logs  
docker logs dchat-user1 --follow

# Terminal 3: Send test messages
docker exec dchat-user1 dchat send-message --to user2 --text "Test"
```

## Next Steps

1. **Fix user node configuration**:
   ```bash
   # Edit docker-compose-testnet.yml
   # Remove --non-interactive from user commands
   docker-compose -f docker-compose-testnet.yml down
   docker-compose -f docker-compose-testnet.yml up -d
   ```

2. **Connect to running user node**:
   ```bash
   docker exec -it dchat-user1 sh
   # Then type interactive commands
   ```

3. **Send message through the network**:
   ```bash
   docker exec dchat-user1 dchat send-message --to user2 --text "Testing message propagation"
   ```

4. **Verify reception on user2**:
   ```bash
   docker logs dchat-user2 | Select-String "received|message"
   ```

## Expected Message Path

```
User1 (localhost:7110)
  ↓
Relay1 (0.0.0.0:7080)
  ↓ (routes via DHT)
Relay2-7 (0.0.0.0:7080)
  ↓
User2 (localhost:7112)
```

## Container Addresses Reference

| Node | Internal Port | External Port | Address in Docker |
|------|---------------|---------------|--------------------|
| validator1 | 7070 | 7070 | validator1:7070 |
| validator2 | 7070 | 7072 | validator2:7070 |
| validator3 | 7070 | 7074 | validator3:7070 |
| validator4 | 7070 | 7076 | validator4:7070 |
| relay1 | 7080 | 7080 | relay1:7080 |
| relay2 | 7080 | 7082 | relay2:7080 |
| relay3 | 7080 | 7084 | relay3:7080 |
| relay4 | 7080 | 7086 | relay4:7080 |
| relay5 | 7080 | 7088 | relay5:7080 |
| relay6 | 7080 | 7090 | relay6:7080 |
| relay7 | 7080 | 7092 | relay7:7080 |
| user1 | 7110 | 7110 | user1:7110 |
| user2 | 7110 | 7112 | user2:7110 |
| user3 | 7110 | 7114 | user3:7110 |

## Logs for Quick Diagnostics

```powershell
# All validator logs
docker logs dchat-validator1 dchat-validator2 dchat-validator3 dchat-validator4

# All relay logs summary
for ($i=1; $i-le 7; $i++) { docker logs "dchat-relay$i" | Select-Object -Last 3 }

# All user logs summary
for ($i=1; $i-le 3; $i++) { docker logs "dchat-user$i" | Select-Object -Last 5 }
```

## Final Test Scenario

Once user nodes are fixed to not use `--non-interactive`:

1. ✅ All 14 nodes running
2. ✅ Validators producing blocks
3. ✅ Relays routing messages
4. ✅ Users connected to relays
5. ✅ Send message from user1 → verify in relay logs → confirm at user2

**This will demonstrate full message propagation through the testnet!**
