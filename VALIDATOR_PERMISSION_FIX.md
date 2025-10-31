# Validator Permission Fix

## Problem
Validators are failing with `Permission denied` error when trying to read key files:
```
Error: Io(Os { code: 13, kind: PermissionDenied, message: "Permission denied" })
Loading validator key from file: /validator_keys/validator1.key
```

## Root Cause
The Docker container runs as user `dchat` (UID 1000), but the `validator_keys/` directory on the host is owned by root or has restrictive permissions (400).

## Solution

### Quick Fix (On Production Server)

```bash
# SSH into the server
ssh user@rpc.webnetcore.top

# Navigate to dchat directory
cd /opt/dchat

# Check what's using port 9090
sudo lsof -i :9090
# or
sudo netstat -tulpn | grep 9090

# Kill the process using port 9090
sudo kill -9 $(sudo lsof -t -i:9090)

# Or if it's another Prometheus instance
sudo systemctl stop prometheus 2>/dev/null || true
docker stop $(docker ps -q --filter "publish=9090") 2>/dev/null || true

# Fix permissions on validator_keys directory
sudo chown -R 1000:1000 validator_keys/
chmod 755 validator_keys/
chmod 644 validator_keys/*.key

# Restart validators
docker-compose -f docker-compose-production.yml restart validator1 validator2 validator3 validator4

# Verify they're running
docker ps | grep validator

# Check logs
docker logs dchat-validator1 --tail=20
```

### Alternative Solution: Generate Keys Inside Container

If you don't have pre-generated keys, let the containers generate their own:

1. **Update docker-compose-production.yml** - Remove the validator_keys volume mount
2. **Let validators auto-generate keys** on first startup in their persistent volumes

```bash
# Stop all services
docker-compose -f docker-compose-production.yml down

# Remove validator_keys volume mount requirement
# Edit docker-compose-production.yml and change:
# volumes:
#   - ./validator_keys:/validator_keys:ro
# TO:
# volumes:
#   - validator1_data:/var/lib/dchat/data

# Update the command to use the data directory:
# command: validator --key /var/lib/dchat/data/validator1.key ...

# Start services
docker-compose -f docker-compose-production.yml up -d
```

### Comprehensive Fix (Recommended)

Create a setup script:

```bash
#!/bin/bash
# fix-validator-permissions.sh

echo "Fixing validator key permissions..."

# Create validator_keys directory if it doesn't exist
mkdir -p validator_keys

# Generate keys if they don't exist
for i in {1..4}; do
    if [ ! -f "validator_keys/validator$i.key" ]; then
        echo "Generating validator$i.key..."
        docker run --rm \
            -v "$(pwd)/validator_keys:/keys" \
            dchat:latest \
            keygen -o /keys/validator$i.key
    fi
done

# Fix ownership (UID 1000 = dchat user in container)
sudo chown -R 1000:1000 validator_keys/

# Fix permissions (readable by container user)
chmod 755 validator_keys/
chmod 644 validator_keys/*.key

echo "✅ Permissions fixed!"
echo "Now restart validators: docker-compose -f docker-compose-production.yml restart validator1 validator2 validator3 validator4"
```

Make it executable and run:
```bash
chmod +x fix-validator-permissions.sh
./fix-validator-permissions.sh
```

## Verification

After fixing permissions:

```bash
# Check file ownership
ls -la validator_keys/

# Expected output:
# -rw-r--r-- 1 1000 1000 <size> <date> validator1.key
# -rw-r--r-- 1 1000 1000 <size> <date> validator2.key
# -rw-r--r-- 1 1000 1000 <size> <date> validator3.key
# -rw-r--r-- 1 1000 1000 <size> <date> validator4.key

# Restart validators
docker-compose -f docker-compose-production.yml restart validator1 validator2 validator3 validator4

# Wait 10 seconds for startup
sleep 10

# Check validator health
docker ps | grep validator

# Expected: All should show "Up" (not "Restarting")

# Check logs for success
docker logs dchat-validator1 --tail=20

# Expected: Should see "✓ Validator key loaded" without errors
```

## Prevention

To prevent this issue in the future:

1. **Always set correct permissions** when generating keys:
   ```bash
   chown 1000:1000 validator_keys/*.key
   chmod 644 validator_keys/*.key
   ```

2. **Use volume mounts with correct user** in docker-compose.yml:
   ```yaml
   volumes:
     - ./validator_keys:/validator_keys:ro
   user: "1000:1000"  # Add this line
   ```

3. **Or use bind mounts with proper permissions**:
   ```yaml
   volumes:
     - type: bind
       source: ./validator_keys
       target: /validator_keys
       read_only: true
   ```

## Related Issues

If users/relays are also unhealthy, check:

```bash
# Check relay logs
docker logs dchat-relay1 --tail=50

# Check user logs
docker logs dchat-user1 --tail=50

# Common issues:
# - Missing validator connection (validators must be healthy first)
# - Health check curl command not found (should use dchat health command)
# - Network connectivity between containers
```

## Status After Fix

✅ Validators should transition from "Restarting" to "Up (healthy)"
✅ Relays should become healthy once validators are running
✅ Users should become healthy once relays are available
