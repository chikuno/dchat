# dchat Testnet: 4 Validators + 7 Relays + 3 User Nodes - Complete Guide

## Overview

This guide explains how to spin up a full dchat testnet with:
- **4 Validator Nodes**: Consensus layer (BFT)
- **7 Relay Nodes**: Message delivery layer
- **3 User Nodes**: End-user clients
- **Monitoring Stack**: Prometheus, Grafana, Jaeger

---

## Architecture Diagram

```
                                    ┌─────────────────────────────────┐
                                    │    CONSENSUS LAYER              │
                                    │  (4 Validator Nodes)            │
                                    │                                 │
                              ┌─────┼─────────────────────────────┐   │
                              │     │                             │   │
                         Validator1 Validator2 Validator3 Validator4│
                              │     │             (Block Time: 2s) │
                              └─────┼─────────────────────────────┘   │
                                    └─────────────────────────────────┘
                                                  ▲
                                    ┌─────────────┴─────────────┐
                                    │                           │
                                    ▼
                        ┌───────────────────────────┐
                        │  MESSAGE DELIVERY LAYER   │
                        │   (7 Relay Nodes)         │
                        │                           │
            ┌───────────┼──────────┬────────────┬───┼────────────┐
            │           │          │            │   │            │
          Relay1       Relay2    Relay3       Relay4 Relay5    Relay6  Relay7
            │           │          │            │   │            │
            └───────────┼──────────┬────────────┬───┼────────────┘
                        │                           │
                        ▼                           ▼
            ┌──────────────────────────────────────────────────┐
            │        END-USER LAYER (3 User Nodes)             │
            │                                                  │
            │       User1          User2          User3        │
            │    (Connected to:  (Connected to:  (Connected to:│
            │    Relay1,2,3)    Relay4,5,1)    Relay6,7,2)   │
            └──────────────────────────────────────────────────┘
```

---

## Prerequisites

1. **Docker & Docker Compose**: v20.10+
2. **4GB RAM minimum** (8GB recommended for comfortable operation)
3. **Rust toolchain**: If building locally
4. **PowerShell 7+**: For running test scripts
5. **curl**: For health checks

---

## Quick Start

### Step 1: Start the Testnet

```powershell
# Navigate to dchat directory
cd c:\Users\USER\dchat

# Run the testnet startup script
.\scripts\testnet-message-propagation.ps1 -Action start
```

**What happens:**
- Generates validator keys
- Builds Docker image with Rust bookworm base
- Starts 4 validators (2-second block time)
- Starts 7 relay nodes (connected to validators)
- Starts 3 user nodes (distributed across relays)
- Starts monitoring stack (Prometheus, Grafana, Jaeger)

**Expected time:** 3-5 minutes (first build) or 30-45 seconds (subsequent)

### Step 2: Check Node Health

```powershell
.\scripts\testnet-message-propagation.ps1 -Action health
```

**Expected output:**
```
✅ Validator1 (port 7071): HEALTHY
✅ Validator2 (port 7073): HEALTHY
✅ Validator3 (port 7075): HEALTHY
✅ Validator4 (port 7077): HEALTHY
✅ Relay1 (port 7081): HEALTHY
✅ Relay2 (port 7083): HEALTHY
... (and so on)
✅ User1 (port 7111): HEALTHY
✅ User2 (port 7113): HEALTHY
✅ User3 (port 7115): HEALTHY
```

### Step 3: Check Network Status

```powershell
.\scripts\testnet-message-propagation.ps1 -Action status
```

This shows:
- Validator blockchain height
- Connected peers
- Registered users

### Step 4: Send Test Message

```powershell
# Send message from user1 to user2
.\scripts\testnet-message-propagation.ps1 -Action send-message `
    -FromUser user1 -ToUser user2 -Message "Hello from dchat testnet!"
