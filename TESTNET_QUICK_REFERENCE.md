# dchat Testnet - Quick Reference Card

## 🚀 Deployment Commands

```bash
# Pre-flight check
./pre-deployment-check.sh

# Full deployment
sudo ./deploy-ubuntu-testnet.sh

# Skip Docker install (if already have it)
sudo ./deploy-ubuntu-testnet.sh --skip-docker

# Skip rebuild (faster restart)
sudo ./deploy-ubuntu-testnet.sh --skip-build
```

## 📊 Management Commands

```bash
# Status
./status-testnet.sh                          # Quick status
docker ps                                    # All containers
docker compose -f docker-compose-testnet.yml -p dchat-testnet ps

# Logs
./logs-testnet.sh                           # All logs
./logs-testnet.sh validator1                # Specific container
docker logs -f dchat-validator1             # Follow logs
docker logs --tail 100 dchat-relay1         # Last 100 lines

# Start/Stop
./start-testnet.sh                          # Start all
./stop-testnet.sh                           # Stop all
docker compose -f docker-compose-testnet.yml -p dchat-testnet restart validator1
docker compose -f docker-compose-testnet.yml -p dchat-testnet restart

# Stats
docker stats                                # Resource usage
docker system df                            # Disk usage
```

## 🔍 Health Checks

```bash
# Quick health check all validators
for port in 7071 7073 7075 7077; do
  echo "Validator on $port:"
  curl -s http://localhost:$port/health | jq
done

# Quick health check all relays
for port in 7081 7083 7085 7087 7089 7091 7093; do
  curl -s http://localhost:$port/health
done

# Check Prometheus
curl http://localhost:9090/-/healthy

# Check Grafana
curl http://localhost:3000/api/health
```

## 🌐 Web Interfaces

| Service | URL | Credentials |
|---------|-----|-------------|
| Grafana | http://YOUR_IP:3000 | admin / admin |
| Prometheus | http://YOUR_IP:9090 | - |
| Jaeger | http://YOUR_IP:16686 | - |

## 🔌 Network Ports

### Validators
- validator1: 7070 (P2P), 7071 (RPC), 9090 (metrics)
- validator2: 7072 (P2P), 7073 (RPC), 9091 (metrics)
- validator3: 7074 (P2P), 7075 (RPC), 9092 (metrics)
- validator4: 7076 (P2P), 7077 (RPC), 9093 (metrics)

### Relays
- relay1-7: 7080-7093 (P2P), 7081-7094 (RPC), 9100-9106 (metrics)

### Users
- user1: 7110 (P2P), 7111 (RPC), 9110 (metrics)
- user2: 7112 (P2P), 7113 (RPC), 9111 (metrics)
- user3: 7114 (P2P), 7115 (RPC), 9112 (metrics)

## 🐛 Troubleshooting

```bash
# Container won't start
docker logs dchat-validator1                 # Check errors
docker compose -f docker-compose-testnet.yml -p dchat-testnet up -d --force-recreate validator1

# Port conflicts
sudo netstat -tulpn | grep 7070             # Check port usage
sudo lsof -i :7070                          # What's using the port

# Out of disk space
df -h                                       # Check space
docker system prune -a --volumes            # Clean up (DELETES DATA)
docker image prune -a                       # Remove unused images

# Performance issues
docker stats                                # Check resource usage
free -h                                     # Check RAM
htop                                        # System monitor

# Network issues
docker network inspect dchat-testnet        # Network details
docker exec -it dchat-user1 ping validator1 # Test connectivity
ufw status                                  # Check firewall

# Container keeps restarting
docker inspect dchat-validator1 | grep -A 10 "RestartCount"
docker logs --tail 200 dchat-validator1     # Recent logs
docker events --since 10m                   # Recent Docker events

# Reset everything (NUCLEAR OPTION)
docker compose -f docker-compose-testnet.yml -p dchat-testnet down -v
docker system prune -a --volumes --force
sudo ./deploy-ubuntu-testnet.sh
```

## 📈 Monitoring Queries (Prometheus)

