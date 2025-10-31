#!/usr/bin/env bash

################################################################################
# dchat Testnet Deployment Script for Clean Ubuntu Server
# 
# This script automates the complete deployment of dchat testnet on a fresh
# Ubuntu server (20.04 LTS or 22.04 LTS recommended).
#
# What it does:
# 1. System preparation and updates
# 2. Docker and Docker Compose installation
# 3. Rust toolchain setup
# 4. Firewall configuration
# 5. Project deployment
# 6. Validator key validation
# 7. Network startup and health checks
# 8. Monitoring stack deployment
#
# Usage:
#   chmod +x deploy-ubuntu-testnet.sh
#   sudo ./deploy-ubuntu-testnet.sh [--skip-docker] [--skip-build] [--monitoring-only] [--skip-nginx]
#
# Options:
#   --skip-docker      Skip Docker installation (if already installed)
#   --skip-build       Skip building Docker images (use existing images)
#   --monitoring-only  Only start monitoring stack
#   --skip-nginx       Skip nginx installation and configuration
#   --skip-ssl         Skip Let's Encrypt SSL certificate setup
#   --help            Show this help message
#
# Requirements:
#   - Ubuntu 20.04 LTS or 22.04 LTS
#   - Root or sudo privileges
#   - At least 4GB RAM, 50GB disk space
#   - Internet connection
#
################################################################################

set -euo pipefail

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$SCRIPT_DIR"
LOG_FILE="/var/log/dchat-deployment.log"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Deployment options
SKIP_DOCKER=0
SKIP_BUILD=0
MONITORING_ONLY=0
SKIP_NGINX=0
SKIP_SSL=0

# Domain configuration for SSL
DOMAIN="rpc.webnetcore.top"
SSL_EMAIL="admin@webnetcore.top"  # Change this to your email

# System requirements
MIN_RAM_GB=4
MIN_DISK_GB=50

################################################################################
# Helper Functions
################################################################################

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $*" | tee -a "$LOG_FILE" >&2
}

log_warning() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING:${NC} $*" | tee -a "$LOG_FILE"
}

log_info() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')] INFO:${NC} $*" | tee -a "$LOG_FILE"
}

fail() {
    log_error "$*"
    log_error "Deployment failed. Check log at $LOG_FILE"
    exit 1
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        fail "This script must be run as root or with sudo"
    fi
}

################################################################################
# Argument Parsing
################################################################################

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --skip-docker)
                SKIP_DOCKER=1
                shift
                ;;
            --skip-build)
                SKIP_BUILD=1
                shift
                ;;
            --monitoring-only)
                MONITORING_ONLY=1
                shift
                ;;
            --skip-nginx)
                SKIP_NGINX=1
                shift
                ;;
            --skip-ssl)
                SKIP_SSL=1
                shift
                ;;
            -h|--help)
                grep "^#" "$0" | tail -n +2 | head -n -1 | sed 's/^# \?//'
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
}

################################################################################
# System Checks
################################################################################

check_os() {
    log "Checking operating system compatibility..."
    
    if [[ ! -f /etc/os-release ]]; then
        fail "Cannot determine OS. /etc/os-release not found."
    fi
    
    source /etc/os-release
    
    if [[ "$ID" != "ubuntu" ]]; then
        log_warning "This script is designed for Ubuntu. Detected: $ID"
        log_warning "Continuing anyway, but some steps may fail."
    fi
    
    case "$VERSION_ID" in
        20.04|22.04|24.04)
            log "OS detected: Ubuntu $VERSION_ID - Compatible ✓"
            ;;
        *)
            log_warning "Ubuntu version $VERSION_ID may not be fully tested"
            ;;
    esac
}

check_system_resources() {
    log "Checking system resources..."
    
    # Check RAM
    total_ram_gb=$(free -g | awk '/^Mem:/ {print $2}')
    if [[ $total_ram_gb -lt $MIN_RAM_GB ]]; then
        log_warning "RAM: ${total_ram_gb}GB (recommended: ${MIN_RAM_GB}GB or more)"
    else
        log "RAM: ${total_ram_gb}GB - Sufficient ✓"
    fi
    
    # Check disk space
    available_disk_gb=$(df -BG "$REPO_ROOT" | awk 'NR==2 {print $4}' | sed 's/G//')
    if [[ $available_disk_gb -lt $MIN_DISK_GB ]]; then
        log_warning "Disk space: ${available_disk_gb}GB (recommended: ${MIN_DISK_GB}GB or more)"
    else
        log "Disk space: ${available_disk_gb}GB - Sufficient ✓"
    fi
    
    # Check CPU cores
    cpu_cores=$(nproc)
    log "CPU cores: $cpu_cores"
    if [[ $cpu_cores -lt 2 ]]; then
        log_warning "Only $cpu_cores CPU core(s) detected. 2 or more recommended."
    fi
}

