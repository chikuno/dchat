# Multi-stage Docker build for dchat
# Production-ready with security hardening

# Stage 1: Build environment
FROM rust:1.83-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev \
    libsqlite3-dev \
    pkg-config \
    curl \
    ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy rust toolchain
COPY rust-toolchain.toml .

# Copy workspace files
# Include Cargo.lock for reproducible builds and better Docker layer caching
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY src ./src
COPY benches ./benches

# Generate and lock dependencies
RUN cargo fetch

# Build release binary
RUN cargo build --release --bin dchat

# Strip debug symbols
RUN strip /app/target/release/dchat

# Stage 2: Runtime environment
FROM debian:bookworm-slim

# Install runtime dependencies and setup
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    libc6 \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user with restricted shell
RUN useradd -m -u 1000 -s /usr/sbin/nologin dchat

# Create data directories
RUN mkdir -p /var/lib/dchat/data /var/lib/dchat/config \
    && chown -R dchat:dchat /var/lib/dchat

# Copy binary from builder
COPY --from=builder /app/target/release/dchat /usr/local/bin/dchat

# Switch to non-root user
USER dchat
WORKDIR /home/dchat

# Expose ports
# 7070: P2P networking
# 7071: RPC API
# 9090: Metrics
EXPOSE 7070 7071 9090

# Set environment variables
ENV RUST_LOG=info
ENV DCHAT_DATA_DIR=/var/lib/dchat/data
ENV DCHAT_CONFIG_DIR=/var/lib/dchat/config

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD dchat health --url http://127.0.0.1:8080/health || exit 1

# Run as relay node by default
ENTRYPOINT ["/usr/local/bin/dchat"]
CMD ["relay", "--listen", "0.0.0.0:7070"]
