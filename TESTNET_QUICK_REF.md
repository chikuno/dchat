# dchat Testnet - Quick Reference Card

## 🚀 Quick Commands

### Start Full Testnet (14 nodes + monitoring)
```powershell
.\scripts\testnet-message-propagation.ps1 -Action start
```

### Check All Nodes Healthy
```powershell
.\scripts\testnet-message-propagation.ps1 -Action health
```

### View Network Status (connectivity, height, peers)
```powershell
.\scripts\testnet-message-propagation.ps1 -Action status
```

### Send Test Message (user1 → user2)
```powershell
.\scripts\testnet-message-propagation.ps1 -Action send-message `
    -FromUser user1 -ToUser user2 -Message "Hello!"
```

### Collect All Logs
```powershell
.\scripts\testnet-message-propagation.ps1 -Action logs
```

### Stop Testnet
```powershell
.\scripts\testnet-message-propagation.ps1 -Action stop
```

---

## 📊 Monitoring URLs

| Component | URL | Login |
|-----------|-----|-------|
| Prometheus | http://localhost:9090 | None |
| Grafana | http://localhost:3000 | admin/admin |
| Jaeger | http://localhost:16686 | None |

---

## 🔍 Health Checks (Manual)

### Check Validator Consensus
```powershell
curl -s http://localhost:7071/status | ConvertFrom-Json | Select height, peers
```

### Check Relay Connectivity
```powershell
curl -s http://localhost:7081/health
```

### Check User Node Status
```powershell
curl -s http://localhost:7111/info | ConvertFrom-Json
```

### View All Running Containers
```powershell
docker ps | Select-String dchat
```

### View Validator Logs
```powershell
docker logs dchat-validator1 --tail=50 -f
```

### View Relay Logs
```powershell
docker logs dchat-relay1 --tail=50 -f
```

### View User Logs
```powershell
docker logs dchat-user1 --tail=50 -f
```

---

## 📝 Test Scenarios

### Scenario 1: Same-Relay Message
```powershell
# Both users connected to relay1,2,3 - should be <500ms
.\scripts\testnet-message-propagation.ps1 -Action send-message `
    -FromUser user1 -ToUser user2 -Message "Same relay test"
```

### Scenario 2: Cross-Relay Message
```powershell
# user1 on relay1, user3 on relay6 - should be <2s
.\scripts\testnet-message-propagation.ps1 -Action send-message `
    -FromUser user1 -ToUser user3 -Message "Cross relay test"
```

### Scenario 3: Rapid Messages
```powershell
# Send 10 messages rapidly (test queue handling)
for ($i=1; $i -le 10; $i++) {
    .\scripts\testnet-message-propagation.ps1 -Action send-message `
        -FromUser user1 -ToUser user2 -Message "Message $i"
    Start-Sleep -Milliseconds 100
}
```

### Scenario 4: Kill and Recovery
```powershell
# Kill relay1 and watch network recover
docker stop dchat-relay1

# Messages should reroute through relay2/3
Start-Sleep -Seconds 2

# Restart relay1
docker start dchat-relay1
```

---

## 🛠️ Troubleshooting Quick Links

| Problem | Check Command |
|---------|---------------|
| Node won't start | `docker logs dchat-validator1` |
| Can't send message | `curl -s http://localhost:7081/health` |
| High latency | `docker stats dchat-relay1` (CPU/Memory) |
| Consensus stalled | `curl -s http://localhost:7071/status` |
| Message not delivered | `docker logs dchat-relay1 \| grep -i queue` |

---

## 📈 Performance Expectations

| Metric | Expected |
|--------|----------|
| Startup Time | 3-5 min (first) / 30-45s (subsequent) |
| Message Latency | 200-500ms (same relay) / 1-2s (cross-relay) |
| Block Time | 2 seconds |
| Node CPU | <15% per validator, <10% per relay |
| Node Memory | 400MB validators, 200MB relays, 100MB users |

---

## 🎯 Architecture Overview

```
4 Validators (Consensus BFT)
         ↓
7 Relays (Message Delivery)
         ↓
3 Users (Clients)
```

**Key Points:**
- ✅ Validators maintain blockchain (2/3 quorum)
- ✅ Relays route messages (no trusted consensus needed)
- ✅ Users stay encrypted end-to-end
- ✅ Messages ordered by validators
- ✅ Delivery proofs create incentives

---

## 📋 Node Port Reference

| Node Type | Instances | P2P Ports | RPC Ports | Metrics |
|-----------|-----------|-----------|-----------|---------|
| Validator | 4 | 7070-7076 | 7071-7077 | 9090-9093 |
| Relay | 7 | 7080-7092 | 7081-7093 | 9100-9106 |
| User | 3 | 7110-7114 | 7111-7115 | 9110-9112 |

---

## 🔐 Security Notes

- All messages encrypted end-to-end
- Validators use Ed25519 signatures
- Relays cannot see plaintext messages
- Each user has unique keypair
- No single point of failure (BFT consensus)

---

## 📚 Further Reading

- Full Guide: `TESTNET_GUIDE.md`
- Architecture: `ARCHITECTURE.md`
- Security Model: `SECURITY.md`
- API Spec: `API_SPECIFICATION.md`

---

## ⚡ Pro Tips

1. **Parallel Testing**: Run multiple message tests simultaneously
2. **Monitor Tab**: Keep Grafana open while testing
3. **Log Grep**: Use `docker logs <container> | grep keyword` to find issues
4. **Volume Cleanup**: `docker volume prune -f` to reset persistent storage
5. **Fresh Start**: Always `stop` before `start` to avoid conflicts

---

**Last Updated**: Session 3 - Testnet Infrastructure  
**Status**: ✅ Production Ready for Testing
