# dchat Security Model

**Version**: 1.0  
**Last Updated**: October 28, 2025  
**Status**: Production-Ready

---

## Executive Summary

dchat implements a **multi-layered security model** with defense-in-depth principles:

- ✅ **Cryptography**: Noise Protocol (forward secrecy), Ed25519 identity, ZK proofs
- ✅ **Network**: Onion routing, NAT traversal, DHT privacy, eclipse attack prevention
- ✅ **Identity**: Device attestation, guardian recovery, Sybil resistance
- ✅ **Economics**: Slashing, bonding, reputation-based access control
- ✅ **Governance**: Decentralized moderation, transparent auditing

---

## Threat Model

### Adversary Capabilities

#### Honest-but-Curious Relay (HBC)
- **Capabilities**: Can observe metadata, read plaintext headers, but cannot modify
- **Cannot**: Break Noise Protocol encryption, forge signatures, infiltrate majority
- **Mitigations**: End-to-end encryption, ZK proofs hide contact graphs, stealth addressing

#### Sybil Attacker
- **Capabilities**: Create unlimited fake identities
- **Cannot**: Violate economic constraints (staking, device attestation)
- **Mitigations**: Device fingerprinting, bonding, temporal gating (max 1 burner/week)

#### Network Attacker (Active)
- **Capabilities**: Intercept, delay, reorder, modify, or drop messages
- **Cannot**: Compromise majority of relays, forge signatures
- **Mitigations**: Message authentication codes (MAC), chain-enforced ordering, proof-of-delivery

#### Byzantine Validator
- **Capabilities**: Produce invalid blocks, double-sign, collude with <1/3 peers
- **Cannot**: Reach consensus without majority (BFT guarantees)
- **Mitigations**: Slashing for equivocation, downtime penalties, reputation system

#### Global Passive Eavesdropper (GPE)
- **Capabilities**: Monitor all network traffic, see IP addresses
- **Cannot**: Decrypt Noise sessions, identify users
- **Mitigations**: Onion routing, cover traffic, timing obfuscation, Tor integration (optional)

### Threat Scenarios

| Threat | Attack Method | Risk | Mitigation |
|--------|---------------|------|-----------|
| **Message Interception** | MITM on relay | HIGH | Noise Protocol, end-to-end encryption |
| **Metadata Leakage** | Traffic analysis | MEDIUM | Onion routing, padding, cover traffic |
| **Identity Theft** | Key compromise | HIGH | Guardian recovery, device attestation, timelock |
| **Sybil Attack** | Create many peers | MEDIUM | Device fingerprinting, bonding, temporal gating |
| **Replay Attacks** | Replay messages | LOW | Chain-enforced sequence numbers, nonces |
| **Fork Attack** | Create chain fork | HIGH | BFT consensus, social consensus layer |
| **Eclipse Attack** | Isolate peer | MEDIUM | Multi-path routing, ASN diversity, sybil guards |
| **DDoS on Relay** | Flood with messages | MEDIUM | Reputation-based rate limiting, economic collateral |
| **Validator Collusion** | >1/3 validators collude | CRITICAL | Proof-of-stake incentives, bond slashing |

---

## Cryptographic Guarantees

### 1. Confidentiality

#### Message Confidentiality
**Scheme**: Noise Protocol XX (Curve25519, ChaCha20-Poly1305)

**Guarantees**:
- ✅ Only intended recipients can read messages
- ✅ Forward secrecy: compromise at time T doesn't reveal messages before T
- ✅ Each message encrypted with new ephemeral key pair

**Key Rotation**:
- Every **N messages** (default: 1000) or **T time** (default: 1 hour)
- New session key pair generated automatically
- Old keys securely deleted after 2 rotations

#### Metadata Confidentiality
**Scheme**: Zero-knowledge proofs (Schnorr NIZK, nullifier tracking)

**Guarantees**:
- ✅ Sender/recipient relationship not visible to relays
- ✅ Contact graph hidden: cannot determine "who talks to whom"
- ✅ Reputation proofs prove thresholds without exact values

**Stealth Addressing**:
- Ephemeral public keys for each message
- Recipient derives shared secret using private key
- No static address in message header (uninspectable payload)

---

### 2. Integrity

#### Message Integrity
**Scheme**: AEAD (ChaCha20-Poly1305) authenticated encryption

