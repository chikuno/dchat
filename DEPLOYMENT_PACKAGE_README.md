# 🚀 dchat Testnet Deployment Package

**Complete, production-ready deployment scripts for Ubuntu Server**

This package contains everything you need to deploy a full dchat testnet on a clean Ubuntu server with zero manual configuration required.

## 📦 What's Included

| File | Purpose |
|------|---------|
| `deploy-ubuntu-testnet.sh` | **Main deployment script** - Automated installation and configuration |
| `pre-deployment-check.sh` | Pre-flight validation - Checks prerequisites before deploying |
| `test-deployment.sh` | Post-deployment validation - Verifies everything works |
| `TESTNET_DEPLOYMENT_UBUNTU.md` | **Comprehensive deployment guide** - Step-by-step instructions |
| `TESTNET_QUICK_REFERENCE.md` | **Quick reference card** - Common commands and troubleshooting |
| `start-testnet.sh` | Generated management script - Start the testnet |
| `stop-testnet.sh` | Generated management script - Stop the testnet |
| `logs-testnet.sh` | Generated management script - View logs |
| `status-testnet.sh` | Generated management script - Check status |

## 🎯 Quick Start (3 Commands)

```bash
# 1. Pre-flight check
chmod +x *.sh
./pre-deployment-check.sh

# 2. Deploy (15-45 minutes)
sudo ./deploy-ubuntu-testnet.sh

# 3. Verify
./test-deployment.sh
```

That's it! Your testnet is now running.

## 📋 Prerequisites

- **Server**: Ubuntu 20.04/22.04/24.04 LTS
- **Resources**: 4GB+ RAM, 50GB+ disk, 2+ CPU cores
- **Network**: Public IP with internet access
- **Access**: Root or sudo privileges
- **Validator Keys**: Must be generated first (see below)

## 🔑 Before You Deploy: Generate Validator Keys

**IMPORTANT**: You must generate validator keys before deploying.

### On Windows (Local Machine)
```powershell
# Run the key generation script
powershell -ExecutionPolicy Bypass -File generate-validator-keys.ps1

# This creates validator_keys/ directory with 4 key files
```

### Upload Keys to Server
```bash
# From your local machine
scp -r validator_keys root@your-server-ip:/root/dchat/

# Verify on server
ls -l /root/dchat/validator_keys/
# Should see: validator1.key, validator2.key, validator3.key, validator4.key
```

## 🚀 Deployment Process

### Step 1: Connect to Server
```bash
ssh root@your-server-ip
# or
ssh your-user@your-server-ip
```

### Step 2: Upload Repository
```bash
# Option A: Git clone
git clone https://github.com/your-org/dchat.git
cd dchat

# Option B: SCP upload (from local machine)
scp -r /path/to/dchat root@your-server-ip:/root/
ssh root@your-server-ip
cd dchat
```

### Step 3: Pre-Flight Check
```bash
chmod +x pre-deployment-check.sh
./pre-deployment-check.sh
```

**Review the output carefully**. Fix any critical failures before proceeding.

### Step 4: Deploy
```bash
chmod +x deploy-ubuntu-testnet.sh
sudo ./deploy-ubuntu-testnet.sh
```

**What happens during deployment:**
1. ✅ System updates (apt upgrade)
2. ✅ Docker & Docker Compose installation
3. ✅ Firewall configuration (UFW)
4. ✅ Validator key validation
5. ✅ Docker image building (10-30 min)
6. ✅ Network startup (4 validators, 7 relays, 3 users)
7. ✅ Monitoring stack (Prometheus, Grafana, Jaeger)
8. ✅ Health checks
9. ✅ Management script generation

**Total time: 15-45 minutes** (depending on server speed)

### Step 5: Verify Deployment
```bash
chmod +x test-deployment.sh
./test-deployment.sh
```

This runs automated tests to verify everything is working correctly.

## 🎛️ Deployment Options

```bash
# Standard deployment
sudo ./deploy-ubuntu-testnet.sh

# Skip Docker installation (if already installed)
sudo ./deploy-ubuntu-testnet.sh --skip-docker

# Skip image rebuild (faster restart with existing images)
sudo ./deploy-ubuntu-testnet.sh --skip-build

# Only update monitoring configuration
sudo ./deploy-ubuntu-testnet.sh --monitoring-only

# Help
./deploy-ubuntu-testnet.sh --help
```

## 📊 What Gets Deployed

