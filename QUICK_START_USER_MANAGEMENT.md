# User Management - Quick Start Testing

**Status**: ✅ Ready for immediate testing  
**Testnet**: 14 nodes operational, block #264+, producing blocks every ~6 seconds  
**Code**: Compiles cleanly (0 errors), all features integrated

---

## 30-Second Setup

```bash
# 1. All containers already running - verify
docker ps | grep -i dchat | wc -l  # Should show 14+ containers

# 2. Enter user1 node
docker exec -it dchat-user1 bash

# 3. Create first user inside container
dchat account create --username alice

# Expected: JSON with user_id, public_key (64 hex chars), private_key (64 hex chars)
```

---

## Core Testing Commands

### Test 1: User Creation (2 minutes)
```bash
# Inside dchat-user1 container

# Create alice
dchat account create --username alice

# Copy the user_id from output
ALICE_ID="<paste-user-id-here>"

# Create bob
dchat account create --username bob
BOB_ID="<paste-user-id-here>"

# Verify both created
echo "Alice ID: $ALICE_ID"
echo "Bob ID: $BOB_ID"
```

**Success Criteria**:
- ✓ Both commands return JSON
- ✓ user_id values are different
- ✓ public_key and private_key are exactly 64 hex characters

---

### Test 2: Direct Messaging (3 minutes)
```bash
# Inside dchat-user1 container
ALICE_ID="<from-test-1>"
BOB_ID="<from-test-1>"

# Alice sends message to Bob
dchat account send-dm --from "$ALICE_ID" --to "$BOB_ID" --message "Hello Bob!"

# Expected: {"message_id": "...", "status": "sent", "timestamp": "...", ...}

# Retrieve Alice's messages
dchat account get-dms --user-id "$ALICE_ID"

# Expected: Array containing the message just sent
```

**Success Criteria**:
- ✓ send-dm returns JSON with message_id
- ✓ get-dms returns array with the message
- ✓ Message content preserved exactly

---

### Test 3: Channel Operations (3 minutes)
```bash
# Inside dchat-user1 container
ALICE_ID="<from-test-1>"

# Create channel
dchat account create-channel \
  --creator-id "$ALICE_ID" \
  --name "general" \
  --description "General chat"

# Copy channel_id from output
CHANNEL_ID="<paste-channel-id-here>"

# Post message to channel
dchat account post-channel \
  --user-id "$ALICE_ID" \
  --channel-id "$CHANNEL_ID" \
  --message "Welcome everyone!"

# Retrieve channel messages
dchat account get-channel-messages --channel-id "$CHANNEL_ID"

# Expected: Array with the message posted
```

**Success Criteria**:
- ✓ create-channel returns JSON with channel_id
- ✓ post-channel returns status "posted"
- ✓ get-channel-messages shows all messages in order

---

### Test 4: Database Verification (2 minutes)
```bash
# Still in dchat-user1 container

# Check users table
sqlite3 /data/dchat.db "SELECT COUNT(*) as user_count FROM users;"

# Expected: 2 (alice and bob)

# Check specific user
sqlite3 /data/dchat.db \
  "SELECT id, username, length(public_key) as key_size FROM users WHERE username='alice';"

# Expected: 
# user-<uuid>|alice|32

# Check messages
sqlite3 /data/dchat.db "SELECT COUNT(*) as msg_count FROM messages;"

# Expected: Should show the messages sent
```

**Success Criteria**:
- ✓ Users table has entries for alice and bob
- ✓ public_key column has 32 bytes each (raw, not hex)
- ✓ Messages table populated with sent messages

---

### Test 5: Cross-Node Testing (5 minutes)
```bash
# In NEW terminal session, access user2 container
docker exec -it dchat-user2 bash

# Inside dchat-user2

# Create charlie
dchat account create --username charlie
CHARLIE_ID="<paste-user-id>"

# Exit and access user3
exit

# In another terminal
docker exec -it dchat-user3 bash

# Inside dchat-user3

# Create diana
dchat account create --username diana
DIANA_ID="<paste-user-id>"

# Send message from diana to charlie (cross-node)
dchat account send-dm --from "$DIANA_ID" --to "$CHARLIE_ID" --message "Hi from user3!"

# Check if delivered (may require time for network sync)
dchat account get-dms --user-id "$DIANA_ID"
```

**Success Criteria**:
- ✓ Users created on different nodes have different IDs
- ✓ Message sending works (even if delivery delayed)
- ✓ No errors from cross-node operations

---

## Blockchain Verification (1 minute)

```bash
# Check block production
docker logs dchat-validator1 2>&1 | grep "Produced block" | tail -5

# Expected output like:
# 2025-10-29T06:35:24.279Z  INFO dchat: 🔗 Produced block #262
# 2025-10-29T06:35:30.278Z  INFO dchat: 🔗 Produced block #263
# 2025-10-29T06:35:36.279Z  INFO dchat: 🔗 Produced block #264

# Verify blocks are increasing
docker logs dchat-validator1 2>&1 | grep "Produced block" | tail -1

# Expected: Should show block numbers increasing every ~6 seconds
```