```

**Test Flow:**
1. **user1** sends message via RPC to relay (encrypted with user2's public key)
2. **Relay** stores message temporarily (user2 offline detection)
3. **Validator** records message ordering and timestamp on-chain
4. **Relay** detects user2 coming online
5. **user2** receives message from relay
6. **user2** decrypts and displays message

---

## Detailed Node Configuration

### Validator Nodes (validator1-4)

**Purpose**: Blockchain consensus and state management

**Configuration:**
- **Consensus Algorithm**: BFT-style (2/3 honest majority needed)
- **Block Time**: 2 seconds
- **Timeout**: 2000ms for consensus rounds
- **Network Ports**: 7070 (P2P), 7071 (RPC), 9090-9093 (metrics)
- **Storage**: Per-validator RocksDB instance
- **Key Location**: `/validator_keys/validator1.key`, etc.

**Responsibilities:**
- Validate transactions
- Produce blocks every 2 seconds
- Maintain message ordering
- Execute governance decisions
- Update reputation scores

**Health Indicators:**
- Block height increasing every 2 seconds
- 3+ peers connected
- Consensus participation >95%
- No errors in logs

**Docker Port Mapping:**
| Node | P2P | RPC | Metrics |
|------|-----|-----|---------|
| validator1 | 7070 | 7071 | 9090 |
| validator2 | 7072→7070 | 7073→7071 | 9091→9090 |
| validator3 | 7074→7070 | 7075→7071 | 9092→9090 |
| validator4 | 7076→7070 | 7077→7071 | 9093→9090 |

### Relay Nodes (relay1-7)

**Purpose**: Store-and-forward message delivery

**Configuration:**
- **Network Ports**: 7080 (P2P), 7081 (RPC), 9100-9106 (metrics)
- **Storage**: Message queue (SQLite, ~24hr retention)
- **Staking**: 100 tokens per relay
- **Geographic Distribution**: Simulated via subnet diversity
- **Failover**: Automatic if relay goes down

**Responsibilities:**
- Accept messages from users
- Queue messages for offline recipients
- Submit proof-of-delivery to validators
- Participate in DHT peer discovery
- Handle message retries with exponential backoff

**Connectivity Pattern:**
- Relay1: Primary bootstrap point for other relays
- Relay2-7: Each connects to Relay1 and then discovers others via DHT
- All relays connect to all 4 validators

**Expected Performance:**
- Message delivery latency: <500ms (same relay) to 2s (cross-relay)
- Queue depth: <1000 messages per relay
- CPU usage: <10% per relay
- Memory: ~200MB per relay

### User Nodes (user1-3)

**Purpose**: End-user clients for messaging

**Configuration:**
- **Network Ports**: 7110 (P2P), 7111 (RPC), 9110-9112 (metrics)
- **Storage**: Local SQLite with full-text search
- **Backup**: Optional encrypted cloud sync
- **Bootstrap Relays**: 3 relays per user (for resilience)

**Connectivity Pattern:**
| User | Primary Relays | Bootstrap |
|------|----------------|-----------|
| user1 | relay1, relay2, relay3 | relay1:7080 |
| user2 | relay4, relay5, relay1 | relay4:7080 |
| user3 | relay6, relay7, relay2 | relay6:7080 |

**Expected Behavior:**
- Automatic peer discovery via bootstrap
- Encrypted local message cache
- Real-time message notifications
- Offline message sync on reconnect
- Reputation-based rate limiting (per relay)

---

## Message Propagation Flow (Detailed)

### Scenario: user1 → user2 (same relay)

```
Time=0ms:  user1 creates message
           - Encrypts with user2's public key
           - Attaches sender proof (signed with user1's key)
           - Adds timestamp and sequence number

Time=10ms: user1 sends to relay1
           - Relay1 verifies sender signature
           - Relay1 checks message size (<1MB)
           - Relay1 stores in local queue
           - Relay1 returns message_id to user1

Time=50ms: relay1 detects user2 is connected
           - Looks up user2's active relays in DHT
           - Finds user2 connected to relay1

Time=60ms: relay1 sends to user2
           - user2 receives encrypted message
           - user2 decrypts (has private key)
           - user2 sends ACK back to relay1

Time=100ms: relay1 generates proof-of-delivery
           - Creates proof: [user1, user2, msg_hash, timestamp]
           - Signs with relay1's key
           - Submits to validator

Time=150ms: validator verifies proof
           - Checks relay1 signature
           - Checks user1 & user2 identities
           - Stores in blockchain
           - Broadcasts to other validators

Time=200ms: All validators reach consensus
           - Message ordering confirmed
           - Message final (immutable)
           - Relay1 earns delivery reward

**Total Latency: ~200ms**
```

### Scenario: user1 → user2 (cross-relay, offline)

```
Time=0ms:   user1 sends to relay2
            - relay2 receives message
            - relay2 detects user2 NOT connected
            - relay2 stores with 24hr TTL

Time=0-1h:  user2 offline (can be hours/days)

Time=+1h:   user2 comes online, connects to relay3
            - relay3 queries DHT for messages destined to user2
            - relay3 finds relay2 has message for user2
            - relay3 requests message from relay2

Time=+1h+200ms:
            - relay2 sends to relay3
            - relay3 sends to user2
            - user2 decrypts and reads

**Total Latency: O(1s) after user2 comes online**
```

---

## Testing Message Propagation

### Test 1: Basic Message Send/Receive

```powershell
# User1 → User2 (direct via shared relay)
.\scripts\testnet-message-propagation.ps1 -Action send-message `
    -FromUser user1 -ToUser user2 -Message "Test 1: Basic message"
```

**Expected Result:**
- Message ID returned
- Message appears in user2's inbox within 500ms
- No errors in validator logs

---

### Test 2: Cross-Relay Message

```powershell
# User1 → User3 (through different relays)
.\scripts\testnet-message-propagation.ps1 -Action send-message `
    -FromUser user1 -ToUser user3 -Message "Test 2: Cross-relay message"
```

**Expected Result:**
- Message routed through 2-3 relays
- Delivery confirmed within 1-2 seconds
- Proof-of-delivery recorded on blockchain

---

### Test 3: Offline Message Delivery

```powershell
# Simulate: Send message while user3 is offline, then come online

# First, get logs to see user3 behavior
docker-compose -f docker-compose-testnet.yml logs user3 --tail=20

# Send message (user3 offline)
.\scripts\testnet-message-propagation.ps1 -Action send-message `
    -FromUser user2 -ToUser user3 -Message "Test 3: Offline delivery"

# Wait 5 seconds
Start-Sleep -Seconds 5

# Check if message was queued
docker-compose -f docker-compose-testnet.yml logs relay6 | Select-String "queued"
```

**Expected Result:**
- Message queued on relay6 (user3's relay)
- When user3 comes online, message auto-delivered
- Receipt confirmed in logs

---

### Test 4: Validator Consensus

```powershell
# Check validator blockchain height and consensus
for ($i=1; $i -le 4; $i++) {
    $port = 7070 + (($i-1) * 2)
    Write-Host "Validator$i:"
    curl -s "http://localhost:$port/status" | ConvertFrom-Json | `
        Select-Object height, peers, consensus_round
}
```

**Expected Result:**
- All validators at same height (within 1 block)
- 3 peers each (N-1 consensus minimum)
- Consensus round advancing every 2 seconds

---

### Test 5: Relay Reputation & Rewards

```powershell
# Check relay metrics and delivery stats
curl -s http://localhost:9100/metrics | Select-String "relay_messages_delivered|relay_uptime|relay_reputation"
```

**Expected Result:**
- Message delivery count increasing
- Uptime >99% per relay
- Reputation scores updating

---

## Monitoring & Observability

### Prometheus (http://localhost:9090)

Metrics available:
- `dchat_validator_blocks_produced`
- `dchat_validator_consensus_rounds`
- `dchat_relay_messages_delivered`
- `dchat_relay_queue_depth`
- `dchat_user_messages_sent`
- `dchat_user_messages_received`

### Grafana (http://localhost:3000)

**Login**: admin / admin

Pre-built dashboards:
1. **Network Overview**: Node count, uptime, connectivity
2. **Validator Performance**: Block height, consensus latency, peer connections
3. **Relay Metrics**: Message throughput, delivery time, queue depth
4. **User Activity**: Messages sent/received, latency, error rates

### Jaeger (http://localhost:16686)

Distributed tracing for:
- Message propagation path
- Validator consensus rounds
- Cross-relay communication
- End-to-end latency

**Search for traces:**
- Service: `dchat-user1`, `dchat-relay1`, `dchat-validator1`
- Operation: `send_message`, `deliver_message`, `validate_block`

---

## Troubleshooting

### Issue: Nodes can't connect

**Symptoms:**
```
❌ Node1 (port 7070): UNREACHABLE
```

**Solution:**
```powershell
# Check docker network
docker network inspect dchat-testnet

# Verify service is running
docker ps | grep dchat

# Check logs
docker logs dchat-validator1 --tail=50
```

### Issue: Messages not propagating

**Symptoms:**
- Send message succeeds
- Message doesn't appear on recipient
- Relay queue filling up

**Solution:**
```powershell
# Check relay connectivity
docker logs dchat-relay1 | Select-String "connected|peer"

# Check if users are registered
docker exec dchat-relay1 curl -s http://relay1:7081/users

# Check message queue
docker exec dchat-relay1 curl -s http://relay1:7081/queue | ConvertFrom-Json | Measure-Object
```

### Issue: Consensus stalled

**Symptoms:**
- Validator height not increasing
- Consensus round stuck

**Solution:**
```powershell
# Check if validators can communicate
docker exec dchat-validator1 ping dchat-validator2

# Check validator logs for errors
docker logs dchat-validator1 | Select-String "ERROR|panic"

# Restart validator
docker restart dchat-validator1
```

### Issue: High latency (>5s message delivery)

**Solutions:**
1. Check relay queue depth
2. Verify relay-validator network latency
3. Increase relay capacity if needed
4. Check validator consensus timeout

---

## Performance Baseline

**Expected Performance on 8GB System:**

| Metric | Expected | Acceptable Range |
|--------|----------|------------------|
| Message Latency (local) | 200ms | 100-500ms |
| Message Latency (cross-relay) | 1-2s | 500ms-5s |
| Validator Block Time | 2s | 1.5-3s |
| Relay Queue Depth | <100 | <1000 |
| CPU per Validator | 5-15% | <50% |
| CPU per Relay | 2-8% | <50% |
| Memory per Validator | 400MB | <2GB |
| Memory per Relay | 200MB | <1GB |
| Memory per User | 100MB | <500MB |
| Network Bandwidth | ~100KB/s | <10MB/s |

---

## Cleanup

```powershell
# Stop testnet
.\scripts\testnet-message-propagation.ps1 -Action stop

# Or manually
docker-compose -f docker-compose-testnet.yml down -v

# Remove volumes (if needed)
docker volume prune -f
```

---

## Next Steps

After confirming basic message propagation:

1. **Scale Test**: Increase to 10+ validators, 20+ relays, 100+ users
2. **Load Test**: Send 1000+ messages/second and measure throughput
3. **Failure Test**: Kill validators/relays and test resilience
4. **Geographic Distribution**: Deploy relays across multiple regions
5. **Security Audit**: Run fuzzing and attack simulations

---

## Reference: Docker Compose File Structure

```yaml
Services:
  - bootstrap: DNS seed point
  - validator1-4: Consensus layer
  - relay1-7: Message delivery
  - user1-3: End-user clients
  - prometheus: Metrics collection
  - grafana: Metrics visualization
  - jaeger: Distributed tracing

Volumes:
  - validator*_data: Blockchain state per validator
  - relay*_data: Message queue per relay
  - user*_data: Local message cache per user
  - prometheus_data: Metrics storage
  - grafana_data: Dashboard configuration

Network:
  - dchat-testnet: Custom bridge (172.28.0.0/16)
```

---

## Key Takeaways

✅ **Message Propagation Works**: Through validators and relays  
✅ **Consensus Active**: Validators maintain synchronized state  
✅ **Relay Incentives**: Delivery proofs recorded on-chain  
✅ **User Privacy**: End-to-end encryption maintained  
✅ **Observability**: Full monitoring and tracing available  

Your testnet is now **production-ready for testing**! 🚀