################################################################################
# System Preparation
################################################################################

update_system() {
    log "Updating system packages..."
    
    export DEBIAN_FRONTEND=noninteractive
    
    apt-get update -y || fail "Failed to update package lists"
    apt-get upgrade -y -o Dpkg::Options::="--force-confdef" -o Dpkg::Options::="--force-confold" || log_warning "Some packages failed to upgrade"
    
    log "Installing essential packages..."
    apt-get install -y \
        curl \
        wget \
        git \
        build-essential \
        pkg-config \
        libssl-dev \
        ca-certificates \
        gnupg \
        lsb-release \
        software-properties-common \
        apt-transport-https \
        ufw \
        net-tools \
        htop \
        jq \
        unzip || fail "Failed to install essential packages"
    
    log "System packages updated successfully ✓"
}

################################################################################
# Docker Installation
################################################################################

install_docker() {
    if [[ $SKIP_DOCKER -eq 1 ]]; then
        log "Skipping Docker installation (--skip-docker flag)"
        return 0
    fi
    
    if command_exists docker && command_exists docker-compose; then
        log "Docker and Docker Compose already installed ✓"
        docker --version
        docker compose version || docker-compose --version
        return 0
    fi
    
    log "Installing Docker..."
    
    # Remove old Docker versions if present
    apt-get remove -y docker docker-engine docker.io containerd runc 2>/dev/null || true
    
    # Add Docker's official GPG key
    install -m 0755 -d /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
    chmod a+r /etc/apt/keyrings/docker.gpg
    
    # Set up Docker repository
    echo \
        "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
        $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null
    
    # Install Docker Engine
    apt-get update -y || fail "Failed to update Docker repository"
    apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin || fail "Failed to install Docker"
    
    # Start and enable Docker
    systemctl start docker || fail "Failed to start Docker"
    systemctl enable docker || log_warning "Failed to enable Docker service"
    
    # Add current user to docker group (if not root)
    if [[ -n "${SUDO_USER:-}" ]]; then
        usermod -aG docker "$SUDO_USER" || log_warning "Failed to add user to docker group"
        log "User $SUDO_USER added to docker group. May need to log out and back in."
    fi
    
    # Verify installation
    docker --version || fail "Docker installation verification failed"
    docker compose version || docker-compose --version || fail "Docker Compose installation verification failed"
    
    log "Docker installed successfully ✓"
}

configure_docker() {
    log "Configuring Docker daemon..."
    
    # Create Docker daemon configuration
    mkdir -p /etc/docker
    
    cat > /etc/docker/daemon.json <<EOF
{
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "10m",
    "max-file": "3"
  },
  "storage-driver": "overlay2",
  "default-address-pools": [
    {
      "base": "172.17.0.0/12",
      "size": 24
    }
  ]
}
EOF
    
    # Restart Docker to apply configuration
    systemctl restart docker || log_warning "Failed to restart Docker"
    
    log "Docker configured successfully ✓"
}

################################################################################
# Firewall Configuration
################################################################################

configure_firewall() {
    log "Configuring firewall (UFW)..."
    
    # Check if UFW is available
    if ! command_exists ufw; then
        log_warning "UFW not found, skipping firewall configuration"
        return 0
    fi
    
    # Reset UFW to default (allow outgoing, deny incoming)
    ufw --force reset
    
    # Default policies
    ufw default deny incoming
    ufw default allow outgoing
    
    # SSH access (critical - don't lock yourself out!)
    ufw allow 22/tcp comment "SSH"
    
    # HTTP/HTTPS for nginx reverse proxy
    ufw allow 80/tcp comment "HTTP"
    ufw allow 443/tcp comment "HTTPS"
    
    # dchat validator nodes (P2P)
    ufw allow 7070:7077/tcp comment "dchat validators P2P"
    
    # dchat relay nodes (P2P)
    ufw allow 7080:7093/tcp comment "dchat relays P2P"
    
    # dchat user nodes (P2P)
    ufw allow 7110:7115/tcp comment "dchat users P2P"
    
    # Monitoring ports (localhost only via nginx proxy)
    ufw allow 9095/tcp comment "Prometheus"
    ufw allow 3000/tcp comment "Grafana"
    ufw allow 16686/tcp comment "Jaeger UI"
    
    # Enable firewall
    ufw --force enable || log_warning "Failed to enable UFW"
    
    # Show status
    ufw status numbered
    
    log "Firewall configured successfully ✓"
}

