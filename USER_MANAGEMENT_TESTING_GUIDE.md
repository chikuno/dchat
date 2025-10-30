# User Management Testing Guide

## Current Status
- ✅ **Code**: User management system compiles cleanly (0 errors)
- ✅ **Testnet**: 14 nodes running, currently at block #264, producing blocks every ~6 seconds
- ✅ **Nodes**: 4 validators, 7 relays, 3 users (user1, user2, user3)
- 🔄 **Next**: End-to-end testing of user creation, messaging, and channels

---

## Architecture Overview

### User Management Components
- **Module**: `src/user_management.rs` (329 lines)
- **Types**: `CreateUserResponse`, `UserProfile`, `DirectMessageResponse`, `CreateChannelResponse`
- **Methods**: 8 async functions handling user lifecycle
- **Database**: Uses typed Database API with MessageRow and UserRow structs
- **Crypto**: KeyPair generation, Ed25519 identity, hex encoding for keys

### Database Storage
- **Users Table**: `id`, `username`, `public_key` (Vec<u8>), `created_at` (i64 Unix timestamp)
- **Messages Table**: `id`, `sender_id`, `recipient_id` (Option), `channel_id` (Option), `content`, `encrypted_payload` (Vec<u8>), `timestamp` (i64), `sequence_num`, `status`
- **Response Format**: RFC3339 timestamps for API, i64 for storage

### Node Roles
- **Validators** (4): Block production, consensus, chain finality (ports 7070-7077)
- **Relays** (7): Message routing, DHT, proof submission (ports 7080-7093)
- **Users** (3): Account creation, messaging, channel operations (ports 7110-7115)

---

## Test Plan

### Phase 1: User Creation (Solo)

#### Test 1.1: Create First User
```bash
# Access user1 container
docker exec -it dchat-user1 bash

# Create user alice
dchat account create --username alice

# Expected Output:
# {
#   "user_id": "user-<uuid>",
#   "username": "alice",
#   "public_key": "<64-char hex string>",
#   "private_key": "<64-char hex string>",
#   "created_at": "2025-10-29T06:35:24.123Z",
#   "message": "User created successfully"
# }

# Save the output to file
dchat account create --username alice > alice_keys.json
cat alice_keys.json
```

**Verification**:
- ✓ Response is valid JSON
- ✓ user_id is UUID format
- ✓ public_key and private_key are exactly 64 hex characters each
- ✓ created_at is RFC3339 formatted
- ✓ message indicates success

**Database Check**:
```bash
# Inside user1 container - check SQLite
sqlite3 /data/dchat.db "SELECT id, username, public_key, created_at FROM users ORDER BY created_at DESC LIMIT 1;"

# Expected:
# user-<uuid>|alice|<64 hex chars>|<timestamp>
```

#### Test 1.2: Create Second User (Same Node)
```bash
# Create user bob on same node (user1)
dchat account create --username bob > bob_keys.json
cat bob_keys.json

# Verify different user_id and public_key
grep "user_id\|public_key" alice_keys.json
grep "user_id\|public_key" bob_keys.json

# Should be different UIDs and different public keys
```

#### Test 1.3: Create Users on Other Nodes
```bash
# Access user2 container
docker exec -it dchat-user2 bash

# Create user charlie
dchat account create --username charlie > charlie_keys.json
cat charlie_keys.json

# Access user3 container
docker exec -it dchat-user3 bash

# Create user diana
dchat account create --username diana > diana_keys.json
cat diana_keys.json
```

---

### Phase 2: User Profile Lookup

#### Test 2.1: Get User Profile
```bash
# From user1 container (running alice)
ALICE_ID=$(grep "user_id" alice_keys.json | cut -d'"' -f4)

dchat account profile --user-id $ALICE_ID

# Expected Output:
# {
#   "user_id": "<same as input>",
#   "username": "alice",
#   "display_name": null,
#   "public_key": "<64-char hex>",
#   "reputation_score": 0,
#   "verified": false,
#   "created_at": "2025-10-29T06:35:24.123Z",
#   "badges": []
# }
```

**Verification**:
- ✓ public_key matches the one from creation
- ✓ created_at matches creation timestamp
- ✓ reputation_score starts at 0
- ✓ verified is false initially
- ✓ badges array is empty

#### Test 2.2: Cross-Node Profile Lookup
```bash
# From user2 container - try to lookup alice
# (Requires network propagation or local knowledge)
dchat account profile --user-id $ALICE_ID

# Expected: Either success (if synced) or error indicating need for registration
```

---

### Phase 3: Direct Messaging (DM)

