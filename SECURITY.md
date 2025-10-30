# Security Policy

## Supported Versions

We release patches for security vulnerabilities. Currently supported versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

### Reporting Process

1. **Email**: Send details to security@dchat.example (replace with actual email)
2. **Encrypt**: Use our PGP key for sensitive information (see below)
3. **Include**:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if available)

### Response Timeline

- **Acknowledgment**: Within 24 hours
- **Initial Assessment**: Within 72 hours
- **Status Update**: Weekly until resolved
- **Fix Timeline**: 
  - Critical: 7 days
  - High: 30 days
  - Medium: 90 days
  - Low: Best effort

### Disclosure Policy

We follow **coordinated disclosure**:

1. **Day 0**: Vulnerability reported
2. **Day 1**: Acknowledgment sent
3. **Day 3**: Initial assessment completed
4. **Day 7-90**: Fix developed and tested (based on severity)
5. **Public Disclosure**: After fix is released or 90 days, whichever comes first

### Security Update Process

1. **Patch Development**: Fix is developed in private repository
2. **Testing**: Comprehensive testing including regression tests
3. **Release**: Security patch released with CVE assignment
4. **Announcement**: Public disclosure with details and mitigation
5. **Credit**: Reporter acknowledged (unless anonymity requested)

## Security Best Practices

### For Users

1. **Keep Updated**: Always use the latest version
2. **Verify Downloads**: Check signatures and checksums
3. **Secure Keys**: Protect private keys with strong passphrases
4. **Network Security**: Use trusted network infrastructure
5. **Monitor Logs**: Watch for suspicious activity

### For Developers

1. **Code Review**: All PRs require security review
2. **Dependencies**: Run `cargo audit` before every release
3. **Testing**: Include security tests for all features
4. **Secrets**: Never commit secrets or private keys
5. **Crypto**: Use only approved cryptographic libraries

## Known Vulnerabilities

### Current Status (as of 2025-10-28)

**Critical**: 0  
**High**: 0  
**Medium**: 0  
**Low**: 2 (in unused features, documented in audit report)

See [PHASE7_SPRINT4_SECURITY_AUDIT.md](PHASE7_SPRINT4_SECURITY_AUDIT.md) for details.

## Security Features

### Cryptography

- **Ed25519**: Digital signatures (NIST FIPS 186-5)
- **X25519**: Key exchange
- **ML-KEM-768**: Post-quantum key encapsulation (NIST FIPS 203)
- **Falcon-512**: Post-quantum signatures
- **Noise Protocol**: Transport encryption with forward secrecy
- **BLAKE3/SHA-256**: Cryptographic hashing

### Network Security

- **Encrypted Transport**: All communication encrypted
- **Peer Authentication**: Mutual authentication required
- **Replay Protection**: Nonce-based replay prevention
- **Rate Limiting**: DoS protection mechanisms
- **Input Validation**: Comprehensive validation on all inputs

### Privacy

- **Metadata Protection**: Minimal metadata leakage
- **Onion Routing**: Multi-hop message routing
- **Stealth Addresses**: Contact graph hiding
- **Zero-Knowledge Proofs**: Privacy-preserving verification

### Operational Security

- **Key Zeroization**: Automatic key cleanup on drop
- **Constant-Time Operations**: Timing attack resistance
- **Secure Random**: Hardware-backed RNG where available
- **Memory Safety**: Rust's memory safety guarantees

## Audit History

| Date | Type | Findings | Status |
|------|------|----------|--------|
| 2025-10-28 | Internal (Phase 7 Sprint 4) | 2 low-risk in unused features | Documented |

## Bug Bounty Program

**Status**: Planned for post-launch

We plan to launch a bug bounty program after production release. Details will be announced.

## Security Contacts

- **Email**: security@dchat.example (to be configured)
- **PGP Key**: [To be published]
- **Security Team**: See MAINTAINERS.md

## Acknowledgments

We thank the security research community for responsible disclosure and helping keep dchat secure.

### Hall of Fame

(To be populated with contributors who responsibly disclose vulnerabilities)

## Related Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture and threat model
- [PHASE7_SPRINT4_SECURITY_AUDIT.md](PHASE7_SPRINT4_SECURITY_AUDIT.md) - Latest security audit
- [CONTRIBUTING.md](CONTRIBUTING.md) - Security guidelines for contributors

---

**Last Updated**: October 28, 2025  
**Policy Version**: 1.0
