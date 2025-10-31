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
#   sudo ./deploy-ubuntu-testnet.sh [--skip-docker] [--skip-build] [--monitoring-only]
#
# Options:
#   --skip-docker      Skip Docker installation (if already installed)
#   --skip-build       Skip building Docker images (use existing images)
#   --monitoring-only  Only start monitoring stack
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
    
    # dchat validator nodes (P2P)
    ufw allow 7070:7077/tcp comment "dchat validators P2P"
    
    # dchat relay nodes (P2P)
    ufw allow 7080:7093/tcp comment "dchat relays P2P"
    
    # dchat user nodes (P2P)
    ufw allow 7110:7115/tcp comment "dchat users P2P"
    
    # Monitoring ports
    ufw allow 9090/tcp comment "Prometheus"
    ufw allow 3000/tcp comment "Grafana"
    ufw allow 16686/tcp comment "Jaeger UI"
    
    # Enable firewall
    ufw --force enable || log_warning "Failed to enable UFW"
    
    # Show status
    ufw status numbered
    
    log "Firewall configured successfully ✓"
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
    
    # Free port 9090 if occupied
    log_info "Checking port 9090 availability..."
    local port_9090_pid=$(lsof -ti :9090 2>/dev/null)
    if [[ -n "$port_9090_pid" ]]; then
        log_warning "Port 9090 in use by PID $port_9090_pid, killing..."
        kill -9 $port_9090_pid 2>/dev/null || true
        sleep 2
    fi
    fuser -k 9090/tcp 2>/dev/null || true
    log "Port 9090 freed ✓"
    
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
        log_warning "Creating basic Prometheus configuration..."
        
        mkdir -p "$REPO_ROOT/monitoring"
        cat > "$REPO_ROOT/monitoring/prometheus.yml" <<'EOF'
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'dchat-validators'
    static_configs:
      - targets:
        - 'validator1:9090'
        - 'validator2:9090'
        - 'validator3:9090'
        - 'validator4:9090'

  - job_name: 'dchat-relays'
    static_configs:
      - targets:
        - 'relay1:9100'
        - 'relay2:9100'
        - 'relay3:9100'
        - 'relay4:9100'
        - 'relay5:9100'
        - 'relay6:9100'
        - 'relay7:9100'

  - job_name: 'dchat-users'
    static_configs:
      - targets:
        - 'user1:9110'
        - 'user2:9110'
        - 'user3:9110'
EOF
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
    
    log "Access Points:"
    log "  • Grafana:    http://${server_ip}:3000 (admin/admin)"
    log "  • Prometheus: http://${server_ip}:9090"
    log "  • Jaeger UI:  http://${server_ip}:16686"
    log ""
    
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
        "http://localhost:9090/-/healthy:Prometheus"
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
    
    cat > "$REPO_ROOT/DEPLOYMENT_INFO.txt" <<EOF
dchat Testnet Deployment Information
=====================================
Deployment Date: $(date)
Server: $(hostname)
IP Address: $(hostname -I | awk '{print $1}')
OS: $(lsb_release -ds 2>/dev/null || echo "Unknown")
Docker Version: $(docker --version)
Compose Version: $(docker compose version 2>/dev/null || docker-compose --version 2>/dev/null || echo "Unknown")

Container Counts:
  - Validators: 4
  - Relays: 7
  - Users: 3
  - Monitoring: 3 (Prometheus, Grafana, Jaeger)

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