### Network Topology
```
┌───────────────────────────────────────────┐
│         dchat Testnet Network              │
├───────────────────────────────────────────┤
│ ┌─────────────────────────────────────┐   │
│ │  Validators (4 nodes)               │   │
│ │  • Consensus & Block Production     │   │
│ │  • Ports: 7070-7077                 │   │
│ └─────────────────────────────────────┘   │
│                                           │
│ ┌─────────────────────────────────────┐   │
│ │  Relays (7 nodes)                   │   │
│ │  • Message Routing                  │   │
│ │  • Ports: 7080-7093                 │   │
│ └─────────────────────────────────────┘   │
│                                           │
│ ┌─────────────────────────────────────┐   │
│ │  Users (3 nodes)                    │   │
│ │  • End-User Clients                 │   │
│ │  • Ports: 7110-7115                 │   │
│ └─────────────────────────────────────┘   │
│                                           │
│ ┌─────────────────────────────────────┐   │
│ │  Monitoring                          │   │
│ │  • Prometheus: 9090                 │   │
│ │  • Grafana: 3000                    │   │
│ │  • Jaeger: 16686                    │   │
│ └─────────────────────────────────────┘   │
└───────────────────────────────────────────┘
```

### Containers
- **14 dchat nodes**: 4 validators + 7 relays + 3 users
- **3 monitoring services**: Prometheus, Grafana, Jaeger
- **Total: 17 containers**

## 🌐 Accessing Services

After deployment, access these URLs (replace `YOUR_IP` with your server IP):

| Service | URL | Credentials |
|---------|-----|-------------|
| **Grafana** | http://YOUR_IP:3000 | admin / admin |
| **Prometheus** | http://YOUR_IP:9090 | - |
| **Jaeger Tracing** | http://YOUR_IP:16686 | - |
| **Validator 1 RPC** | http://YOUR_IP:7071 | - |
| **Validator 2 RPC** | http://YOUR_IP:7073 | - |
| **Validator 3 RPC** | http://YOUR_IP:7075 | - |
| **Validator 4 RPC** | http://YOUR_IP:7077 | - |

**Security Note**: Change default passwords immediately in production!

## 🎮 Managing the Testnet

### Status & Monitoring
```bash
./status-testnet.sh                    # Quick status
docker ps                               # All containers
./logs-testnet.sh                      # All logs
./logs-testnet.sh validator1           # Specific container logs
docker stats                            # Live resource usage
```

### Start/Stop/Restart
```bash
./start-testnet.sh                     # Start all
./stop-testnet.sh                      # Stop all
docker compose -f docker-compose-testnet.yml -p dchat-testnet restart
```

### Health Checks
```bash
# Check all validators
for port in 7071 7073 7075 7077; do
  curl http://localhost:$port/health
done

# Check monitoring
curl http://localhost:9090/-/healthy   # Prometheus
curl http://localhost:3000/api/health   # Grafana
```

## 🧪 Testing the Network

### Test Message Sending
```bash
# Send message from user1 to user2
docker exec -it dchat-user1 dchat send \
  --to user2@dchat.local \
  --message "Hello from dchat testnet!"

# Check user2's inbox
docker exec -it dchat-user2 dchat inbox
```

### Test Validator Consensus
```bash
# Check block heights (should be similar)
for port in 7071 7073 7075 7077; do
  echo "Validator on port $port:"
  curl -s http://localhost:$port/status | jq '.block_height'
done
```

### Load Testing
```bash
# Run benchmark on relay1
docker exec -it dchat-relay1 dchat benchmark \
  --duration 60 \
  --messages 1000
```

## 🐛 Troubleshooting

### Common Issues

#### Container Won't Start
```bash
# Check logs
docker logs dchat-validator1

# Check for port conflicts
sudo netstat -tulpn | grep 7070

# Force recreate
docker compose -f docker-compose-testnet.yml -p dchat-testnet up -d --force-recreate validator1
```

#### Out of Disk Space
```bash
# Check space
df -h

# Clean Docker
docker system prune -a --volumes

# Clean logs
rm -rf testnet-logs/*.log
```

#### Can't Access Web UI
```bash
# Check if service is running
docker ps | grep grafana

# Check firewall
sudo ufw status | grep 3000

# Test locally first
curl http://localhost:3000
```

#### Network Issues
```bash
# Test container connectivity
docker exec -it dchat-user1 ping validator1

# Check Docker network
docker network inspect dchat-testnet

# Check firewall
sudo ufw status
```

### Get Help
```bash
# View logs
./logs-testnet.sh

# Check deployment log
tail -n 100 /var/log/dchat-deployment.log

# Run diagnostics
./test-deployment.sh

# Full documentation
cat TESTNET_DEPLOYMENT_UBUNTU.md
```

