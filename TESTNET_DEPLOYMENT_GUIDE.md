# dchat Testnet Deployment Guide - October 30, 2025

## 🚀 Deployment to rpc.webnetcore.top:8080

### Prerequisites
- Virtual server with Docker and Docker Compose installed
- Domain: `rpc.webnetcore.top` pointing to your server
- Port 8080 exposed and forwarded to your server

### Deployment Steps

#### 1. SSH into Your Virtual Server
```bash
ssh user@your-server-ip
cd /opt/dchat-testnet
```

#### 2. Clone/Copy the dchat Repository
```bash
git clone https://github.com/chikuno/dchat.git
cd dchat
```

#### 3. Build the Docker Image
```bash
# Build the main dchat image
docker build -t dchat:latest .

# Verify build
docker images | grep dchat
```

#### 4. Generate Validator Keys
```bash
# Create validator keys directory
mkdir -p validator_keys

# Run key generation (from main project)
cargo run --release -- keys generate --count 4 --output validator_keys/
```

#### 5. Start the Testnet

**Option A: Basic Testnet (Lighter Resources)**
```bash
docker-compose -f docker-compose.yml up -d
```

**Option B: Full Testnet (Production-like)**
```bash
docker-compose -f docker-compose-testnet.yml up -d
```

#### 6. Verify Services Are Running
```bash
# Check all containers
docker ps

# View logs from a specific service
docker logs dchat-validator1 -f

# Health check
curl http://localhost:7071/health
```

### Service Endpoints

| Service | Internal Port | External Port | URL |
|---------|---|---|---|
| Validator 1 RPC | 7071 | 7071 | http://localhost:7071 |
| Validator 1 P2P | 7070 | 7070 | 0.0.0.0:7070 |
| Relay 1 RPC | 7081 | 7081 | http://localhost:7081 |
| Relay 1 P2P | 7080 | 7080 | 0.0.0.0:7080 |
| Prometheus | 9090 | 9090 | http://localhost:9090 |
| Grafana | 3000 | 3000 | http://localhost:3000 (admin/admin) |
| Jaeger | 16686 | 16686 | http://localhost:16686 |

### Nginx Reverse Proxy Configuration

If you want to expose via `rpc.webnetcore.top:8080`:

**File: `/etc/nginx/sites-available/rpc.webnetcore.top`**
```nginx
upstream dchat_rpc {
    server localhost:7071;  # Validator 1 RPC
}

server {
    listen 8080;
    server_name rpc.webnetcore.top;
    
    location / {
        proxy_pass http://dchat_rpc;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_read_timeout 86400;
    }
}
```

**Enable and test:**
```bash
sudo ln -s /etc/nginx/sites-available/rpc.webnetcore.top /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### Access Testnet

**From Your Local Machine:**
```bash
# Query validator 1
curl http://rpc.webnetcore.top:8080/health

# Query relay 1
curl http://rpc.webnetcore.top:8081/health
```

### Monitoring & Debugging

**View Real-Time Logs:**
```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f validator1

# With timestamps
docker-compose logs -f --timestamps
```

**Check Container Stats:**
```bash
docker stats

# Specific container
docker stats dchat-validator1
```

**Execute Commands Inside Container:**
```bash
docker exec -it dchat-validator1 /bin/bash
```

**Inspect Network:**
```bash
docker network inspect dchat-testnet
docker network inspect dchat-network
```

### Stopping & Cleanup

**Stop All Services (Keep Data):**
```bash
docker-compose down
```

**Stop & Remove Everything (Delete Volumes):**
```bash
docker-compose down -v
```

**Restart Specific Service:**
```bash
docker-compose restart validator1
```

### Performance Tuning

**For Production Deployment:**

1. **Increase Resource Limits:**
```yaml
services:
  validator1:
    resources:
      limits:
        cpus: '2'
        memory: 4G
      reservations:
        cpus: '1'
        memory: 2G
```

2. **Enable Log Rotation:**
```yaml
services:
  validator1:
    logging:
      driver: "json-file"
      options:
        max-size: "100m"
        max-file: "5"
```

3. **Optimize Consensus:**
```environment
DCHAT_CONSENSUS_TIMEOUT=5000      # Increase timeout
DCHAT_BLOCK_TIME=5000             # Slower blocks = less CPU
```

### Troubleshooting

**Containers Keep Restarting:**
```bash
# Check logs
docker logs dchat-validator1 --tail 50

# Rebuild image
docker-compose build --no-cache validator1
docker-compose up -d validator1
```

**Network Connectivity Issues:**
```bash
# Test P2P connection between nodes
docker exec dchat-validator1 nc -zv validator2 7070
docker exec dchat-validator1 nc -zv relay1 7080
```

**Consensus Stalled:**
```bash
# Check validator status
curl http://localhost:7071/status
curl http://localhost:7071/validators

# Restart validators
docker-compose restart validator1 validator2 validator3 validator4
```

**Port Already in Use:**
```bash
# Find what's using the port
sudo lsof -i :8080
sudo kill -9 <PID>
```

### Backup & Recovery

**Backup Validator Data:**
```bash
docker-compose exec -T validator1 tar czf - /var/lib/dchat/data | gzip > validator1_backup.tar.gz
```

**Restore Validator Data:**
```bash
zcat validator1_backup.tar.gz | docker-compose exec -T validator1 tar xzf -
```

### Next Steps

1. ✅ Deploy testnet to rpc.webnetcore.top:8080
2. Test message propagation between nodes
3. Run consensus tests with multiple validators
4. Monitor performance via Grafana dashboard
5. Conduct chaos testing (network partitions, node failures)
6. Prepare for mainnet deployment

### Support & Documentation

- **Architecture**: See `ARCHITECTURE.md`
- **API Reference**: See `API_SPECIFICATION.md`
- **Security Audit**: See `PHASE7_SPRINT4_SECURITY_AUDIT.md`
- **Docker Docs**: See `DOCKER_QUICK_REF.md`

---

**Deployment Date**: October 30, 2025
**Status**: Ready for Deployment ✅