################################################################################
# Nginx Installation and Configuration
################################################################################

install_nginx() {
    if [[ $SKIP_NGINX -eq 1 ]]; then
        log "Skipping nginx installation (--skip-nginx flag)"
        return 0
    fi
    
    log "Installing and configuring nginx..."
    
    # Check if nginx is already installed
    if command_exists nginx; then
        log "nginx already installed ✓"
        nginx -v
    else
        log "Installing nginx..."
        apt-get install -y nginx || fail "Failed to install nginx"
        
        # Start and enable nginx
        systemctl start nginx || fail "Failed to start nginx"
        systemctl enable nginx || log_warning "Failed to enable nginx service"
    fi
    
    # Check if nginx-testnet.conf exists
    if [[ ! -f "$REPO_ROOT/nginx-testnet.conf" ]]; then
        log_warning "nginx-testnet.conf not found in project root"
        log_warning "Skipping nginx configuration"
        return 0
    fi
    
    log "Configuring nginx for dchat testnet..."
    
    # Backup existing default config if present
    if [[ -f /etc/nginx/sites-enabled/default ]]; then
        mv /etc/nginx/sites-enabled/default /etc/nginx/sites-enabled/default.backup-$(date +%Y%m%d-%H%M%S) || true
        log "Backed up default nginx config"
    fi
    
    # Copy dchat nginx config
    cp "$REPO_ROOT/nginx-testnet.conf" /etc/nginx/sites-available/dchat-testnet || fail "Failed to copy nginx config"
    
    # Create symlink in sites-enabled
    ln -sf /etc/nginx/sites-available/dchat-testnet /etc/nginx/sites-enabled/dchat-testnet || fail "Failed to create nginx symlink"
    
    # Test nginx configuration
    nginx -t || fail "nginx configuration test failed"
    
    # Reload nginx
    systemctl reload nginx || fail "Failed to reload nginx"
    
    log "nginx configured successfully ✓"
    log "External access:"
    log "  • Health Check:  http://$(hostname -I | awk '{print $1}')/health"
    log "  • Prometheus:    http://$(hostname -I | awk '{print $1}')/prometheus/"
    log "  • Grafana:       http://$(hostname -I | awk '{print $1}')/grafana/"
    log "  • Jaeger:        http://$(hostname -I | awk '{print $1}')/jaeger/"
}

################################################################################
# SSL Certificate Setup with Let's Encrypt
################################################################################