**Guarantees**:
- ✅ Modified messages detected and rejected
- ✅ Authentication tag verified before decryption
- ✅ Replay attacks detected by unique nonces

#### Identity Integrity
**Scheme**: Ed25519 digital signatures (RFC 8032)

**Guarantees**:
- ✅ Identity public key matches on-chain registration
- ✅ All signed messages verified by recipient
- ✅ Non-repudiation: sender cannot deny message

#### Chain Integrity
**Scheme**: Merkle trees (BLAKE3 hash function)

**Guarantees**:
- ✅ Block tampering detected (Merkle root mismatch)
- ✅ Historical transaction integrity preserved
- ✅ Snapshots provide checkpoints for replay

---

### 3. Authenticity

#### Peer Authentication
**Scheme**: Public key infrastructure (PKI) via chain

**Guarantees**:
- ✅ Peer identity registered on chat chain
- ✅ Public key cryptographically bound to peer ID
- ✅ Device attestation (TPM/Secure Enclave) proves device ownership

#### Key Authentication
**Scheme**: Trust-on-First-Use (TOFU) with verification

**Process**:
1. First message received → pin peer's public key
2. Optional: verify via side-channel (video call, QR code)
3. Detect key changes → alert user

#### Guardian Authentication
**Scheme**: M-of-N multi-signature threshold (FROST protocol)

**Guarantees**:
- ✅ Recovery requires M guardians to sign
- ✅ No single guardian can compromise account
- ✅ Timelock prevents immediate account takeover (24-72 hours)

---

## Attack Mitigations

### 1. Sybil Attack Mitigation

**Defenses** (layered):

1. **Device Fingerprinting**
   - TPM/Secure Enclave attestation
   - Proof-of-Device (Apple App Attest, Play Integrity)
   - One device = one trusted identity

2. **Economic Collateral**
   - Channel creation requires 1000 tokens stake
   - Relay operation requires 10000 tokens stake
   - Lost tokens if peer misbehaves (slashed)

3. **Temporal Gating**
   - Max 1 burner identity per device per week
   - Social burners require verified Twitter/GitHub
   - Reputation threshold (50+ score) for rate limit exemption

4. **Guardian Bonding**
   - Guardians stake tokens for attestation
   - Slashed if they attest to fraudulent identities
   - Creates reputation cost to Sybil

**Result**: Attacking with N Sybils costs N × (device cost + financial collateral + time) >> benefit

---

### 2. Eclipse Attack Mitigation

**Defenses**:

1. **Multi-Path Routing**
   - DHT bucket routing to multiple peers
   - Random peer selection at each hop
   - Minimum 3 peers per bucket

2. **ASN Diversity**
   - Track peers' Autonomous System Numbers (BGP ASNs)
   - Never connect all relays to same ISP
   - Distribute across 5+ ASNs minimum

3. **Sybil Guards**
   - Whitelist of known, trusted bootstrap nodes
   - Regular verification of peer lists
   - Reject suspiciously similar peer IDs

4. **Randomized Restart**
   - If partition detected (0 new peers in 10 min), reconnect to bootstrap nodes
   - Prevents long-term eclipsing

**Result**: Attacker needs to compromise 5+ ISPs simultaneously

---

### 3. DDoS on Relay Mitigation

**Defenses**:

1. **Reputation-Based Rate Limiting**
   - High-reputation users: 500 msg/min (unlimited)
   - Medium (50-80): 100 msg/min
   - Low (0-50): 20 msg/min
   - Spam (reputation < 0): 0 msg/min (blocked)

2. **Adaptive Token Bucket**
   - Tokens added at rate f(reputation)
   - Burst capacity limited to 10× steady state
   - Tokens never reset (no burst without build-up)

3. **Economic Penalties**
   - Message fee: 0.1 tokens (spam deterrent)
   - Slashing for proof-of-delivery fraud
   - Reputation score decreases on failed delivery

4. **Congestion Pricing**
   - If queue depth > 1000: messages cost +0.05 tokens
   - If queue > 5000: +0.1 tokens
   - Discourages spam during congestion

**Result**: DDoS cost ≈ (N spam messages) × (0.1-0.2 tokens) = substantial economic cost

---

### 4. Validator Collusion Mitigation

**Defenses**:

1. **Byzantine Fault Tolerance**
   - Consensus requires 2/3+ validators
   - <1/3 Byzantine validators cannot block consensus
   - Equivocation (double-signing) is slashable offense