```promql
# Message rate
rate(dchat_messages_total[5m])

# Validator block height
dchat_block_height{role="validator"}

# Relay uptime %
avg_over_time(up{job="dchat-relays"}[1h]) * 100

# P2P connections
dchat_p2p_connections

# Memory usage
container_memory_usage_bytes{name=~"dchat-.*"}

# CPU usage
rate(container_cpu_usage_seconds_total{name=~"dchat-.*"}[5m])

# Network latency p95
histogram_quantile(0.95, rate(dchat_network_latency_seconds_bucket[5m]))
```

## 🔒 Security

```bash
# Secure validator keys
chmod 600 validator_keys/*.key
chown root:root validator_keys/*.key

# Change Grafana password
docker exec -it dchat-grafana grafana-cli admin reset-admin-password NEWPASS

# SSH tunnel for monitoring (instead of exposing ports)
ssh -L 3000:localhost:3000 user@server-ip

# Update firewall rules
sudo ufw allow from YOUR_IP to any port 3000
sudo ufw delete allow 3000

# Check open ports
sudo netstat -tulpn
```

## 🔄 Updates & Maintenance

```bash
# Pull latest code
git pull origin main

# Rebuild and restart (zero downtime)
docker compose -f docker-compose-testnet.yml build
docker compose -f docker-compose-testnet.yml up -d --no-deps validator1
# Wait 30s between validators

# Update system packages
sudo apt update && sudo apt upgrade -y

# Update Docker images
docker compose -f docker-compose-testnet.yml pull
docker compose -f docker-compose-testnet.yml up -d

# Backup data
tar -czf backup-$(date +%Y%m%d).tar.gz dchat_data/ validator_keys/
```

## 💾 Backup & Restore

```bash
# Backup
tar -czf dchat-backup-$(date +%Y%m%d).tar.gz \
  dchat_data/ \
  validator_keys/ \
  testnet-config.toml

# Restore
tar -xzf dchat-backup-YYYYMMDD.tar.gz
sudo ./deploy-ubuntu-testnet.sh --skip-build

# Export metrics (Prometheus)
curl http://localhost:9090/api/v1/query?query=up > metrics-snapshot.json
```

## 🧪 Testing

```bash
# Send test message
docker exec -it dchat-user1 dchat send --to user2@dchat.local --message "Test"

# Check inbox
docker exec -it dchat-user2 dchat inbox

# Run benchmark
docker exec -it dchat-relay1 dchat benchmark --duration 60 --messages 1000

# Check consensus
for port in 7071 7073 7075 7077; do
  curl -s http://localhost:$port/status | jq '.block_height'
done
```

## 📞 Emergency Procedures

### Total System Crash
```bash
# Check system health
free -h                                     # RAM
df -h                                       # Disk
dmesg | tail                                # Kernel messages

# Restart everything
sudo reboot                                 # If needed
sudo ./deploy-ubuntu-testnet.sh --skip-build
```

### Data Corruption
```bash
# Stop services
./stop-testnet.sh

# Restore from backup
rm -rf dchat_data/
tar -xzf dchat-backup-YYYYMMDD.tar.gz

# Restart
./start-testnet.sh
```

### Network Partition
```bash
# Check validator consensus
./status-testnet.sh
for port in 7071 7073 7075 7077; do
  curl -s http://localhost:$port/status
done

# Restart affected validators one at a time
docker compose -f docker-compose-testnet.yml -p dchat-testnet restart validator1
```

## 📚 Log Locations

- Deployment: `/var/log/dchat-deployment.log`
- Container logs: `docker logs <container-name>`
- System logs: `/var/log/syslog`
- Docker daemon: `journalctl -u docker`

## 🆘 Quick Help

```bash
# Get help
./deploy-ubuntu-testnet.sh --help
./pre-deployment-check.sh

# Documentation
cat README.md
cat TESTNET_DEPLOYMENT_UBUNTU.md
cat ARCHITECTURE.md
```

---

**Pro Tips:**
- Always run `pre-deployment-check.sh` before deploying
- Monitor disk space regularly: `df -h`
- Check logs if container restarts: `docker logs <container>`
- Use `docker stats` to watch resource usage live
- Create regular backups of `dchat_data/` and `validator_keys/`
- Keep Grafana open to watch network health
- SSH tunnel for monitoring access (more secure than open ports)

**Emergency Contact:**
- Logs: `/var/log/dchat-deployment.log`
- Status: `./status-testnet.sh`
- GitHub Issues: https://github.com/your-org/dchat/issues

**Last Updated:** October 31, 2025
