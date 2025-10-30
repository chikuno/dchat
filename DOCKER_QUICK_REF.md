# Docker Quick Reference - dchat Testnet

## 🚀 Quick Start (5 Minutes)

```powershell
# 1. Start Docker Desktop (if not running)
Start-Service Docker-Desktop

# 2. Verify setup
.\verify-docker.ps1

# 3. Create config files (if needed)
.\verify-docker.ps1 -Fix

# 4. Deploy testnet
docker-compose up -d

# 5. Check status
docker-compose ps

# 6. View logs
docker-compose logs -f relay1

# 7. Test connectivity
.\test-connectivity.ps1

# 8. Open dashboards
Start-Process "http://localhost:3000"      # Grafana
Start-Process "http://localhost:16686"     # Jaeger
Start-Process "http://localhost:9093"      # Prometheus
```

---

## 📊 Dashboard URLs

| Dashboard | URL | Login |
|-----------|-----|-------|
| **Grafana** | http://localhost:3000 | admin/admin |
| **Prometheus** | http://localhost:9093 | (no auth) |
| **Jaeger** | http://localhost:16686 | (no auth) |
| **Relay1 Health** | http://localhost:8080/health | (no auth) |
| **Relay1 Metrics** | http://localhost:9090/metrics | (no auth) |

---

## 🐳 Essential Docker Commands

### Deployment
```powershell
docker-compose up -d              # Start all services (background)
docker-compose up                 # Start with logs (foreground)
docker-compose down               # Stop all services
docker-compose down -v            # Stop and remove volumes (DATA LOSS!)
docker-compose restart            # Restart all services
docker-compose restart relay1     # Restart specific service
```

### Status
```powershell
docker-compose ps                 # Show all services
docker-compose ps --services      # List service names
docker ps                          # Show running containers
docker volume ls                   # Show volumes
docker network ls                  # Show networks
```

### Logs & Debugging
```powershell
docker-compose logs               # Show all logs (once)
docker-compose logs -f            # Follow logs in real-time
docker-compose logs --tail 50     # Last 50 lines
docker logs dchat-relay1          # Single container
docker logs -f dchat-relay1       # Follow single container
docker exec dchat-relay1 bash     # Execute command in container
docker exec dchat-relay1 curl http://relay2:9091  # Test connectivity
```

### Maintenance
```powershell
docker-compose config             # Validate configuration
docker-compose build              # Rebuild images
docker-compose pull               # Update images
docker system prune               # Clean up unused objects
docker volume prune               # Remove unused volumes
```

---

## 🔧 Configuration

### Relay Nodes
- **Config file**: `config/relay{1,2,3}.toml`
- **Data volume**: `dchat_relay{1,2,3}_data`
- **Ports**: 7070 (relay1), 7072 (relay2), 7074 (relay3)
- **Metrics**: 9090 (relay1), 9091 (relay2), 9092 (relay3)

### Database
- **Service**: `postgres:16-alpine`
- **Volume**: `dchat_postgres_data`
- **Port**: 5432 (internal only, use container exec)
- **User**: dchat
- **Password**: dchat

### Monitoring
- **Prometheus**: `monitoring/prometheus.yml`
- **Grafana**: `monitoring/grafana/`
- **Jaeger**: OTLP on 4317 (gRPC), 4318 (HTTP)

---

## ✅ Verification Checklist

```powershell
# Run all checks automatically
.\verify-docker.ps1

# Or check individually:
docker --version                    # Docker installed
docker ps                           # Daemon running
Test-Path "docker-compose.yml"      # Files exist
docker-compose ps                   # Services running
curl http://localhost:3000          # Grafana accessible
curl http://localhost:9093          # Prometheus accessible
curl http://localhost:16686         # Jaeger accessible
```

---

## 🚨 Troubleshooting

| Problem | Solution |
|---------|----------|
| **"Docker daemon not running"** | Start Docker Desktop: `Start-Service Docker-Desktop` |
| **"Port already in use"** | Kill process: `Get-NetStatistics -LocalPort 3000 \| Stop-Process` |
| **"Config file not found"** | Create: `.\verify-docker.ps1 -Fix` |
| **"Relay won't start"** | Check logs: `docker logs dchat-relay1` |
| **"Grafana shows no data"** | Verify Prometheus targets: `curl http://localhost:9093/api/v1/targets` |
| **"Out of disk space"** | Clean up: `docker system prune -a` |

---

## 📈 Performance Commands

```powershell
# Monitor in real-time
.\test-connectivity.ps1 -Watch

# Check container resource usage
docker stats

# View network traffic
docker exec dchat-relay1 netstat -an

# Check database connections
docker exec dchat-postgres psql -U dchat -c "SELECT count(*) FROM pg_stat_activity;"

# View metrics directly
curl http://localhost:9090/metrics | grep dchat_
```

---

## 🔄 Common Workflows

### Reset Everything
```powershell
docker-compose down -v
docker-compose up -d
.\verify-docker.ps1
```

### Update Configuration
```powershell
# Edit config
notepad config\relay1.toml

# Restart affected service
docker-compose restart relay1

# Check logs
docker logs -f dchat-relay1
```

### Scale to 5 Relays
```powershell
# Edit docker-compose.yml to add relay4, relay5
notepad docker-compose.yml

# Update prometheus.yml with new targets
notepad monitoring\prometheus.yml

# Restart
docker-compose down
docker-compose up -d
```

### Collect Debug Info
```powershell
# Save all information for bug report
docker version > debug.txt
docker-compose version >> debug.txt
docker-compose ps >> debug.txt
docker logs dchat-relay1 >> debug.txt
docker logs dchat-postgres >> debug.txt

# Share debug.txt for support
```

---

## 📚 Full Documentation

See `DOCKER_SETUP.md` for complete documentation including:
- Detailed setup instructions
- Configuration options
- Advanced troubleshooting
- Performance tuning
- Multi-environment setup

---

## 🎯 Next Steps

1. ✅ Start Docker Desktop
2. ✅ Run `.\verify-docker.ps1`
3. ✅ Run `docker-compose up -d`
4. ✅ Open http://localhost:3000 (Grafana)
5. ✅ Run `.\test-connectivity.ps1 -Watch`

---

**Status**: Production Ready ✅  
**Version**: dchat 0.1.0  
**Docker**: 4.49.0  
**Last Updated**: 2024
