# Contributing to dchat

Thank you for your interest in contributing to dchat! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Security Guidelines](#security-guidelines)
- [Testing Requirements](#testing-requirements)
- [Code Review Process](#code-review-process)
- [Style Guide](#style-guide)

## Code of Conduct

### Our Standards

- **Be Respectful**: Treat everyone with respect and professionalism
- **Be Constructive**: Provide helpful feedback and constructive criticism
- **Be Inclusive**: Welcome contributors of all backgrounds and skill levels
- **Be Collaborative**: Work together towards common goals

### Unacceptable Behavior

- Harassment, discrimination, or personal attacks
- Trolling, insulting, or derogatory comments
- Publishing others' private information
- Any conduct that creates a hostile environment

## Getting Started

### Prerequisites

- **Rust**: 1.70 or later (stable)
- **Cargo**: Latest version
- **Git**: For version control
- **Platform**: Linux, macOS, or Windows

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/dchat/dchat.git
cd dchat

# Install dependencies
cargo build

# Run tests
cargo test

# Run security checks
cargo audit
cargo clippy -- -D warnings
```

## Development Setup

### Required Tools

```bash
# Install development tools
cargo install cargo-audit
cargo install cargo-deny
cargo install cargo-watch
cargo install cargo-llvm-cov

# Optional but recommended
cargo install cargo-edit
cargo install cargo-outdated
```

### IDE Setup

**VS Code** (recommended):
- Install "rust-analyzer" extension
- Install "Even Better TOML" extension
- Install "CodeLLDB" for debugging

**Settings** (.vscode/settings.json):
```json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.cargo.features": "all"
}
```

## Making Changes

### Branch Naming

- `feature/` - New features
- `fix/` - Bug fixes
- `security/` - Security fixes
- `docs/` - Documentation updates
- `refactor/` - Code refactoring
- `test/` - Test additions/improvements

Example: `feature/rate-limiting`

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `security`: Security fix
- `docs`: Documentation
- `test`: Tests
- `refactor`: Refactoring
- `perf`: Performance improvement
- `chore`: Maintenance

**Examples**:
```
feat(network): implement rate limiting for peer connections

security(crypto): migrate to ML-KEM-768 for post-quantum readiness

fix(messaging): resolve message ordering issue in DHT routing
```

### Pull Request Process

1. **Create Feature Branch**
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make Changes**
   - Write code following style guide
   - Add tests for new functionality
   - Update documentation
   - Run local tests and lints

3. **Commit Changes**
   ```bash
   git add .
   git commit -m "feat(scope): description"
   ```

4. **Push Branch**
   ```bash
   git push origin feature/my-feature
   ```

5. **Create Pull Request**
   - Fill out PR template completely
   - Link related issues
   - Add screenshots/examples if applicable
   - Request review from maintainers

6. **Address Review Feedback**
   - Make requested changes
   - Push updates to same branch
   - Re-request review

## Security Guidelines

### Critical Rules

1. **Never Commit Secrets**
   - No private keys, passwords, or API keys
   - Use environment variables for sensitive data
   - Check with `git log -p` before pushing

2. **Cryptographic Code**
   - Only use approved cryptographic libraries
   - No custom crypto implementations
   - Consult security team for crypto changes
   - See [SECURITY.md](SECURITY.md) for approved libraries

3. **Input Validation**
   - Validate all external inputs
   - Sanitize user-provided data
   - Enforce size limits on messages/payloads
   - Use type-safe parsing

4. **Error Handling**
   - Never expose internal errors to users
   - No sensitive data in error messages
   - Use proper error types (Result<T, E>)
   - Log errors securely

5. **Dependencies**
   - Run `cargo audit` before adding dependencies
   - Check license compatibility
   - Avoid unmaintained crates
   - Document why each dependency is needed

### Security Checklist

Before submitting security-related PRs:

- [ ] Run `cargo audit` with no high/critical vulnerabilities
- [ ] Run `cargo clippy` with security lints
- [ ] No `unsafe` code without documentation and review
- [ ] All cryptographic operations tested
- [ ] No secrets in code or commits
- [ ] Input validation comprehensive
- [ ] Error handling doesn't leak information
- [ ] Documentation updated with security considerations

### Reporting Security Issues

**Do NOT open public issues for security vulnerabilities.**

See [SECURITY.md](SECURITY.md) for reporting process.

## Testing Requirements

### Test Coverage

- **Unit Tests**: Required for all new code
- **Integration Tests**: Required for component interactions
- **Security Tests**: Required for security-critical code
- **Performance Tests**: Required for performance-sensitive code

### Running Tests

```bash
# All tests
cargo test

# Specific package
cargo test --package dchat-crypto

# With output
cargo test -- --nocapture

# Coverage report
cargo llvm-cov --all-features --workspace
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        // Arrange
        let input = setup_test_data();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }
}
```

### Property Testing

For security-critical code, use property-based testing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_property(input in any::<Vec<u8>>()) {
        // Property that should hold for all inputs
        prop_assert!(property_holds(input));
    }
}
```

## Code Review Process

### Review Criteria

Reviewers check for:

1. **Correctness**: Does the code work as intended?
2. **Security**: Are there security implications?
3. **Performance**: Is performance acceptable?
4. **Tests**: Is test coverage adequate?
5. **Documentation**: Is the code well-documented?
6. **Style**: Does it follow style guidelines?

### Review Timeline

- **Initial Review**: Within 48 hours
- **Follow-up Reviews**: Within 24 hours
- **Approval Required**: 2 approvals for main branch

### Addressing Feedback

- Respond to all review comments
- Mark conversations as resolved after addressing
- Ask questions if feedback is unclear
- Push updates as new commits (don't force-push during review)

## Style Guide

### Rust Code Style

Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

```rust
// Use rustfmt for automatic formatting
cargo fmt

// Public items need documentation
/// Brief description.
///
/// # Examples
///
/// ```
/// use dchat::Module;
/// let instance = Module::new();
/// ```
pub struct Module {
    // Private fields
    field: Type,
}

impl Module {
    /// Constructor documentation.
    pub fn new() -> Self {
        Self { field: Default::default() }
    }
}

// Constants use SCREAMING_SNAKE_CASE
const MAX_MESSAGE_SIZE: usize = 1024;

// Modules and functions use snake_case
mod message_queue {
    pub fn send_message() { }
}
```

### Documentation

- All public APIs must have documentation
- Include examples for non-trivial functions
- Document panics, errors, and safety invariants
- Use proper markdown formatting

### Error Handling

```rust
// Use Result for fallible operations
pub fn parse_message(data: &[u8]) -> Result<Message, Error> {
    // Prefer early returns for error cases
    if data.is_empty() {
        return Err(Error::empty_input());
    }
    
    // Use ? operator for propagation
    let header = parse_header(data)?;
    let body = parse_body(&data[header.len()..])?;
    
    Ok(Message { header, body })
}

// Use custom error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Cryptographic error")]
    CryptoError(#[from] CryptoError),
}
```

## Areas for Contribution

### Good First Issues

Look for issues tagged with:
- `good-first-issue`
- `help-wanted`
- `documentation`

### High-Priority Areas

- Rate limiting and DoS protection
- Fuzz testing expansion
- Documentation improvements
- Performance optimization
- Cross-platform compatibility

### What We're Looking For

- Bug fixes with tests
- Performance improvements with benchmarks
- Documentation improvements
- Test coverage improvements
- Security enhancements

## Community

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **Pull Requests**: Code contributions
- **Discussions**: General questions and ideas

### Getting Help

- Check existing issues and documentation first
- Ask questions in GitHub Discussions
- Tag maintainers if urgent (sparingly)

## License

By contributing to dchat, you agree that your contributions will be licensed under the same license as the project (MIT OR Apache-2.0).

## Recognition

Contributors will be:
- Listed in CONTRIBUTORS.md
- Acknowledged in release notes
- Credited in security hall of fame (for security findings)

---

**Thank you for contributing to dchat!** 🚀

Questions? Open a discussion or reach out to maintainers.
