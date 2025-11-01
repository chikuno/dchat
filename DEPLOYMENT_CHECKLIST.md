# Deployment Checklist - dchat Testnet Urgent Fixes

**Date:** November 1, 2025  
**Server:** 4.221.211.71  
**User:** azureuser

---

## Pre-Deployment Checks

- [ ] SSH key available at `C:\Users\USER\Downloads\anacreon.pem`
- [ ] SSH access verified: `ssh -i key azureuser@4.221.211.71`
- [ ] Docker installed locally for building image
- [ ] cargo build working (no compilation errors)
- [ ] Deployment script executable: `chmod +x urgent-fix-deploy.sh`
- [ ] Test script executable: `chmod +x test-urgent-fix.sh`

---

## Deployment Steps

### Phase 1: Pre-Deployment Backup (5 min)
- [ ] SSH to server
- [ ] Check current container status: `sudo docker ps`
- [ ] Save current logs (optional):
  ```bash
  sudo docker logs dchat-validator1-test > validator1-pre-deploy.log
  sudo docker logs dchat-user1-test > user1-pre-deploy.log
  ```
- [ ] Note current block height from validator logs

### Phase 2: Execute Deployment (10-15 min)
- [ ] Run deployment script: `./urgent-fix-deploy.sh`
- [ ] Monitor output for errors
- [ ] Verify build completes successfully
- [ ] Verify Docker image builds successfully
- [ ] Verify upload completes (may take 5-10 min)
- [ ] Verify server deployment starts

**Expected Output:**
```
🚀 dchat Urgent Fix Deployment
================================
📦 Step 1: Building Rust project...
✅ Build successful
🐳 Step 2: Building Docker image...
✅ Docker image built
💾 Step 3: Saving Docker image...
✅ Image saved to dchat-latest.tar.gz
⬆️  Step 4: Uploading to server...
✅ Files uploaded
🚀 Step 5: Deploying on server...
✅ Deployment complete!
```

### Phase 3: Verification (5-10 min)

#### Step 1: Container Status
- [ ] SSH to server
- [ ] Check all containers running:
  ```bash
  sudo docker ps --format 'table {{.Names}}\t{{.Status}}' | grep dchat
  ```
- [ ] **Expected:** 14 containers running, all showing `(healthy)` status
- [ ] **If unhealthy:** Wait 30s and check again (containers need startup time)

#### Step 2: Health Endpoint Tests
- [ ] Test validator health:
  ```bash
  curl http://localhost:8080/health
  # Should return JSON status, NOT connection refused
  ```
- [ ] Test from Docker container directly:
  ```bash
  sudo docker exec dchat-validator1-test curl http://localhost:8080/health
  ```
- [ ] **Expected:** `{"status":"ok"}` or similar health response

#### Step 3: Validator Logs Check
- [ ] Check validator1: `sudo docker logs dchat-validator1-test --tail 50`
- [ ] Check validator2: `sudo docker logs dchat-validator2-test --tail 50`
- [ ] **Look for:** Block production continuing (block numbers incrementing)
- [ ] **Should NOT see:** Crash, panic, or fatal errors
- [ ] Note new block height and compare to pre-deployment

#### Step 4: Relay Logs Check
- [ ] Check relay1: `sudo docker logs dchat-relay1-test --tail 50`
- [ ] Check relay2: `sudo docker logs dchat-relay2-test --tail 50`
- [ ] **Look for:** Message forwarding stats, bandwidth stats
- [ ] **Should NOT see:** Connection failures, errors

#### Step 5: User Node Logs Check (CRITICAL)
- [ ] Check user1: `sudo docker logs dchat-user1-test --tail 100`
- [ ] Check user2: `sudo docker logs dchat-user2-test --tail 100`
- [ ] Check user3: `sudo docker logs dchat-user3-test --tail 100`

**Look for these SUCCESS indicators:**
```
✓ Network initialized (peer_id: ...)
✓ Subscribed to #global channel
Waiting 30s for gossipsub subscription exchange and mesh formation...
📊 Gossipsub mesh status: X peers in #global (X should increase over time)
✓ Subscription exchange complete - X mesh peers for #global (X > 0)
📊 Current mesh status: X peers before publishing (X > 0)
📤 Sent test message #1
📤 Sent test message #2
📤 Sent test message #3
```

**Should NOT see:**
```
❌ InsufficientPeers
❌ Failed to publish message
ERROR dchat: Network error: Publish failed
```

- [ ] **CRITICAL CHECK:** User nodes successfully publishing messages
- [ ] **CRITICAL CHECK:** No `InsufficientPeers` errors in user logs
- [ ] **CRITICAL CHECK:** Mesh peer count > 0 before publishing

### Phase 4: Automated Testing (5 min)
- [ ] Run test script: `./test-urgent-fix.sh`
- [ ] Review test results
- [ ] All tests passing