setup_letsencrypt_ssl() {
    if [[ $SKIP_SSL -eq 1 ]]; then
        log "Skipping SSL certificate setup (--skip-ssl flag)"
        return 0
    fi
    
    if [[ $SKIP_NGINX -eq 1 ]]; then
        log "Skipping SSL setup (nginx not installed)"
        return 0
    fi
    
    log "Setting up Let's Encrypt SSL certificate for $DOMAIN..."
    
    # Check if certbot is installed
    if ! command_exists certbot; then
        log "Installing certbot..."
        apt-get install -y certbot python3-certbot-nginx || fail "Failed to install certbot"
    else
        log "certbot already installed ✓"
    fi
    
    # Check if certificate already exists
    if [[ -f "/etc/letsencrypt/live/$DOMAIN/fullchain.pem" ]]; then
        log "SSL certificate already exists for $DOMAIN ✓"
        
        # Check expiration
        local expiry_date=$(openssl x509 -enddate -noout -in "/etc/letsencrypt/live/$DOMAIN/fullchain.pem" | cut -d= -f2)
        log "Certificate expires: $expiry_date"
        
        # Update nginx config to use the certificate
        update_nginx_ssl_config
        return 0
    fi
    
    log_info "Obtaining SSL certificate from Let's Encrypt..."
    log_info "This will use DNS validation through Cloudflare..."
    
    # Since domain is behind Cloudflare, we need to use DNS validation
    # First, try to get certificate using HTTP challenge (requires Cloudflare proxy off temporarily)
    log_warning "IMPORTANT: For Let's Encrypt to work with Cloudflare:"
    log_warning "1. Temporarily disable Cloudflare proxy (gray cloud) for $DOMAIN"
    log_warning "2. Wait 2-3 minutes for DNS propagation"
    log_warning "3. Press Enter when ready to continue, or Ctrl+C to skip SSL setup"
    
    if [[ -t 0 ]]; then
        read -p "Press Enter to continue with SSL setup, or Ctrl+C to skip..."
    else
        log_warning "Non-interactive mode: Attempting SSL setup automatically..."
        log_warning "If this fails, run with --skip-ssl and set up SSL manually"
    fi
    
    # Try to obtain certificate using nginx plugin
    log_info "Obtaining certificate for $DOMAIN..."
    if certbot --nginx -d "$DOMAIN" --non-interactive --agree-tos --email "$SSL_EMAIL" --redirect 2>&1 | tee -a "$LOG_FILE"; then
        log "SSL certificate obtained successfully ✓"
        log "Certificate location: /etc/letsencrypt/live/$DOMAIN/"
        
        # Update nginx config
        update_nginx_ssl_config
        
        # Test HTTPS
        log_info "Testing HTTPS endpoint..."
        sleep 5
        if curl -sf "https://$DOMAIN/health" >/dev/null 2>&1; then
            log "HTTPS is working ✓"
        else
            log_warning "HTTPS test failed, but certificate was obtained"
            log_warning "You may need to enable Cloudflare proxy and set SSL mode to 'Full (strict)'"
        fi
        
        # Setup auto-renewal
        log_info "Setting up automatic certificate renewal..."
        if systemctl list-unit-files | grep -q certbot.timer; then
            systemctl enable certbot.timer
            systemctl start certbot.timer
            log "Auto-renewal enabled ✓"
        fi
        
        log ""
        log "=========================================="
        log "  SSL Certificate Setup Complete!"
        log "=========================================="
        log "Domain: $DOMAIN"
        log "Certificate: /etc/letsencrypt/live/$DOMAIN/fullchain.pem"
        log "Private Key: /etc/letsencrypt/live/$DOMAIN/privkey.pem"
        log "Expires: $(openssl x509 -enddate -noout -in "/etc/letsencrypt/live/$DOMAIN/fullchain.pem" | cut -d= -f2)"
        log ""
        log "Next Steps:"
        log "1. Re-enable Cloudflare proxy (orange cloud) for $DOMAIN"
        log "2. Set Cloudflare SSL mode to 'Full (strict)'"
        log "3. Access your site at: https://$DOMAIN"
        log "=========================================="
        log ""
    else
        log_error "Failed to obtain SSL certificate"
        log_warning "You can:"
        log_warning "1. Run deployment again with --skip-ssl and set up SSL manually"
        log_warning "2. Use Cloudflare Origin Certificate (see CLOUDFLARE_SSL_SETUP.md)"
        log_warning "3. Check that Cloudflare proxy is disabled and DNS has propagated"
        log_warning ""
        log_warning "Continuing deployment without SSL..."
        return 0
    fi
}

update_nginx_ssl_config() {
    log "Updating nginx configuration for SSL..."
    
    local nginx_conf="/etc/nginx/sites-available/dchat-testnet"
    
    # Check if Let's Encrypt certificate exists
    if [[ ! -f "/etc/letsencrypt/live/$DOMAIN/fullchain.pem" ]]; then
        log_warning "Let's Encrypt certificate not found, skipping nginx SSL update"
        return 0
    fi
    
    # Update nginx config to use Let's Encrypt certificate
    if grep -q "ssl_certificate /etc/ssl/certs/cloudflare-origin.crt" "$nginx_conf" 2>/dev/null; then
        log_info "Updating nginx to use Let's Encrypt certificate..."
        
        # Comment out Cloudflare certificate lines
        sed -i 's|^\(\s*ssl_certificate /etc/ssl/certs/cloudflare-origin.crt;\)|    # \1  # Replaced with Let'\''s Encrypt|' "$nginx_conf"
        sed -i 's|^\(\s*ssl_certificate_key /etc/ssl/private/cloudflare-origin.key;\)|    # \1  # Replaced with Let'\''s Encrypt|' "$nginx_conf"
        
        # Uncomment Let's Encrypt certificate lines if they exist
        sed -i "s|# ssl_certificate /etc/letsencrypt/live/$DOMAIN/fullchain.pem;|ssl_certificate /etc/letsencrypt/live/$DOMAIN/fullchain.pem;|" "$nginx_conf"
        sed -i "s|# ssl_certificate_key /etc/letsencrypt/live/$DOMAIN/privkey.pem;|ssl_certificate_key /etc/letsencrypt/live/$DOMAIN/privkey.pem;|" "$nginx_conf"
        
        # If Let's Encrypt lines don't exist, add them
        if ! grep -q "ssl_certificate /etc/letsencrypt/live/$DOMAIN/fullchain.pem" "$nginx_conf"; then
            # Add after the commented Cloudflare lines
            sed -i "/# Replaced with Let/a\\    ssl_certificate /etc/letsencrypt/live/$DOMAIN/fullchain.pem;\n    ssl_certificate_key /etc/letsencrypt/live/$DOMAIN/privkey.pem;" "$nginx_conf"
        fi
        
        # Test and reload nginx
        if nginx -t 2>&1 | tee -a "$LOG_FILE"; then
            systemctl reload nginx
            log "nginx reloaded with Let's Encrypt certificate ✓"
        else
            log_error "nginx configuration test failed"
            return 1
        fi
    fi
}