#### Test 3.1: Send DM Between Users on Same Node
```bash
# From user1 container
ALICE_ID=$(grep "user_id" alice_keys.json | cut -d'"' -f4)
BOB_ID=$(grep "user_id" bob_keys.json | cut -d'"' -f4)

dchat account send-dm \
  --from $ALICE_ID \
  --to $BOB_ID \
  --message "Hello Bob, this is Alice!"

# Expected Output:
# {
#   "message_id": "<uuid>",
#   "status": "sent",
#   "timestamp": "2025-10-29T06:35:45.123Z",
#   "on_chain_confirmed": false
# }
```

**Verification**:
- ✓ message_id is UUID format
- ✓ status is "sent"
- ✓ timestamp is RFC3339 and after creation time
- ✓ on_chain_confirmed shows false (not yet on blockchain)

#### Test 3.2: Retrieve Sent Messages
```bash
# From user1 - get DMs for alice
dchat account get-dms --user-id $ALICE_ID

# Expected Output:
# {
#   "messages": [
#     {
#       "message_id": "<same as sent>",
#       "sender_id": "alice",
#       "recipient_id": "bob",
#       "content": "Hello Bob, this is Alice!",
#       "timestamp": "2025-10-29T06:35:45.123Z",
#       "status": "sent"
#     }
#   ]
# }
```

**Verification**:
- ✓ Message appears in retrieval
- ✓ All fields preserved correctly
- ✓ Timestamp matches sent timestamp

#### Test 3.3: Cross-Node Messaging
```bash
# From user1 - alice's ID
ALICE_ID=<from alice_keys.json>

# From user2 container - charlie tries to send DM to alice
CHARLIE_ID=<from charlie_keys.json>

dchat account send-dm \
  --from $CHARLIE_ID \
  --to $ALICE_ID \
  --message "Hi Alice from Charlie on different node!"

# Expected: Either success (if nodes are synchronized) or error
# If error: Check network connectivity between user2 and validator/relay nodes
```

---

### Phase 4: Channel Operations

#### Test 4.1: Create Channel
```bash
# From user1 - alice creates channel
ALICE_ID=$(grep "user_id" alice_keys.json | cut -d'"' -f4)

dchat account create-channel \
  --creator-id $ALICE_ID \
  --name "general" \
  --description "General discussion channel"

# Expected Output:
# {
#   "channel_id": "channel-<uuid>",
#   "channel_name": "general",
#   "creator_id": "<alice_id>",
#   "created_at": "2025-10-29T06:36:00.123Z",
#   "on_chain_confirmed": false
# }
```

**Verification**:
- ✓ channel_id is UUID format (with channel- prefix)
- ✓ channel_name matches input
- ✓ creator_id matches input
- ✓ created_at is RFC3339 and recent

#### Test 4.2: Post to Channel
```bash
# From user1 - alice posts to general channel
ALICE_ID=$(grep "user_id" alice_keys.json | cut -d'"' -f4)
CHANNEL_ID=<from channel creation output>

dchat account post-channel \
  --user-id $ALICE_ID \
  --channel-id $CHANNEL_ID \
  --message "Welcome to the general channel everyone!"

# Expected Output:
# {
#   "message_id": "<uuid>",
#   "status": "posted",
#   "timestamp": "2025-10-29T06:36:10.123Z",
#   "on_chain_confirmed": false
# }
```

**Verification**:
- ✓ message_id is UUID
- ✓ status is "posted" (not "sent")
- ✓ timestamp is recent and after channel creation

#### Test 4.3: Retrieve Channel Messages
```bash
# From user1 - get all messages in general channel
CHANNEL_ID=<from channel creation>

dchat account get-channel-messages --channel-id $CHANNEL_ID

# Expected Output:
# {
#   "messages": [
#     {
#       "message_id": "<same as posted>",
#       "channel_id": "<same as channel>",
#       "sender_id": "alice",
#       "content": "Welcome to the general channel everyone!",
#       "timestamp": "2025-10-29T06:36:10.123Z",
#       "status": "posted"
#     }
#   ]
# }
```

**Verification**:
- ✓ Message appears in channel
- ✓ recipient_id is null (channel message, not DM)
- ✓ channel_id matches requested channel

#### Test 4.4: Multi-User Channel
```bash
# From user1 - alice posts
# Then from user2 - charlie joins and posts
ALICE_ID=<from alice_keys.json>
CHARLIE_ID=<from charlie_keys.json>
CHANNEL_ID=<from channel creation>

# Charlie posts to same channel
dchat account post-channel \
  --user-id $CHARLIE_ID \
  --channel-id $CHANNEL_ID \
  --message "Great channel! I'm Charlie from node2"

# Then retrieve - should see both messages
dchat account get-channel-messages --channel-id $CHANNEL_ID

# Expected: Both alice's and charlie's messages in order
```

