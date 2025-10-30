# Load Testing for dchat

This directory contains load testing scripts for the dchat relay network.

## Tools

### k6 (JavaScript)
- **File**: `relay_stress_test.js`
- **Language**: JavaScript
- **Best For**: HTTP/WebSocket load testing, complex scenarios

### Locust (Python)
- **File**: `locustfile.py`
- **Language**: Python
- **Best For**: Distributed load testing, realistic user behavior

## k6 Load Testing

### Installation
```bash
# macOS
brew install k6

# Windows
choco install k6

# Linux
sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6
```

### Usage
```bash
# Basic run
k6 run relay_stress_test.js

# Custom VUs and duration
k6 run --vus 50 --duration 2m relay_stress_test.js

# Output results to file
k6 run --out json=results.json relay_stress_test.js

# With environment variables
k6 run -e RELAY_URL=http://production-relay:7071 relay_stress_test.js
```

### Test Stages
1. Ramp up to 10 VUs (30s)
2. Ramp up to 50 VUs (1m)
3. Ramp up to 100 VUs (2m)
4. Sustain 100 VUs (2m)
5. Ramp down to 50 VUs (1m)
6. Ramp down to 0 VUs (30s)

**Total Duration**: 7 minutes

### Metrics
- **HTTP duration**: p95 < 500ms
- **Error rate**: < 1%
- **Custom error rate**: < 5%
- **Message latency**: Tracked as trend

### Test Scenarios
- Health check
- Send message
- Receive messages
- Relay status

## Locust Load Testing

### Installation
```bash
pip install locust
```

### Usage
```bash
# Start web UI
locust -f locustfile.py --host=http://localhost:7071

# Headless mode
locust -f locustfile.py --host=http://localhost:7071 --headless -u 100 -r 10 -t 5m

# Distributed mode (master)
locust -f locustfile.py --master

# Distributed mode (worker)
locust -f locustfile.py --worker --master-host=<master-ip>
```

### User Types

#### DchatUser (Weight: 10)
- Normal user behavior
- 1-3 second wait between requests
- Tasks:
  - Send message (weight 3)
  - Get messages (weight 2)
  - Relay status (weight 1)
  - Health check (weight 1)

#### HeavyUser (Weight: 1)
- High-volume user
- 0.5-1.5 second wait between requests
- Sends bursts of 5 messages with 100ms intervals

#### BurstUser (Weight: 0.5)
- Bursty traffic pattern
- 5-10 second wait between bursts
- Sends 10-20 messages per burst

### Web UI
- Default: http://localhost:8089
- Real-time metrics
- Charts and graphs
- Download results (CSV)

## Comparison: k6 vs Locust

| Feature | k6 | Locust |
|---------|-----|--------|
| **Language** | JavaScript | Python |
| **Performance** | Faster (Go-based) | Slower (Python-based) |
| **Ease of Use** | Simpler syntax | More flexible |
| **Distributed** | Cloud-only | Built-in |
| **UI** | CLI + Cloud | Web UI |
| **Cost** | Free (CLI) | Free (open-source) |
| **Best For** | CI/CD, simple tests | Complex scenarios, distributed |

## Relay Setup

Before running load tests, ensure relay nodes are running:

### Docker Compose (Recommended)
```bash
cd ../..
docker-compose up -d
```

### Manual
```bash
cd ../..
cargo run --release -- --role relay --port 7071
```

## Interpreting Results

### Good Performance
- p95 latency < 500ms
- p99 latency < 1s
- Error rate < 1%
- Throughput > 100 msg/s per relay

### Warning Signs
- p95 latency > 500ms → Database or network bottleneck
- Error rate > 1% → Application errors, check logs
- Throughput < 50 msg/s → Relay overloaded
- Memory growth → Memory leak

### Troubleshooting
1. Check relay logs: `docker-compose logs -f relay1`
2. Check Prometheus metrics: http://localhost:9093
3. Check Grafana dashboards: http://localhost:3000
4. Check Jaeger traces: http://localhost:16686
5. Check database connections: `docker-compose exec postgres psql -U dchat -c "SELECT * FROM pg_stat_activity;"`

## CI Integration

### GitHub Actions Example
```yaml
- name: Run k6 load test
  run: |
    docker-compose up -d
    sleep 10  # Wait for services to be ready
    k6 run tests/load/relay_stress_test.js
    docker-compose down
```

### GitLab CI Example
```yaml
load_test:
  stage: test
  script:
    - docker-compose up -d
    - sleep 10
    - k6 run tests/load/relay_stress_test.js
  after_script:
    - docker-compose down
```

## Advanced Scenarios

### Custom Test Data
Edit `relay_stress_test.js` or `locustfile.py` to:
- Change message content
- Adjust user behavior
- Add new endpoints
- Customize metrics

### Performance Profiling
```bash
# CPU profiling
cargo flamegraph --bin dchat -- --role relay

# Memory profiling
heaptrack cargo run --release -- --role relay
```

## References

- [k6 Documentation](https://k6.io/docs/)
- [Locust Documentation](https://docs.locust.io/)
- [dchat Architecture](../../ARCHITECTURE.md)
- [dchat API Documentation](../../docs/API_DOCUMENTATION.md)
- [Sprint 5 Completion](../../PHASE7_SPRINT5_FINAL_COMPLETE.md)