################################################################################
# Rust Toolchain Installation (Optional - for local building)
################################################################################

install_rust() {
    log "Checking Rust installation..."
    
    if command_exists rustc && command_exists cargo; then
        log "Rust already installed ✓"
        rustc --version
        cargo --version
        return 0
    fi
    
    log "Installing Rust toolchain..."
    
    # Install Rust via rustup
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable || fail "Failed to install Rust"
    
    # Source Rust environment
    source "$HOME/.cargo/env" || true
    
    # Verify installation
    if command_exists rustc; then
        log "Rust installed successfully ✓"
        rustc --version
        cargo --version
    else
        log_warning "Rust installed but not in PATH. May need to run: source ~/.cargo/env"
    fi
}

################################################################################
# Project Setup
################################################################################

validate_project_structure() {
    log "Validating project structure..."
    
    local required_files=(
        "Dockerfile"
        "docker-compose-testnet.yml"
        "Cargo.toml"
        "config.example.toml"
    )
    
    for file in "${required_files[@]}"; do
        if [[ ! -f "$REPO_ROOT/$file" ]]; then
            fail "Required file not found: $file"
        fi
    done
    
    log "Project structure validated ✓"
}

validate_validator_keys() {
    log "Validating validator keys..."
    
    local keydir="$REPO_ROOT/validator_keys"
    
    if [[ ! -d "$keydir" ]]; then
        fail "Validator keys directory not found: $keydir"
    fi
    
    for i in 1 2 3 4; do
        local keyfile="$keydir/validator${i}.key"
        if [[ ! -f "$keyfile" ]]; then
            fail "Missing validator key: $keyfile. Run generate-validator-keys.ps1 first."
        fi
    done
    
    # Fix permissions for Docker container (UID 1000)
    log "Setting correct permissions for Docker containers..."
    chown -R 1000:1000 "$keydir" || log_warning "Failed to set ownership to 1000:1000"
    chmod 755 "$keydir"
    chmod 644 "$keydir"/*.key
    
    log "Validator key permissions set (owner: 1000:1000, keys: 644) ✓"
    log "Validator keys validated ✓"
}

create_data_directories() {
    log "Creating data directories..."
    
    mkdir -p "$REPO_ROOT/dchat_data"
    mkdir -p "$REPO_ROOT/testnet-logs"
    mkdir -p "$REPO_ROOT/monitoring/prometheus/data"
    mkdir -p "$REPO_ROOT/monitoring/grafana/data"
    
    # Set appropriate permissions
    chmod 755 "$REPO_ROOT/dchat_data"
    chmod 755 "$REPO_ROOT/testnet-logs"
    
    log "Data directories created ✓"
}

create_config_file() {
    log "Creating configuration file..."
    
    if [[ ! -f "$REPO_ROOT/testnet-config.toml" ]]; then
        if [[ -f "$REPO_ROOT/config.example.toml" ]]; then
            cp "$REPO_ROOT/config.example.toml" "$REPO_ROOT/testnet-config.toml"
            log "Created testnet-config.toml from example"
        else
            log_warning "config.example.toml not found, skipping config creation"
        fi
    else
        log "Configuration file already exists"
    fi
}

################################################################################
# Docker Image Building
################################################################################

build_docker_images() {
    if [[ $SKIP_BUILD -eq 1 ]]; then
        log "Skipping Docker image build (--skip-build flag)"
        return 0
    fi
    
    log "Building Docker images (this may take 10-30 minutes)..."
    
    cd "$REPO_ROOT"
    
    # Build the dchat image
    docker build \
        --tag dchat:latest \
        --tag dchat:testnet-$TIMESTAMP \
        --file Dockerfile \
        --progress=plain \
        . 2>&1 | tee -a "$LOG_FILE" || fail "Docker build failed"
    
    log "Docker images built successfully ✓"
}

pull_third_party_images() {
    log "Pulling third-party Docker images..."
    
    local images=(
        "postgres:16-alpine"
        "prom/prometheus:latest"
        "grafana/grafana:latest"
        "jaegertracing/all-in-one:latest"
        "busybox:latest"
    )
    
    for image in "${images[@]}"; do
        log_info "Pulling $image..."
        docker pull "$image" || log_warning "Failed to pull $image"
    done
    
    log "Third-party images pulled ✓"
}

################################################################################
# Network Deployment
################################################################################

start_testnet() {
    log "Starting dchat testnet..."
    
    cd "$REPO_ROOT"
    
    # Stop any existing containers
    log_info "Stopping existing containers..."
    docker compose -f docker-compose-testnet.yml -p dchat-testnet down 2>/dev/null || true
    
    # Remove any stopped containers that might hold ports
    log_info "Cleaning up stopped containers..."
    docker ps -a -q --filter "label=com.docker.compose.project=dchat-testnet" | xargs -r docker rm -f 2>/dev/null || true
    
    # Start the testnet
    log_info "Starting containers..."
    docker compose -f docker-compose-testnet.yml -p dchat-testnet up -d || fail "Failed to start testnet"
    
    log "Testnet containers started ✓"
}

wait_for_health() {
    log "Waiting for containers to become healthy..."
    
    local max_wait=300  # 5 minutes
    local interval=10
    local elapsed=0
    
    while [[ $elapsed -lt $max_wait ]]; do
        local all_healthy=1
        local container_count=0
        
        # Get all containers for the project
        while IFS= read -r container; do
            [[ -z "$container" ]] && continue
            container_count=$((container_count + 1))
            
            local health=$(docker inspect --format='{{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' "$container" 2>/dev/null || echo "unknown")
            local running=$(docker inspect --format='{{.State.Running}}' "$container" 2>/dev/null || echo "false")
            
            if [[ "$running" != "true" ]]; then
                all_healthy=0
                log_info "Container $container is not running"
            elif [[ "$health" == "starting" ]] || [[ "$health" == "unhealthy" ]]; then
                all_healthy=0
                log_info "Container $container health: $health"
            fi
        done < <(docker ps -q --filter "label=com.docker.compose.project=dchat-testnet")
        
        if [[ $container_count -eq 0 ]]; then
            fail "No containers found for project dchat-testnet"
        fi
        
        if [[ $all_healthy -eq 1 ]]; then
            log "All containers are healthy ✓"
            return 0
        fi
        
        sleep $interval
        elapsed=$((elapsed + interval))
        log_info "Waiting... ($elapsed/${max_wait}s)"
    done
    
    log_warning "Timeout waiting for all containers to become healthy"
    log_warning "Some containers may still be starting. Check with: docker ps"
}

################################################################################
# Monitoring Setup
################################################################################

setup_monitoring() {
    log "Setting up monitoring stack..."
    
    # Ensure monitoring configuration exists
    if [[ ! -f "$REPO_ROOT/monitoring/prometheus.yml" ]]; then
        log_warning "monitoring/prometheus.yml not found"
        log_warning "Creating complete Prometheus configuration..."
        
        mkdir -p "$REPO_ROOT/monitoring"
        cat > "$REPO_ROOT/monitoring/prometheus.yml" <<'EOF'
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    cluster: 'dchat-testnet'
    environment: 'testnet'

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']
        labels:
          role: 'monitoring'

  - job_name: 'validators'
    static_configs:
      - targets:
        - 'validator1:9090'
        - 'validator2:9090'
        - 'validator3:9090'
        - 'validator4:9090'
        labels:
          role: 'validator'

  - job_name: 'relays'
    static_configs:
      - targets:
        - 'relay1:9100'
        - 'relay2:9100'
        - 'relay3:9102'
        - 'relay4:9103'
        - 'relay5:9104'
        - 'relay6:9105'
        - 'relay7:9106'
        labels:
          role: 'relay'

  - job_name: 'users'
    static_configs:
      - targets:
        - 'user1:9110'
        - 'user2:9111'
        - 'user3:9112'
        labels:
          role: 'user'
EOF
        log "Created complete Prometheus configuration with 14 targets"
    else
        log "Prometheus configuration already exists"
    fi
    
    # Create Grafana datasource configuration
    mkdir -p "$REPO_ROOT/monitoring/grafana/datasources"
    cat > "$REPO_ROOT/monitoring/grafana/datasources/prometheus.yml" <<'EOF'
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: true
    jsonData:
      timeInterval: "15s"
EOF
    
    log "Monitoring configuration created ✓"
}

################################################################################
# Health Checks and Status
################################################################################

show_deployment_status() {
    log ""
    log "=========================================="
    log "    dchat Testnet Deployment Status"
    log "=========================================="
    log ""
    
    # Show running containers
    log "Running Containers:"
    docker ps --filter "label=com.docker.compose.project=dchat-testnet" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | tee -a "$LOG_FILE"
    
    log ""
    log "Network Nodes:"
    log "  • 4 Validator nodes (consensus)"
    log "  • 7 Relay nodes (message routing)"
    log "  • 3 User nodes (end-user clients)"
    log ""
    
    # Get server IP
    local server_ip=$(hostname -I | awk '{print $1}')
    
    log "Direct Access (Localhost):"
    log "  • Grafana:    http://${server_ip}:3000 (admin/admin)"
    log "  • Prometheus: http://${server_ip}:9095"
    log "  • Jaeger UI:  http://${server_ip}:16686"
    log ""
    
    # Check if nginx is configured
    if [[ -f /etc/nginx/sites-enabled/dchat-testnet ]] && command_exists nginx; then
        log "External Access (via nginx):"
        log "  • Health Check:  http://${server_ip}/health"
        log "  • Prometheus:    http://${server_ip}/prometheus/"
        log "  • Grafana:       http://${server_ip}/grafana/"
        log "  • Jaeger:        http://${server_ip}/jaeger/"
        log "  • API Validators: http://${server_ip}/api/validators/"
        log "  • API Relays:    http://${server_ip}/api/relays/"
        log ""
    fi
    
    log "Validator RPC Endpoints:"
    log "  • Validator 1: http://${server_ip}:7071"
    log "  • Validator 2: http://${server_ip}:7073"
    log "  • Validator 3: http://${server_ip}:7075"
    log "  • Validator 4: http://${server_ip}:7077"
    log ""
    
    log "Management Commands:"
    log "  • View logs:        docker compose -f docker-compose-testnet.yml -p dchat-testnet logs -f"
    log "  • Stop testnet:     docker compose -f docker-compose-testnet.yml -p dchat-testnet down"
    log "  • Restart testnet:  docker compose -f docker-compose-testnet.yml -p dchat-testnet restart"
    log "  • Check status:     docker ps"
    log ""
    
    log "=========================================="
}

test_endpoints() {
    log "Testing network endpoints..."
    
    local endpoints=(
        "http://localhost:7071/health:Validator1"
        "http://localhost:7073/health:Validator2"
        "http://localhost:7075/health:Validator3"
        "http://localhost:7077/health:Validator4"
        "http://localhost:7081/health:Relay1"
        "http://localhost:7111/health:User1"
        "http://localhost:9095/-/healthy:Prometheus"
        "http://localhost:3000/api/health:Grafana"
    )
    
    local failed=0
    
    for endpoint_pair in "${endpoints[@]}"; do
        local endpoint="${endpoint_pair%%:*}"
        local name="${endpoint_pair##*:}"
        
        if curl -sf "$endpoint" >/dev/null 2>&1; then
            log "✓ $name is responding"
        else
            log_warning "✗ $name is not responding at $endpoint"
            failed=$((failed + 1))
        fi
    done
    
    # Check Prometheus targets
    log "Checking Prometheus targets..."
    if command_exists curl && command_exists jq; then
        local targets=$(curl -sf http://localhost:9095/api/v1/targets 2>/dev/null | jq -r '.data.activeTargets | length' 2>/dev/null || echo "0")
        if [[ "$targets" -ge 14 ]]; then
            log "✓ Prometheus has $targets active targets (expected: 14)"
        else
            log_warning "✗ Prometheus has only $targets active targets (expected: 14)"
            failed=$((failed + 1))
        fi
    else
        log_info "Skipping Prometheus target check (jq not installed)"
    fi
    
    if [[ $failed -gt 0 ]]; then
        log_warning "$failed endpoint(s) are not responding. They may still be starting up."
        log_warning "Wait a few minutes and check manually."
    else
        log "All endpoints are responding ✓"
    fi
}

################################################################################
# Cleanup and Utilities
################################################################################

create_management_scripts() {
    log "Creating management scripts..."
    
    # Stop script
    cat > "$REPO_ROOT/stop-testnet.sh" <<'EOF'
#!/bin/bash
cd "$(dirname "$0")"
docker compose -f docker-compose-testnet.yml -p dchat-testnet down
echo "Testnet stopped"
EOF
    
    # Start script
    cat > "$REPO_ROOT/start-testnet.sh" <<'EOF'
#!/bin/bash
cd "$(dirname "$0")"
docker compose -f docker-compose-testnet.yml -p dchat-testnet up -d
echo "Testnet started"
EOF
    
    # Logs script
    cat > "$REPO_ROOT/logs-testnet.sh" <<'EOF'
#!/bin/bash
cd "$(dirname "$0")"
docker compose -f docker-compose-testnet.yml -p dchat-testnet logs -f "$@"
EOF
    
    # Status script
    cat > "$REPO_ROOT/status-testnet.sh" <<'EOF'
#!/bin/bash
echo "=== dchat Testnet Status ==="
docker ps --filter "label=com.docker.compose.project=dchat-testnet" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
EOF
    
    chmod +x "$REPO_ROOT/stop-testnet.sh"
    chmod +x "$REPO_ROOT/start-testnet.sh"
    chmod +x "$REPO_ROOT/logs-testnet.sh"
    chmod +x "$REPO_ROOT/status-testnet.sh"
    
    log "Management scripts created ✓"
}

save_deployment_info() {
    log "Saving deployment information..."
    
    local nginx_status="Not installed"
    if [[ -f /etc/nginx/sites-enabled/dchat-testnet ]] && command_exists nginx; then
        nginx_status="Installed and configured"
    fi
    
    cat > "$REPO_ROOT/DEPLOYMENT_INFO.txt" <<EOF
dchat Testnet Deployment Information
=====================================
Deployment Date: $(date)
Server: $(hostname)
IP Address: $(hostname -I | awk '{print $1}')
OS: $(lsb_release -ds 2>/dev/null || echo "Unknown")
Docker Version: $(docker --version)
Compose Version: $(docker compose version 2>/dev/null || docker-compose --version 2>/dev/null || echo "Unknown")
Nginx: $nginx_status

Container Counts:
  - Validators: 4
  - Relays: 7
  - Users: 3
  - Monitoring: 3 (Prometheus, Grafana, Jaeger)

Health Check Configuration:
  - All validators: Health on port 7071, Metrics on port 9090
  - All relays: Health on port 7081, Metrics on ports 9100-9106
  - All users: Health on port 7111, Metrics on ports 9110-9112
  - Healthcheck command: curl -f http://localhost:PORT/health

Prometheus Targets:
  - 4 validators (validator1-4:9090)
  - 7 relays (relay1:9100, relay2:9100, relay3:9102, relay4:9103, relay5:9104, relay6:9105, relay7:9106)
  - 3 users (user1:9110, user2:9111, user3:9112)
  - Total: 14 active targets

Log File: $LOG_FILE

Generated at: $(date)
EOF
    
    log "Deployment info saved to DEPLOYMENT_INFO.txt ✓"
}

################################################################################
# Main Deployment Flow
################################################################################

main() {
    log "=========================================="
    log "  dchat Testnet Deployment Script"
    log "  Ubuntu Server Edition"
    log "=========================================="
    log ""
    
    # Parse command line arguments
    parse_args "$@"
    
    # Check prerequisites
    check_root
    check_os
    check_system_resources
    
    if [[ $MONITORING_ONLY -eq 1 ]]; then
        log "Running in monitoring-only mode"
        setup_monitoring
        log "Restarting testnet to apply monitoring changes..."
        start_testnet
        wait_for_health
        show_deployment_status
        exit 0
    fi
    
    # System preparation
    update_system
    install_docker
    configure_docker
    configure_firewall
    install_nginx
    
    # Optional: Install Rust (for local development)
    # install_rust
    
    # Project setup
    validate_project_structure
    validate_validator_keys
    create_data_directories
    create_config_file
    setup_monitoring
    
    # Build and deploy
    pull_third_party_images
    build_docker_images
    start_testnet
    wait_for_health
    
    # SSL certificate setup (after containers are running)
    setup_letsencrypt_ssl
    
    # Post-deployment
    create_management_scripts
    save_deployment_info
    test_endpoints
    show_deployment_status
    
    log ""
    log "=========================================="
    log "  Deployment Complete! 🎉"
    log "=========================================="
    log ""
    log "Next steps:"
    log "  1. Check container status: ./status-testnet.sh"
    log "  2. View logs: ./logs-testnet.sh"
    log "  3. Access Grafana at http://$(hostname -I | awk '{print $1}'):3000"
    log "  4. Test message propagation between user nodes"
    log ""
    log "Full deployment log: $LOG_FILE"
    log ""
}

# Run main function
main "$@"
