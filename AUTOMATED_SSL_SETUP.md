# Automated Let's Encrypt SSL Setup

**Domain**: `rpc.webnetcore.top`  
**Automation**: Fully automated via `deploy-ubuntu-testnet.sh`

---

## Overview

The deployment script now **automatically obtains and configures Let's Encrypt SSL certificates** for your domain `rpc.webnetcore.top` during deployment. No manual SSL setup required! 🎉

---

## How It Works

### Automated SSL Process

The `deploy-ubuntu-testnet.sh` script handles everything:

1. **Installs Certbot** - Let's Encrypt client
2. **Obtains SSL Certificate** - Uses HTTP-01 challenge (requires temporary Cloudflare proxy disable)
3. **Configures nginx** - Updates nginx to use the certificate
4. **Sets up Auto-Renewal** - Certificates renew automatically every 90 days
5. **Tests HTTPS** - Verifies the certificate is working

---

## Pre-Deployment Requirements

### Step 1: Temporarily Disable Cloudflare Proxy

Before running the deployment script:

1. Log in to [Cloudflare Dashboard](https://dash.cloudflare.com/)
2. Go to **DNS** settings for `webnetcore.top`
3. Find the `rpc` A record pointing to your server IP
4. Click the **orange cloud** ☁️ to turn it **gray** (proxy disabled)
5. Wait **2-3 minutes** for DNS propagation

**Why?** Let's Encrypt needs to verify you own the domain by connecting directly to your server. Cloudflare proxy blocks this verification.

---

## Deployment with Automated SSL

### Full Deployment (Recommended)

```bash
# SSH to your server
ssh user@rpc.webnetcore.top

# Clone or update repository
cd /opt
git clone https://github.com/chikuno/dchat.git
cd dchat

# Make script executable
chmod +x deploy-ubuntu-testnet.sh

# Run deployment (SSL setup included automatically)
sudo ./deploy-ubuntu-testnet.sh
```

The script will:
- ✅ Install Docker, nginx, certbot
- ✅ Build and start all containers
- ✅ **Pause and ask you to disable Cloudflare proxy**
- ✅ Obtain Let's Encrypt certificate for rpc.webnetcore.top
- ✅ Configure nginx for HTTPS
- ✅ Enable auto-renewal
- ✅ Test the certificate

### Skip SSL Setup (Manual Setup Later)

If you want to set up SSL manually later:

```bash
sudo ./deploy-ubuntu-testnet.sh --skip-ssl
```

---

## During Deployment

### What to Expect

When the script reaches the SSL setup phase, you'll see:

```
[2025-10-31 12:34:56] Setting up Let's Encrypt SSL certificate for rpc.webnetcore.top...
[2025-10-31 12:34:57] WARNING: IMPORTANT: For Let's Encrypt to work with Cloudflare:
[2025-10-31 12:34:57] WARNING: 1. Temporarily disable Cloudflare proxy (gray cloud) for rpc.webnetcore.top
[2025-10-31 12:34:57] WARNING: 2. Wait 2-3 minutes for DNS propagation
[2025-10-31 12:34:57] WARNING: 3. Press Enter when ready to continue, or Ctrl+C to skip SSL setup
Press Enter to continue with SSL setup, or Ctrl+C to skip...
```

### Your Action

1. **If you haven't disabled Cloudflare proxy yet**: 
   - Open Cloudflare Dashboard
   - Disable proxy (turn orange cloud to gray)
   - Wait 2-3 minutes
   - Press **Enter**

2. **If you already disabled it**: 
   - Just press **Enter**

3. **If you want to skip SSL for now**: 
   - Press **Ctrl+C** (script will continue without SSL)

---

## After SSL Certificate Obtained

### Automatic Steps

The script will:
1. ✅ Obtain certificate from Let's Encrypt
2. ✅ Update nginx configuration
3. ✅ Reload nginx with HTTPS enabled
4. ✅ Enable auto-renewal timer
5. ✅ Test HTTPS endpoint

You'll see:

```
==========================================
  SSL Certificate Setup Complete!
==========================================
Domain: rpc.webnetcore.top
Certificate: /etc/letsencrypt/live/rpc.webnetcore.top/fullchain.pem
Private Key: /etc/letsencrypt/live/rpc.webnetcore.top/privkey.pem
Expires: Dec 30 12:34:56 2026 GMT

Next Steps:
1. Re-enable Cloudflare proxy (orange cloud) for rpc.webnetcore.top
2. Set Cloudflare SSL mode to 'Full (strict)'
3. Access your site at: https://rpc.webnetcore.top
==========================================
```

### Post-Deployment Steps

#### 1. Re-enable Cloudflare Proxy

1. Go back to Cloudflare Dashboard → DNS
2. Click the **gray cloud** ☁️ next to `rpc` record
3. Turn it **orange** (proxy enabled)

#### 2. Configure Cloudflare SSL Mode

1. Go to **SSL/TLS** → **Overview**
2. Set encryption mode to: **Full (strict)** ⭐
   - This ensures end-to-end encryption with certificate validation
   - Do NOT use "Flexible" (insecure)

#### 3. Enable HTTPS Features

In Cloudflare Dashboard:

**SSL/TLS** → **Edge Certificates**:
- ✅ Always Use HTTPS: **ON**
- ✅ Automatic HTTPS Rewrites: **ON**
- ✅ Minimum TLS Version: **TLS 1.2**

**Speed** → **Optimization**:
- ✅ HTTP/2: **ON**
- ✅ HTTP/3 (with QUIC): **ON**

---

## Verification

### Test HTTPS Endpoint

```bash
# Test health check
curl https://rpc.webnetcore.top/health

# Expected output:
{"status":"healthy","version":"0.1.0","timestamp":"2025-10-31T12:34:56Z"}
```

### Check Certificate Details

```bash
# View certificate expiration
sudo openssl x509 -enddate -noout -in /etc/letsencrypt/live/rpc.webnetcore.top/fullchain.pem

# View full certificate details
sudo openssl x509 -text -noout -in /etc/letsencrypt/live/rpc.webnetcore.top/fullchain.pem
```

### Test SSL Configuration

Visit: https://www.ssllabs.com/ssltest/analyze.html?d=rpc.webnetcore.top

Expected grade: **A** or **A+**

### Check Auto-Renewal

```bash
# Check renewal timer status
sudo systemctl status certbot.timer

# Test renewal (dry run, doesn't actually renew)
sudo certbot renew --dry-run
```

---

## Auto-Renewal

### How It Works

Certbot automatically installs a systemd timer that:
- Checks for certificate renewal **twice daily**
- Renews certificates **30 days before expiration**
- Reloads nginx after successful renewal
- No manual intervention required! 🎉

### Verify Auto-Renewal is Active

```bash
# Check timer is enabled
sudo systemctl status certbot.timer

# View recent renewal attempts
sudo journalctl -u certbot.timer

# List all certificates and expiration dates
sudo certbot certificates
```

### Manual Renewal (if needed)

```bash
# Renew all certificates (if within 30 days of expiration)
sudo certbot renew

# Force renewal (for testing)
sudo certbot renew --force-renewal
```

---

## Configuration Details

### Certificate Locations

```
Certificate: /etc/letsencrypt/live/rpc.webnetcore.top/fullchain.pem
Private Key: /etc/letsencrypt/live/rpc.webnetcore.top/privkey.pem
Chain: /etc/letsencrypt/live/rpc.webnetcore.top/chain.pem
Full Chain: /etc/letsencrypt/live/rpc.webnetcore.top/fullchain.pem
```

### nginx Configuration

File: `/etc/nginx/sites-available/dchat-testnet`

```nginx
server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name rpc.webnetcore.top;

    ssl_certificate /etc/letsencrypt/live/rpc.webnetcore.top/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/rpc.webnetcore.top/privkey.pem;
    
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers 'ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:...';
    # ... rest of config
}
```

### Renewal Hook (optional)

To reload nginx automatically after renewal, certbot creates:

```bash
/etc/letsencrypt/renewal-hooks/deploy/reload-nginx.sh
```

This runs after each successful renewal.

---

## Troubleshooting

### Certificate Obtained But HTTPS Not Working

**Issue**: Certificate obtained successfully, but https://rpc.webnetcore.top shows error

**Solution**:
1. Check Cloudflare proxy is **enabled** (orange cloud)
2. Set Cloudflare SSL mode to **"Full (strict)"**
3. Wait 2-3 minutes for Cloudflare to update
4. Clear browser cache or try incognito mode

### "Challenge Failed" Error During Deployment

**Issue**: Let's Encrypt can't verify domain ownership

**Possible Causes**:
1. Cloudflare proxy still enabled (must be gray cloud)
2. DNS not propagated yet (wait 5 minutes)
3. Firewall blocking port 80
4. Domain not pointing to server IP

**Solution**:
```bash
# Check DNS resolution
nslookup rpc.webnetcore.top

# Check port 80 is accessible
sudo ufw status | grep 80

# Test direct connection
curl http://[your-server-ip]/.well-known/acme-challenge/test

# Re-run deployment with SSL
sudo ./deploy-ubuntu-testnet.sh --skip-docker --skip-build
```

### Certificate Expired / Not Renewing

**Issue**: Certificate expired or renewal failing

**Check renewal logs**:
```bash
sudo journalctl -u certbot.timer -n 50
sudo tail -f /var/log/letsencrypt/letsencrypt.log
```

**Manual renewal**:
```bash
# Stop nginx temporarily
sudo systemctl stop nginx

# Renew certificate
sudo certbot renew --force-renewal

# Start nginx
sudo systemctl start nginx
```

### Want to Switch to Cloudflare Origin Certificate?

If you prefer the 15-year Cloudflare Origin Certificate instead:

1. Follow steps in `CLOUDFLARE_SSL_SETUP.md`
2. Update nginx config to use Cloudflare certificate:
   ```bash
   sudo nano /etc/nginx/sites-available/dchat-testnet
   # Comment out Let's Encrypt lines
   # Uncomment Cloudflare certificate lines
   sudo nginx -t && sudo systemctl reload nginx
   ```

---

## Command Reference

### Deployment Commands

```bash
# Full deployment with SSL
sudo ./deploy-ubuntu-testnet.sh

# Skip SSL setup
sudo ./deploy-ubuntu-testnet.sh --skip-ssl

# Skip Docker and build (quick redeploy)
sudo ./deploy-ubuntu-testnet.sh --skip-docker --skip-build

# Multiple options
sudo ./deploy-ubuntu-testnet.sh --skip-docker --skip-build --skip-ssl
```

### Certificate Management

```bash
# List all certificates
sudo certbot certificates

# Test renewal
sudo certbot renew --dry-run

# Force renewal
sudo certbot renew --force-renewal

# Revoke certificate
sudo certbot revoke --cert-path /etc/letsencrypt/live/rpc.webnetcore.top/cert.pem

# Delete certificate
sudo certbot delete --cert-name rpc.webnetcore.top
```

### nginx Commands

```bash
# Test config
sudo nginx -t

# Reload (without downtime)
sudo systemctl reload nginx

# Restart
sudo systemctl restart nginx

# View logs
sudo tail -f /var/log/nginx/dchat-testnet-access.log
sudo tail -f /var/log/nginx/dchat-testnet-error.log
```

---

## Benefits of Automated Setup

✅ **Zero Manual Work** - Script handles everything  
✅ **Production-Ready** - HTTPS enabled out of the box  
✅ **Auto-Renewal** - Certificates renew automatically  
✅ **Best Practices** - Modern TLS configuration (A+ rating)  
✅ **Cloudflare Compatible** - Works seamlessly with Cloudflare CDN  
✅ **Fast Deployment** - SSL setup takes < 2 minutes  
✅ **Reliable** - Let's Encrypt is trusted by all browsers  

---

## Summary

The `deploy-ubuntu-testnet.sh` script now provides **fully automated Let's Encrypt SSL setup**:

1. Run deployment script
2. Temporarily disable Cloudflare proxy when prompted
3. Script obtains and configures SSL certificate
4. Re-enable Cloudflare proxy
5. Set Cloudflare SSL mode to "Full (strict)"
6. Access your secure site at https://rpc.webnetcore.top

**No manual certificate generation, no config editing, no renewal management needed!** 🚀🔒