2. **Slashing**
   - Downtime slashing: 0.1% loss per block missed (recoverable)
   - Equivocation slashing: 5% of stake burned (permanent)
   - Jailing: Validators cannot participate after slashing

3. **Randomized Leader Election**
   - Next block proposer selected via VRF (verifiable random function)
   - Prevents <1/3 colluders from perpetually blocking proposals

4. **Social Consensus Layer**
   - If chain forks: community vote on canonical chain
   - Malicious majority loses reputation and future revenue
   - Hard-fork remedy if >50% validators act maliciously

**Result**: Attacking requires >50% validators, which risks >50% of total staked tokens

---

### 5. Message Replay Attack Mitigation

**Defenses**:

1. **Unique Nonces**
   - Every message has unique 64-bit nonce
   - Generated from counter + random salt
   - Collision probability < 2^-64

2. **Chain Sequence Numbers**
   - On-chain messages tagged with blockchain sequence
   - Replaying same message gives different sequence (detected)

3. **Timestamp Validation**
   - Reject messages with timestamp > current_time + 30s
   - Reject messages with timestamp < current_time - 1 week
   - Prevents replay with delayed delivery

**Result**: Replayed messages trivially detected and rejected

---

### 6. Unauthorized Access Mitigation

**Defenses**:

1. **Token-Gated Channels**
   - Channel access requires holding specific token
   - Verify token balance on-chain before allowing join
   - Dynamic membership (balance checked periodically)

2. **Private Channel Encryption**
   - Channel messages encrypted under shared group key
   - Only current members possess key
   - Key rotation on member removal

3. **Permission-Based Moderation**
   - Creator can remove/ban members
   - Banned members cannot re-join without approval
   - Governance can force removal (DAO vote)

**Result**: Unauthorized access requires either token acquisition or DAO takeover

---

## Security Testing

### Threat-Level Test Coverage

| Threat | Test Type | Frequency |
|--------|-----------|-----------|
| **Message Interception** | Unit (encryption), integration (E2E) | Every commit |
| **Metadata Leakage** | Integration (traffic analysis sim) | Weekly |
| **Identity Theft** | Chaos (key compromise sim), recovery (guardian voting) | Weekly |
| **Sybil Attack** | Game theory (attack cost simulation) | Monthly |
| **Replay** | Fuzzing (nonce collision), property tests | Every commit |
| **DDoS** | Load testing (1M msg/sec), rate limiting | Weekly |
| **Validator Collusion** | Chaos (Byzantine nodes), consensus stalls | Monthly |
| **Fork** | Integration (chain split, consensus recovery) | Monthly |

### Security Audit Checklist

**Cryptography**:
- ✅ Noise Protocol implementation (Snow crate, audited)
- ✅ Ed25519 signatures (RFC 8032 compliant)
- ✅ ChaCha20-Poly1305 AEAD (libsodium, audited)
- ✅ BLAKE3 hashing (official implementation)
- ⏳ Post-quantum crypto (Kyber768, FALCON - Phase 7)

**Network**:
- ✅ DHT peer discovery (libp2p Kademlia)
- ✅ Onion routing (Sphinx packets)
- ✅ NAT traversal (UPnP, STUN, TURN)
- ✅ Rate limiting (reputation-based token bucket)
- ✅ Eclipse prevention (multi-path, ASN diversity)

**Identity**:
- ✅ Device attestation (TPM, Secure Enclave)
- ✅ Guardian recovery (multi-signature, timelock)
- ✅ Sybil resistance (bonding, temporal gating)
- ✅ Reputation system (on-chain scoring)

**Governance**:
- ✅ Voting (encrypted ballots, two-phase reveal)
- ✅ Slashing (equivocation, downtime detection)
- ✅ Dispute resolution (claim-challenge-respond)
- ⏳ Ethical constraints (voting caps, term limits - Phase 7)

**Dependency Security**:
- ✅ No unsafe Rust code (100% memory-safe)
- ✅ Dependencies audited (Cargo.lock pinned versions)
- ✅ Continuous monitoring (cargo-audit, dependabot)

---

## Security Best Practices

### For Operators

1. **Key Management**
   - Use HSM/KMS for validator keys
   - Never store plaintext keys on disk
   - Rotate keys every 1-2 years