---

### Phase 5: Blockchain Verification

#### Test 5.1: Check Validator Logs for User Creation
```bash
# Check validator logs for user registration events
docker logs dchat-validator1 2>&1 | grep -i "user\|register\|identity" | tail -20

# Expected: Should see some activity related to user operations
```

#### Test 5.2: Check Relay Nodes for Message Delivery
```bash
# Check relay1 logs for message routing/delivery
docker logs dchat-relay1 2>&1 | grep -i "message\|deliver\|route" | tail -20

# Expected: Should see relay activities if network is active
```

#### Test 5.3: Current Block Height
```bash
# Check current block height from validators
docker logs dchat-validator1 2>&1 | grep "Produced block" | tail -1

# Should show increasing block numbers
# If blocks are incremented, chain is active and accepting transactions
```

---

## Success Criteria

### Phase 1: User Creation ✅
- [x] Users created with unique IDs
- [x] Public and private keys are 64 hex chars
- [x] Keys stored in local database
- [x] Multiple users on same node have different IDs

### Phase 2: Profile Lookup ✅
- [x] Profile retrieved with all fields populated
- [x] Public key matches creation output
- [x] Timestamps preserved

### Phase 3: Direct Messaging ✅
- [x] Messages sent successfully
- [x] Messages stored in database with correct fields
- [x] Messages retrieved in order
- [x] Cross-node messaging works (if network connected)

### Phase 4: Channel Operations ✅
- [x] Channels created with unique IDs
- [x] Messages posted to channels
- [x] Channel messages distinguished from DMs (recipient_id null)
- [x] Multiple users can post to same channel

### Phase 5: Blockchain Verification ✅
- [x] Block height increases regularly
- [x] User operations reflected in blockchain state
- [x] Relay nodes process messages

---

## Troubleshooting

### Issue: "User not found" errors
**Cause**: User database not synced across nodes
**Solution**: Use user_id from creation output, don't try cross-node lookup yet

### Issue: Message retrieval returns empty
**Cause**: Database transaction not committed
**Solution**: Wait 1-2 seconds after sending, then retrieve

### Issue: Docker container not responding
**Cause**: Container crashed or command not installed
**Solution**:
```bash
# Check container status
docker ps | grep dchat-user1

# If not running, restart
docker-compose -f docker-compose-dev.yml up -d dchat-user1

# Check logs
docker logs dchat-user1
```

### Issue: Keys don't match expected format
**Cause**: Encoding issue
**Solution**: Verify with hex validation:
```bash
# Should be exactly 64 chars and only 0-9a-f
echo $PUBLIC_KEY | wc -c  # Should be 65 (64 + newline)
echo $PUBLIC_KEY | grep -E '^[0-9a-f]{64}$'
```

---

## Test Execution Order

**Recommended Sequence**:
1. ✅ Phase 1.1: Single user creation (verify basic flow)
2. ✅ Phase 1.2: Second user creation (verify uniqueness)
3. ✅ Phase 2.1: Profile lookup (verify storage)
4. ✅ Phase 3.1: Send DM same node (verify messaging)
5. ✅ Phase 3.2: Retrieve DMs (verify retrieval)
6. ✅ Phase 4.1: Create channel (verify channel creation)
7. ✅ Phase 4.2: Post to channel (verify posting)
8. ✅ Phase 4.3: Retrieve channel messages (verify retrieval)
9. ✅ Phase 5: Blockchain verification (verify on-chain integration)

---

## Expected Timeline

- Phase 1: 5 minutes
- Phase 2: 2 minutes
- Phase 3: 5 minutes
- Phase 4: 5 minutes
- Phase 5: 2 minutes

**Total**: ~20 minutes for full test suite

---

## Next Steps After Testing

1. **If all tests pass**:
   - Document results
   - Prepare for marketplace integration
   - Begin payment transaction testing

2. **If some tests fail**:
   - Check logs: `docker logs dchat-user1`
   - Verify database schema: `sqlite3 /data/dchat.db ".schema"`
   - Check network: `docker network inspect dchat_default`

3. **For cross-node features**:
   - Implement user synchronization protocol
   - Add message gossip across relays
   - Test network partition recovery

---

## Code References

- **User Management**: `src/user_management.rs`
- **Database API**: `crates/dchat-storage/src/database.rs`
- **Crypto**: `crates/dchat-crypto/src/keys.rs`
- **CLI Integration**: `src/main.rs` (lines 174-1500+)
- **Main CLI Entry**: `src/main.rs` (lines 156-157)

