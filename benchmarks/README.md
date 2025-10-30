# dchat Performance Benchmarks

## Overview

This directory contains performance benchmarks for the dchat system, measuring throughput, latency, resource usage, and scalability under various conditions.

## Benchmark Categories

### 1. Message Throughput
- Single client send rate
- Multi-client concurrent sending
- Relay message processing rate
- End-to-end delivery latency

### 2. Network Performance
- Peer discovery time
- Connection establishment latency
- NAT traversal success rate
- Relay routing overhead

### 3. Database Operations
- Message insertion rate
- Query performance (recent messages, search)
- Index efficiency
- Concurrent access scaling

### 4. Cryptographic Operations
- Key generation time
- Signature creation/verification
- Encryption/decryption throughput
- Noise protocol handshake time

### 5. Memory & Resource Usage
- Per-client memory footprint
- Per-relay memory footprint
- Message queue memory growth
- Database cache effectiveness

### 6. Scalability Tests
- 10 concurrent clients
- 50 concurrent clients
- 100 concurrent clients
- 500 concurrent clients

### 7. Relay Node Performance
- Messages relayed per second
- Connection capacity
- Proof-of-delivery overhead
- Reward calculation time

## Running Benchmarks

### Individual Benchmarks
```bash
cargo bench --bench message_throughput
cargo bench --bench crypto_performance
cargo bench --bench database_queries
cargo bench --bench network_latency
```

### All Benchmarks
```bash
cargo bench
```

### With Result Export
```bash
cargo bench -- --save-baseline main
cargo bench -- --baseline main
```

## Benchmark Results Format

Results are saved in `target/criterion/` as:
- HTML reports with graphs
- JSON data files
- CSV exports for analysis

## Performance Targets

### Message Throughput
- Single client: > 100 msgs/sec
- 10 clients: > 500 msgs/sec aggregate
- 50 clients: > 1000 msgs/sec aggregate
- Relay node: > 5000 msgs/sec

### Latency
- Local delivery: < 10ms p99
- Single relay hop: < 50ms p99
- Multi-hop (3 relays): < 150ms p99

### Cryptographic Operations
- Key generation: < 5ms
- Signature: < 1ms
- Encryption (1KB): < 2ms
- Handshake: < 20ms

### Database Operations
- Insert: < 5ms p99
- Query (100 messages): < 10ms p99
- Search: < 50ms p99

### Memory Usage
- Client base: < 50MB
- Client (1000 messages): < 100MB
- Relay base: < 100MB
- Relay (10k active connections): < 2GB

## Baseline Measurements

Run `./scripts/establish_baselines.sh` to create initial performance baselines for comparison.

## Regression Detection

Benchmarks run in CI will compare against saved baselines and fail if:
- Throughput decreases > 10%
- Latency increases > 15%
- Memory usage increases > 20%