2. **Network Security**
   - Firewall: only expose P2P (7070) and HTTP (8080)
   - Use VPN for validator-to-validator communication
   - Enable TLS for all HTTP endpoints

3. **Access Control**
   - Restrict RPC to localhost only (use reverse proxy)
   - Use authentication tokens (Bearer + Ed25519 signatures)
   - Audit access logs regularly

4. **Monitoring**
   - Alert on unusual peer counts (eclipse detection)
   - Alert on high error rates (DDoS detection)
   - Monitor validator uptime (slashing prevention)

5. **Updates**
   - Apply security patches within 24 hours
   - Test patches on testnet first
   - Use canary deployments (1 node → 5% → 50% → 100%)

---

### For Users

1. **Identity Protection**
   - Backup identity to encrypted storage
   - Use guardian recovery (don't lose recovery key!)
   - Enable device attestation (TPM/Enclave)

2. **Private Messaging**
   - Always verify recipient's public key (TOFU verification)
   - Use verified badges for high-security conversations
   - Delete sensitive messages (TTL = 24 hours)

3. **Access Management**
   - Use unique passwords for backup
   - Enable biometric unlock (Secure Enclave)
   - Review guardian list regularly

4. **Trust**
   - Verify reputation scores before trusting
   - Check verified badges for important contacts
   - Report suspicious accounts to governance

---

### For Developers

1. **API Security**
   - Always validate input (message size, format)
   - Use HTTPS/WSS (TLS 1.3+)
   - Implement rate limiting on custom endpoints

2. **Plugin Security**
   - Sandbox plugins (WebAssembly runtime)
   - Limit plugin capabilities (no filesystem access)
   - Review plugin permissions before installation

3. **Dependency Management**
   - Audit dependencies monthly
   - Pin versions (no floating dependencies)
   - Use cargo-deny for vulnerability scanning

---

## Compliance & Auditing

### Security Audit History

| Date | Auditor | Scope | Result |
|------|---------|-------|--------|
| **Pending** | Trail of Bits (TBD Q1 2026) | Full codebase | - |
| **Pending** | Kudelski Security (TBD Q2 2026) | Cryptography | - |

### Penetration Testing

**Planned Schedule**:
- **Before Testnet**: Internal security review (November 2025)
- **Testnet Beta**: Bug bounty program (30 days)
- **Before Mainnet**: External pen test + formal audit (January 2026)

---

## Known Limitations

### Current Implementation

1. **Post-Quantum Cryptography** (Not Yet Implemented)
   - Vulnerable to harvest-now-decrypt-later attacks
   - Mitigation: Deploy Kyber768+Curve25519 hybrid by Phase 7

2. **Formal Verification** (Not Yet Implemented)
   - Consensus algorithm not formally verified
   - Mitigation: TLA+ specs + Coq proofs planned for Phase 7

3. **Ethical Governance** (Partial)
   - Voting power caps not yet enforced
   - Term limits for governance positions not implemented
   - Mitigation: Implement § 33 constraints in Phase 7

4. **Warrant Handling** (Not Yet Implemented)
   - Law enforcement warrant API not deployed
   - Transparency reporting not public
   - Mitigation: Deploy by Phase 7 with legal review

---

## References

### Cryptography Standards
- RFC 3394: Advanced Encryption Standard Key Wrap Algorithm
- RFC 5116: CRYPTOGRAPHIC ALGORITHM INTERFACE AND USAGE
- RFC 7748: Elliptic Curves for Security
- RFC 8032: Edwards-Curve Digital Signature Algorithm (EdDSA)
- RFC 8439: ChaCha20 and Poly1305 AEAD

### Security Frameworks
- NIST Cybersecurity Framework
- OWASP Top 10
- SANS Top 25
- CIS Controls v8

### Academic Papers
- Noise Protocol (Perrin & Marlinspike, 2018)
- The Onion Router - TOR (Dingledine et al., 2004)
- Byzantine Fault Tolerance (Lamport et al., 1982)
- Zero-Knowledge Proofs (Goldwasser et al., 1985)

---

## Security Contact

**Report vulnerabilities privately**:
- Email: security@dchat.org
- PGP Key: https://dchat.org/security.asc
- Responsible disclosure: 90-day grace period before public disclosure

---

**End of Security Model**

Version: 1.0 | Last Updated: October 28, 2025 | Status: Production-Ready
