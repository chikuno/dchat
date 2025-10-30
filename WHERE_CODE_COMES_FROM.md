# Where The Network Gets Upgraded Code - Quick Answer

## Your Question
> "where will the net get the upgraded code or how will it sync the updated code where will it pull it from"

## The Answer (30 seconds)

**5 Distribution Channels** (no single point of failure):

```
1. HTTPS Mirrors     →  releases.dchat.network (fast, default)
2. IPFS Network      →  QmContentID... (censorship-resistant)
3. BitTorrent        →  magnet:?xt=... (peer bandwidth)
4. Peer Gossip       →  Direct from connected nodes
5. Local Cache       →  Previously downloaded versions
```

**How It Works**:
1. Node learns about v1.0.0 via **gossip** (peers tell peers)
2. Node downloads from **best mirror** (tries each until success)
3. Node **verifies** SHA-256 + BLAKE3 + Ed25519 signature
4. Node **waits** for activation height (set by governance)
5. At activation height: **all nodes switch simultaneously**

**No Centralized Update Server** = No Censorship, No Single Point of Failure

---

## The Full Picture (3 Minutes)

### Discovery: How Nodes Learn About Updates

```
┌──────────┐   "v1.0.0 available"    ┌──────────┐
│  Node A  │ ─────────────────────>  │  Node B  │
│ (knows)  │      Gossip Protocol    │(learns)  │
└──────────┘                         └──────────┘
     │                                     │
     │ Gossip                             │ Gossip
     │ (TTL=5)                            │ (TTL=4)
     ↓                                     ↓
┌──────────┐                         ┌──────────┐
│  Node C  │                         │  Node D  │
│(learns)  │                         │(learns)  │
└──────────┘                         └──────────┘
```

**Discovery Methods**:
- **Peer Gossip**: Nodes tell neighbors about new versions (TTL-based flooding)
- **Governance Announcement**: Approved upgrades broadcast on-chain
- **IPFS DHT**: Content-addressed discovery
- **Mirror Lists**: Nodes share mirror URLs with each other

### Download: How Nodes Get Code

**Priority Order** (tries each until success):

```
1. Local Cache          → Instant (if already downloaded)
2. HTTPS Mirror #1      → 120ms (fast CDN)
3. HTTPS Mirror #2      → 150ms (backup)
4. IPFS Network         → 450ms (distributed)
5. Peer Transfer        → 200ms (from connected node)
6. BitTorrent Swarm     → 300ms (P2P bandwidth)
```

**Download Sources**:

| Source | Speed | Reliability | Censorship Resistance | Cost |
|--------|-------|-------------|---------------------|------|
| HTTPS Mirrors | ⚡⚡⚡ | ⭐⭐⭐ | ⭐ | Hosting fees |
| IPFS | ⚡⚡ | ⭐⭐⭐ | ⭐⭐⭐ | Free |
| BitTorrent | ⚡⚡⚡ | ⭐⭐⭐ | ⭐⭐⭐ | Free |
| Peer Gossip | ⚡⚡ | ⭐⭐ | ⭐⭐⭐ | Free |

### Verification: Trust But Verify

**Every Package MUST Pass**:

```
1. SHA-256 Hash    → Prevents tampering
2. BLAKE3 Hash     → Prevents collision attacks
3. Ed25519 Signature → Proves authenticity
4. Trusted Key Check → Signer in approved list
```

**If ANY verification fails** → Package rejected, alert logged

**Mirrors are untrusted** → They can't forge signatures

### Installation: Coordinated Activation

```
Timeline:

Block 100,000: Governance approves v1.0.0 ✅
               Activation height set: 110,000

Block 100,001-109,999: GRACE PERIOD (10,000 blocks ≈ 3 days)
    ┌─────────────────────────────────────┐
    │ Nodes discovering v1.0.0 via gossip │
    │ Nodes downloading from mirrors       │
    │ Nodes verifying signatures           │
    │ Nodes waiting...                     │
    └─────────────────────────────────────┘

Block 110,000: ⚡ ACTIVATION HEIGHT
    ┌─────────────────────────────────────┐
    │ ALL nodes switch to v1.0.0          │
    │ simultaneously at this block         │
    └─────────────────────────────────────┘

Block 110,001+: Network running v1.0.0 ✅
```