**Success Criteria**:
- ✓ Blocks produced regularly (~6 second intervals)
- ✓ Block numbers increasing sequentially
- ✓ Validators remain healthy

---

## Troubleshooting Quick Reference

### Container not responding
```bash
# Check if running
docker ps | grep dchat-user1

# If not running, restart
docker-compose -f docker-compose-dev.yml up -d dchat-user1

# View logs
docker logs dchat-user1
```

### Command not found
```bash
# Verify binary is in container
docker exec dchat-user1 which dchat

# If not found, rebuild
cd /path/to/dchat
docker build -t dchat:latest .
docker-compose -f docker-compose-dev.yml up -d --force-recreate
```

### Database not found
```bash
# Check if database exists
docker exec dchat-user1 ls -la /data/

# If dchat.db missing, it will be created on first user creation
```

### Invalid key format
```bash
# Verify keys are hex
ALICE_KEYS=$(docker exec dchat-user1 dchat account create --username test | grep public_key)

# Should be 64 hex chars (0-9, a-f only)
echo "$ALICE_KEYS" | grep -oE '[0-9a-f]{64}' | head -1
```

---

## Expected Test Output Examples

### User Creation Response
```json
{
  "user_id": "user-a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "username": "alice",
  "public_key": "abcd1234ef5678901234567890abcdef1234567890abcdef1234567890abcdef",
  "private_key": "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
  "created_at": "2025-10-29T06:35:24.279315Z",
  "message": "User created successfully"
}
```

### Direct Message Response
```json
{
  "message_id": "msg-12345678-1234-1234-1234-123456789012",
  "status": "sent",
  "timestamp": "2025-10-29T06:35:30.123456Z",
  "on_chain_confirmed": false
}
```

### Get DMs Response
```json
{
  "messages": [
    {
      "message_id": "msg-12345678-1234-1234-1234-123456789012",
      "sender_id": "user-a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "recipient_id": "user-f0e9d8c7-b6a5-4321-8765-fedcba987654",
      "content": "Hello Bob!",
      "timestamp": "2025-10-29T06:35:30.123456Z",
      "status": "sent"
    }
  ]
}
```

---

## Performance Expectations

| Operation | Expected Time | Notes |
|-----------|---------------|-------|
| Create user | <100ms | Instant |
| Get profile | <50ms | Cached |
| Send DM | <100ms | Local only |
| Get messages | <100ms | Local query |
| Create channel | <100ms | No blockchain yet |
| Post to channel | <100ms | Local only |

---

## Test Suite Completion Checklist

### Quick Test (15 minutes)
- [ ] Test 1: User Creation (2 min)
- [ ] Test 2: Direct Messaging (3 min)
- [ ] Test 3: Channel Operations (3 min)
- [ ] Test 4: Database (2 min)
- [ ] Test 5: Blockchain (1 min)
- [ ] Success: All tests passed

### Full Test (20 minutes, per USER_MANAGEMENT_TESTING_GUIDE.md)
- [ ] Phase 1: Solo user creation
- [ ] Phase 2: Profile lookup
- [ ] Phase 3: Cross-node messaging
- [ ] Phase 4: Channel operations
- [ ] Phase 5: Blockchain verification

---

## Success Indicators

### ✅ All Green
```
✓ Users created with unique IDs
✓ Public/private keys 64 hex chars
✓ Messages sent and retrieved
✓ Channels created and posted to
✓ Database entries verified
✓ Blocks produced regularly
✓ Network stable
```

### ⚠️ Issues to Watch
```
⚠ Cross-node messages delayed (expected, network sync pending)
⚠ on_chain_confirmed always false (expected, not wired yet)
⚠ Marketplace/payments not available (future phase)
⚠ Channel storage minimal (placeholder implementation)
```

---

## Next Steps After Testing

1. **If All Tests Pass** ✅
   - Run test suite 2-3 more times for consistency
   - Document results
   - Proceed to marketplace integration

2. **If Issues Found** ⚠️
   - Check logs: `docker logs dchat-user1`
   - Try restarting container: `docker-compose up -d dchat-user1`
   - Review USER_MANAGEMENT_TESTING_GUIDE.md troubleshooting

3. **For Enhancements** 🚀
   - Implement on-chain confirmation wiring
   - Add cross-node synchronization
   - Build marketplace transaction support

---

## Files to Reference

- **Testing Guide**: `USER_MANAGEMENT_TESTING_GUIDE.md` (comprehensive)
- **Status Report**: `USER_MANAGEMENT_COMPLETE_STATUS.md` (detailed)
- **Quick Script**: `test-user-management.sh` (automated checks)
- **This Guide**: `QUICK_START_USER_MANAGEMENT.md` (you are here)

---

**Ready?** Start with Test 1: User Creation in a few seconds! 🚀

