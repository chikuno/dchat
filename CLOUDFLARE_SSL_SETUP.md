# Cloudflare SSL Setup Guide for rpc.webnetcore.top

**Domain**: `rpc.webnetcore.top`  
**DNS**: Cloudflare CDN (Proxy enabled - orange cloud ☁️)  
**Server**: Your Ubuntu server

---

## Overview

Since your domain is already configured in Cloudflare CDN, you have two SSL options:

1. **Cloudflare Origin Certificate** (Recommended ⭐) - Free, easy, Cloudflare-to-server encryption
2. **Let's Encrypt** - Free, publicly trusted, requires renewal every 90 days

---

## Option 1: Cloudflare Origin Certificate (Recommended) ⭐

### Benefits
- ✅ Free and easy to set up
- ✅ Valid for 15 years (no renewals needed)
- ✅ Automatically trusted by Cloudflare
- ✅ Works even with Cloudflare proxy enabled
- ✅ Encrypts traffic between Cloudflare and your server

### Steps

#### 1. Generate Origin Certificate in Cloudflare

1. Log in to [Cloudflare Dashboard](https://dash.cloudflare.com/)
2. Select your domain: `webnetcore.top`
3. Go to **SSL/TLS** → **Origin Server**
4. Click **Create Certificate**
5. Configure:
   - **Private key type**: RSA (2048)
   - **Hostnames**: 
     - `rpc.webnetcore.top`
     - `*.webnetcore.top` (optional, for wildcards)
   - **Certificate Validity**: 15 years
6. Click **Create**
7. Copy both:
   - **Origin Certificate** (save as `cloudflare-origin.crt`)
   - **Private Key** (save as `cloudflare-origin.key`)

#### 2. Upload Certificates to Server

```bash
# SSH to your server
ssh user@rpc.webnetcore.top

# Create certificate directory
sudo mkdir -p /etc/ssl/certs /etc/ssl/private

# Create certificate file
sudo nano /etc/ssl/certs/cloudflare-origin.crt
# Paste the Origin Certificate content, then save (Ctrl+X, Y, Enter)

# Create private key file
sudo nano /etc/ssl/private/cloudflare-origin.key
# Paste the Private Key content, then save (Ctrl+X, Y, Enter)

# Set proper permissions
sudo chmod 644 /etc/ssl/certs/cloudflare-origin.crt
sudo chmod 600 /etc/ssl/private/cloudflare-origin.key
sudo chown root:root /etc/ssl/certs/cloudflare-origin.crt
sudo chown root:root /etc/ssl/private/cloudflare-origin.key
```

#### 3. Configure Cloudflare SSL/TLS Mode

1. Go to **SSL/TLS** → **Overview**
2. Set encryption mode to: **Full (strict)** ⭐
   - This ensures end-to-end encryption with certificate validation
   - Do NOT use "Flexible" (insecure) or "Full" (less secure)

#### 4. Test nginx Configuration

```bash
# Test nginx config
sudo nginx -t

# If OK, reload nginx
sudo systemctl reload nginx
```

#### 5. Verify HTTPS is Working

```bash
# Test from anywhere
curl https://rpc.webnetcore.top/health
curl https://rpc.webnetcore.top/status

# Should return JSON responses
```

✅ **Done!** Your site now has HTTPS encryption for 15 years with no renewals needed.

---

## Option 2: Let's Encrypt SSL (Alternative)

### Benefits
- ✅ Free and publicly trusted
- ✅ Recognized by all browsers
- ✅ Good for non-Cloudflare scenarios

### Limitations
- ⚠️ Requires renewal every 90 days
- ⚠️ Needs Cloudflare proxy temporarily disabled OR DNS verification
- ⚠️ More complex setup with Cloudflare CDN

### Steps

#### 1. Install Certbot

```bash
sudo apt-get update
sudo apt-get install -y certbot python3-certbot-nginx
```

#### 2. Temporarily Disable Cloudflare Proxy (HTTP Challenge)

If using HTTP-01 challenge:
1. Go to Cloudflare DNS settings
2. Click on `rpc.webnetcore.top` record
3. Turn off proxy (gray cloud ☁️)
4. Wait 5 minutes for DNS propagation

#### 3. Obtain Certificate

```bash
# Using nginx plugin (easiest)
sudo certbot --nginx -d rpc.webnetcore.top

# OR using standalone mode
sudo systemctl stop nginx
sudo certbot certonly --standalone -d rpc.webnetcore.top
sudo systemctl start nginx
```

#### 4. Update nginx Config

Edit `/etc/nginx/sites-available/dchat-testnet`:

```nginx
# Comment out Cloudflare certificate lines
# ssl_certificate /etc/ssl/certs/cloudflare-origin.crt;
# ssl_certificate_key /etc/ssl/private/cloudflare-origin.key;

# Uncomment Let's Encrypt lines
ssl_certificate /etc/letsencrypt/live/rpc.webnetcore.top/fullchain.pem;
ssl_certificate_key /etc/letsencrypt/live/rpc.webnetcore.top/privkey.pem;
```

#### 5. Re-enable Cloudflare Proxy

1. Go back to Cloudflare DNS
2. Turn proxy back on (orange cloud ☁️)

#### 6. Set Cloudflare SSL Mode

Set to **Full (strict)**

#### 7. Auto-Renewal

Certbot automatically installs a renewal timer:

```bash
# Check renewal timer
sudo systemctl status certbot.timer

# Test renewal (dry run)
sudo certbot renew --dry-run
```

---

## DNS Verification Method (For Let's Encrypt with Cloudflare Proxy)

If you want to keep Cloudflare proxy enabled during Let's Encrypt setup:

```bash
# Install Cloudflare DNS plugin
sudo apt-get install python3-certbot-dns-cloudflare

# Create Cloudflare API token file
sudo mkdir -p /etc/letsencrypt
sudo nano /etc/letsencrypt/cloudflare.ini
```

Add to `cloudflare.ini`:
```ini
dns_cloudflare_api_token = YOUR_CLOUDFLARE_API_TOKEN
```

Get API token from: [Cloudflare API Tokens](https://dash.cloudflare.com/profile/api-tokens)
- Permission: Zone - DNS - Edit
- Zone Resources: Include - Specific zone - webnetcore.top

```bash
# Secure the file
sudo chmod 600 /etc/letsencrypt/cloudflare.ini

# Obtain certificate using DNS challenge
sudo certbot certonly \
  --dns-cloudflare \
  --dns-cloudflare-credentials /etc/letsencrypt/cloudflare.ini \
  -d rpc.webnetcore.top
```

---

## Cloudflare Settings Checklist

### SSL/TLS Settings
- ✅ **SSL/TLS Encryption Mode**: Full (strict)
- ✅ **Always Use HTTPS**: ON
- ✅ **Automatic HTTPS Rewrites**: ON
- ✅ **Minimum TLS Version**: TLS 1.2

### Security Settings
- ✅ **Security Level**: Medium (or as needed)
- ✅ **Challenge Passage**: 30 seconds
- ✅ **Browser Integrity Check**: ON

### Speed Settings
- ✅ **Auto Minify**: JavaScript, CSS, HTML (all ON)
- ✅ **Brotli**: ON
- ✅ **HTTP/2**: ON
- ✅ **HTTP/3 (with QUIC)**: ON

### Caching
- ✅ **Caching Level**: Standard
- ✅ **Browser Cache TTL**: 4 hours (or as needed)

### Firewall Rules (Optional)
Create firewall rules to protect your endpoints:

```
(http.request.uri.path eq "/prometheus/" or http.request.uri.path eq "/grafana/") and not ip.geoip.country in {"US" "YOUR_COUNTRY"}
Action: Block
```

---

## Testing Your SSL Configuration

### 1. Basic HTTPS Test
```bash
curl -I https://rpc.webnetcore.top/health
```

Expected: `HTTP/2 200` with JSON response

### 2. SSL Labs Test
Visit: [SSL Labs SSL Test](https://www.ssllabs.com/ssltest/analyze.html?d=rpc.webnetcore.top)

Expected grade: A or A+

### 3. Check Certificate
```bash
echo | openssl s_client -connect rpc.webnetcore.top:443 -servername rpc.webnetcore.top 2>/dev/null | openssl x509 -noout -text
```

### 4. Verify Cloudflare is Working
```bash
curl -I https://rpc.webnetcore.top/health | grep -i cf-ray
```

Should see: `cf-ray: [some-id]` (indicates Cloudflare is proxying)

### 5. Test All Endpoints
```bash
# Health check
curl https://rpc.webnetcore.top/health

# Status
curl https://rpc.webnetcore.top/status

# Prometheus (may require auth)
curl https://rpc.webnetcore.top/prometheus/

# Grafana
curl https://rpc.webnetcore.top/grafana/

# Jaeger
curl https://rpc.webnetcore.top/jaeger/
```

---

## Troubleshooting

### Error: "SSL certificate problem"
- Check that certificates are in correct locations
- Verify file permissions (crt: 644, key: 600)
- Ensure Cloudflare SSL mode is "Full (strict)"

### Error: "502 Bad Gateway"
- Check that Docker containers are running and healthy
- Verify nginx upstream targets are correct
- Check nginx error log: `sudo tail -f /var/log/nginx/dchat-testnet-error.log`

### Cloudflare Shows "Error 525: SSL handshake failed"
- Cloudflare can't verify your origin certificate
- Check SSL/TLS mode (should be "Full (strict)" with Cloudflare Origin Cert)
- Or use "Full" mode temporarily (less secure)

### Real IP Not Showing in Logs
- Ensure `set_real_ip_from` directives are in nginx config
- Check that `CF-Connecting-IP` header is present
- Verify logs show real IP: `sudo tail -f /var/log/nginx/dchat-testnet-access.log`

---

## Quick Setup Script

Here's a complete script for Cloudflare Origin Certificate setup:

```bash
#!/bin/bash
# Run on server after getting certificates from Cloudflare

# Create directories
sudo mkdir -p /etc/ssl/certs /etc/ssl/private

# You'll need to manually paste certificate content
echo "Paste Origin Certificate, then press Ctrl+D:"
sudo tee /etc/ssl/certs/cloudflare-origin.crt > /dev/null

echo "Paste Private Key, then press Ctrl+D:"
sudo tee /etc/ssl/private/cloudflare-origin.key > /dev/null

# Set permissions
sudo chmod 644 /etc/ssl/certs/cloudflare-origin.crt
sudo chmod 600 /etc/ssl/private/cloudflare-origin.key
sudo chown root:root /etc/ssl/certs/cloudflare-origin.crt
sudo chown root:root /etc/ssl/private/cloudflare-origin.key

# Test and reload nginx
sudo nginx -t && sudo systemctl reload nginx

echo "✅ SSL certificates installed!"
echo "🔒 Test: curl https://rpc.webnetcore.top/health"
```

---

## Recommended Setup: Cloudflare Origin Certificate

For your deployment, I **strongly recommend Option 1** (Cloudflare Origin Certificate) because:

1. ✅ **Simpler**: No renewal management needed
2. ✅ **15-year validity**: Set it and forget it
3. ✅ **Works seamlessly** with Cloudflare proxy enabled
4. ✅ **Free**: Like Let's Encrypt but easier
5. ✅ **Better for your use case**: Server behind CDN

---

## After SSL Setup

Update Cloudflare to force HTTPS:

1. **SSL/TLS** → **Edge Certificates**
2. Enable **Always Use HTTPS**
3. Enable **Automatic HTTPS Rewrites**
4. Set **Minimum TLS Version** to 1.2

Now all traffic to `rpc.webnetcore.top` will be encrypted end-to-end! 🔒

---

## Access Your Services Securely

After SSL setup, access via:

- **Health Check**: https://rpc.webnetcore.top/health
- **Prometheus**: https://rpc.webnetcore.top/prometheus/
- **Grafana**: https://rpc.webnetcore.top/grafana/
- **Jaeger**: https://rpc.webnetcore.top/jaeger/
- **API Validators**: https://rpc.webnetcore.top/api/validators/
- **API Relays**: https://rpc.webnetcore.top/api/relays/
- **Status**: https://rpc.webnetcore.top/status

All secured with HTTPS! 🚀🔒