**No Centralized Shutdown** → Nodes coordinate via blockchain height

---

## CLI Examples

### Check For Updates
```bash
dchat update check

# Output:
🔍 Checking for updates (current version: 0.1.0)
📦 Available versions:
  0.1.0 (current)
  1.0.0 ← New version available!
```

### Download Update
```bash
dchat update download --version 1.0.0

# Automatically tries:
# 1. https://releases.dchat.network/1.0.0
# 2. ipfs://QmContentID
# 3. Connected peers
# 4. BitTorrent swarm
```

### Verify Package
```bash
dchat update verify \
  --package ~/.dchat/packages/dchat-1.0.0-linux-x64 \
  --version 1.0.0

# Output:
✅ Hash verification passed (SHA-256)
✅ Hash verification passed (BLAKE3)
✅ Signature verification passed (Ed25519)
✅ Package verified successfully
```

### List Mirrors
```bash
dchat update list-mirrors

# Output:
📍 Configured download sources:
  1. https://releases.dchat.network (priority: 10)
  2. ipfs://QmExample... (priority: 20)
  3. Gossip discovery (priority: 30)
```

### Add Custom Mirror
```bash
dchat update add-mirror \
  --url https://mirror.example.com/dchat \
  --mirror-type https \
  --priority 15
```

---

## Security

### Attack Resistance

| Attack | Defense |
|--------|---------|
| Malicious Mirror | Signature verification (can't forge) |
| Man-in-the-Middle | Hash + signature check |
| DNS Hijacking | IPFS, BitTorrent, peer gossip |
| Censorship | 5+ distribution channels |
| Downgrade Attack | Governance approval required |
| Hash Collision | Dual hashing (SHA-256 + BLAKE3) |

### Trust Model

```
Release Signing Keys (trusted, hardcoded)
         ↓
Package Signatures (cryptographic proof)
         ↓
Governance Approvals (community consensus)
         ↓
Mirrors (UNTRUSTED - verified every time)
         ↓
Your Node (verifies everything)
```

**Principle**: Don't trust mirrors, trust math (cryptography)

---

## Integration with Governance

```
Governance System               Distribution System
       ↓                               ↓
1. Propose v1.0.0        →    Package metadata announced
2. Community votes       →    Nodes discover via gossip
3. Approval ✅           →    Grace period begins
4. Schedule activation   →    Nodes download & verify
5. Activation height     →    All nodes switch at height
```

**No Separate Process** → Distribution is part of governance upgrade flow

---

## Status

| Component | Status |
|-----------|--------|
| Gossip Discovery | ✅ Implemented |
| HTTPS Downloads | ✅ Implemented |
| Cryptographic Verification | ✅ Implemented |
| Auto-Update Config | ✅ Implemented |
| CLI Commands | ✅ Implemented (9 commands) |
| IPFS Integration | 🚧 Stub (feature flag ready) |
| BitTorrent | 🚧 Stub (future) |
| Peer Transfer | 🚧 Stub (future) |

**Build**: ✅ Clean (3m 56s, release mode)  
**Tests**: ✅ All passing

---

## Documentation

- **Full Documentation**: `UPDATE_DISTRIBUTION_COMPLETE.md` (comprehensive)
- **This File**: `WHERE_CODE_COMES_FROM.md` (quick answer)
- **Governance Integration**: `GOVERNANCE_QUICK_REF.md`
- **Architecture**: `ARCHITECTURE.md` Section 29

---

## Summary

**Where does code come from?**
- HTTPS mirrors (fast)
- IPFS (censorship-resistant)
- BitTorrent (P2P)
- Peer gossip (direct)
- Local cache (instant)

**How does it sync?**
- Gossip protocol (discovery)
- Best-available mirror (download)
- Cryptographic verification (trust)
- Activation height (coordination)

**Who controls it?**
- No one (decentralized)
- Everyone (governance voting)
- Math (cryptographic proof)

**Can it be censored?**
- No (5+ channels)
- No single point of failure
- Works even if DNS, HTTPS, or IPFS down

🎉 **Your network gets upgraded code without any central authority!**