---

## Success Criteria

### Health Checks ✅
- [ ] 14/14 containers showing `(healthy)` status
- [ ] Health endpoints responding with 200 OK
- [ ] No connection refused errors

### Validators ✅
- [ ] Block production continuing
- [ ] Block height incrementing
- [ ] No errors in logs

### Relays ✅
- [ ] Message forwarding active
- [ ] Bandwidth stats present
- [ ] No connection errors

### User Nodes ✅ (MOST IMPORTANT)
- [ ] Gossipsub mesh forming (peer count > 0)
- [ ] Messages publishing successfully
- [ ] NO `InsufficientPeers` errors
- [ ] Test messages #1, #2, #3 all sent

---

## Troubleshooting

### Issue: Containers showing unhealthy after 60s
**Solution:**
```bash
# Check if health server is running
sudo docker exec dchat-validator1-test curl http://localhost:8080/health

# Check container logs for errors
sudo docker logs dchat-validator1-test --tail 100

# Restart specific container if needed
sudo docker restart dchat-validator1-test
```

### Issue: User nodes still showing InsufficientPeers
**Solution:**
```bash
# Check if gossipsub mesh formed
sudo docker logs dchat-user1-test --tail 200 | grep "mesh status"

# Check peer connections
sudo docker logs dchat-user1-test --tail 200 | grep "Peer connected"

# May need to wait longer (up to 60s total for mesh formation)
# Check again after 30s
```

### Issue: Validators stopped producing blocks
**Solution:**
```bash
# Check validator logs for errors
sudo docker logs dchat-validator1-test --tail 100
sudo docker logs dchat-validator2-test --tail 100

# Check consensus logs
sudo docker logs dchat-validator1-test | grep "consensus"

# May need to restart all validators in sequence
sudo docker restart dchat-validator1-test
# Wait 10s
sudo docker restart dchat-validator2-test
# etc.
```

### Issue: Build fails locally
**Solution:**
```bash
# Clean and rebuild
cargo clean
cargo build --release

# Check for compilation errors
# Fix any errors before deploying
```

---

## Post-Deployment Monitoring (First 1 Hour)

### 15 Minutes After Deployment
- [ ] Check all container health status
- [ ] Check validator block height progressing
- [ ] Check user nodes still publishing successfully
- [ ] No new errors in any logs

### 30 Minutes After Deployment
- [ ] Verify sustained message publishing (no InsufficientPeers)
- [ ] Check relay bandwidth/message stats
- [ ] Verify gossipsub mesh stable (peer counts not dropping to 0)
- [ ] Check Prometheus metrics (if accessible)

### 60 Minutes After Deployment
- [ ] Final health check on all containers
- [ ] Confirm sustained operations
- [ ] Document any anomalies
- [ ] Mark deployment as successful

---

## Rollback Procedure (If Needed)

### Quick Rollback
```bash
ssh -i "C:\Users\USER\Downloads\anacreon.pem" azureuser@4.221.211.71
cd ~/chain/dchat
sudo docker-compose -f docker-compose-testnet.yml down
# Restore old image if backed up
sudo docker load < backup-image.tar  # If available
sudo docker-compose -f docker-compose-testnet.yml up -d
```

**When to rollback:**
- Validators stop producing blocks for >5 minutes
- Critical errors appearing in logs
- User nodes completely unable to publish after 60s
- System instability

---

## Sign-Off

### Deployment Sign-Off
- [ ] All pre-deployment checks passed
- [ ] Deployment completed without errors
- [ ] All verification checks passed
- [ ] Success criteria met
- [ ] No critical errors in logs
- [ ] Deployment marked as **SUCCESSFUL**

**Deployed By:** _________________  
**Date/Time:** _________________  
**Status:** ☐ SUCCESS  ☐ PARTIAL  ☐ ROLLBACK

### Notes:
_____________________________________________________________________________
_____________________________________________________________________________
_____________________________________________________________________________

---

## Quick Commands Reference

```bash
# SSH to server
ssh -i "C:\Users\USER\Downloads\anacreon.pem" azureuser@4.221.211.71

# Check all container status
sudo docker ps --format 'table {{.Names}}\t{{.Status}}' | grep dchat

# Check specific logs
sudo docker logs dchat-validator1-test --tail 50
sudo docker logs dchat-relay1-test --tail 50
sudo docker logs dchat-user1-test --tail 100

# Test health endpoint
curl http://localhost:8080/health

# Restart specific container
sudo docker restart dchat-validator1-test

# Full stack restart
cd ~/chain/dchat
sudo docker-compose -f docker-compose-testnet.yml restart

# View Prometheus
http://4.221.211.71:9095

# View Grafana
http://4.221.211.71:3000
```