## 📚 Documentation

- **[TESTNET_DEPLOYMENT_UBUNTU.md](TESTNET_DEPLOYMENT_UBUNTU.md)** - Complete deployment guide (detailed)
- **[TESTNET_QUICK_REFERENCE.md](TESTNET_QUICK_REFERENCE.md)** - Quick reference card (commands & troubleshooting)
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture and design
- **[API_SPECIFICATION.md](API_SPECIFICATION.md)** - API documentation

## 🔒 Security Best Practices

### Before Production
- [ ] Change Grafana password
- [ ] Restrict firewall to specific IPs
- [ ] Enable SSL/TLS with reverse proxy
- [ ] Secure validator keys (chmod 600)
- [ ] Enable automatic security updates
- [ ] Set up monitoring alerts
- [ ] Configure log rotation
- [ ] Implement backup strategy

### Securing Monitoring Access
```bash
# Option 1: Firewall restriction
sudo ufw delete allow 3000
sudo ufw allow from YOUR_IP to any port 3000

# Option 2: SSH tunnel (recommended)
ssh -L 3000:localhost:3000 user@server-ip
# Then access http://localhost:3000 on your local machine
```

## 📦 Backup & Recovery

### Create Backup
```bash
# Backup all critical data
tar -czf dchat-backup-$(date +%Y%m%d).tar.gz \
  dchat_data/ \
  validator_keys/ \
  testnet-config.toml \
  docker-compose-testnet.yml
```

### Restore from Backup
```bash
# Extract backup
tar -xzf dchat-backup-YYYYMMDD.tar.gz

# Redeploy
sudo ./deploy-ubuntu-testnet.sh --skip-build
```

## 🔄 Updating

### Update System
```bash
# Update system packages
sudo apt update && sudo apt upgrade -y

# Update Docker images
docker compose -f docker-compose-testnet.yml pull
docker compose -f docker-compose-testnet.yml up -d
```

### Update dchat
```bash
# Pull latest code
git pull origin main

# Rebuild and restart
docker compose -f docker-compose-testnet.yml build
docker compose -f docker-compose-testnet.yml up -d
```

## ❓ FAQ

**Q: How long does deployment take?**  
A: 15-45 minutes depending on server speed and network bandwidth.

**Q: Can I deploy on other Linux distributions?**  
A: The script is designed for Ubuntu. Other distros may work but aren't tested.

**Q: Do I need to expose all ports publicly?**  
A: No. Use SSH tunneling for monitoring ports in production.

**Q: What if deployment fails?**  
A: Check `/var/log/dchat-deployment.log` and run `./pre-deployment-check.sh`

**Q: Can I scale up the number of nodes?**  
A: Yes, edit `docker-compose-testnet.yml` and add more services.

**Q: How do I completely reset everything?**  
A: Run `docker compose -f docker-compose-testnet.yml -p dchat-testnet down -v`

## 🆘 Support

- **Logs**: `/var/log/dchat-deployment.log`
- **Status**: `./status-testnet.sh`
- **Tests**: `./test-deployment.sh`
- **Documentation**: See files listed above
- **GitHub Issues**: https://github.com/your-org/dchat/issues

## 📋 Deployment Checklist

Complete this checklist to ensure successful deployment:

- [ ] Ubuntu server provisioned (4GB+ RAM, 50GB+ disk)
- [ ] SSH access configured
- [ ] Validator keys generated on local machine
- [ ] Validator keys uploaded to server
- [ ] Repository cloned/uploaded to server
- [ ] Scripts made executable (`chmod +x *.sh`)
- [ ] Pre-flight check passed (`./pre-deployment-check.sh`)
- [ ] Deployment script executed (`sudo ./deploy-ubuntu-testnet.sh`)
- [ ] All containers running (`./status-testnet.sh`)
- [ ] Post-deployment tests passed (`./test-deployment.sh`)
- [ ] Web interfaces accessible
- [ ] Grafana password changed
- [ ] Test message sent successfully
- [ ] Monitoring configured and working
- [ ] Backup strategy established

## 🎉 Success!

If you see this after running `./test-deployment.sh`:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ✓ All tests passed!
  Testnet is operational and healthy.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Congratulations! Your dchat testnet is live!** 🎊

---

**Version**: dchat v0.1.0 Testnet  
**Last Updated**: October 31, 2025  
**Deployment Scripts**: Production-Ready  
**Status**: ✅ Complete & Tested
