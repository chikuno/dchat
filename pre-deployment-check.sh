#!/usr/bin/env bash

################################################################################
# dchat Pre-Deployment Validation Script
# 
# Run this script BEFORE deploying to verify all prerequisites are met
# This catches common issues early and saves deployment time
#
# Usage: ./pre-deployment-check.sh
################################################################################

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Counters
CHECKS_PASSED=0
CHECKS_FAILED=0
CHECKS_WARNING=0

################################################################################
# Helper Functions
################################################################################

print_header() {
    echo -e "${BLUE}"
    echo "=========================================="
    echo "  dchat Pre-Deployment Validation"
    echo "=========================================="
    echo -e "${NC}"
}

check_pass() {
    echo -e "${GREEN}✓${NC} $1"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
}

check_fail() {
    echo -e "${RED}✗${NC} $1"
    CHECKS_FAILED=$((CHECKS_FAILED + 1))
}

check_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
    CHECKS_WARNING=$((CHECKS_WARNING + 1))
}

section() {
    echo ""
    echo -e "${BLUE}[$1]${NC}"
}

################################################################################
# Validation Checks
################################################################################

check_os() {
    section "Operating System"
    
    if [[ -f /etc/os-release ]]; then
        source /etc/os-release
        echo "  OS: $PRETTY_NAME"
        
        if [[ "$ID" == "ubuntu" ]]; then
            case "$VERSION_ID" in
                20.04|22.04|24.04)
                    check_pass "Ubuntu $VERSION_ID is supported"
                    ;;
                *)
                    check_warn "Ubuntu $VERSION_ID may not be tested"
                    ;;
            esac
        else
            check_warn "Non-Ubuntu OS detected: $ID"
        fi
    else
        check_fail "Cannot determine OS version"
    fi
    
    # Check if running as root
    if [[ $EUID -eq 0 ]]; then
        check_pass "Running with root privileges"
    else
        check_warn "Not running as root. Will need sudo for deployment"
    fi
}

check_resources() {
    section "System Resources"
    
    # RAM check
    total_ram_gb=$(free -g | awk '/^Mem:/ {print $2}')
    echo "  RAM: ${total_ram_gb}GB"
    if [[ $total_ram_gb -ge 4 ]]; then
        check_pass "Sufficient RAM ($total_ram_gb GB >= 4 GB)"
    elif [[ $total_ram_gb -ge 2 ]]; then
        check_warn "Low RAM ($total_ram_gb GB). 4GB+ recommended"
    else
        check_fail "Insufficient RAM ($total_ram_gb GB < 2 GB minimum)"
    fi
    
    # Disk space check
    available_disk_gb=$(df -BG "$SCRIPT_DIR" | awk 'NR==2 {print $4}' | sed 's/G//')
    echo "  Disk Space: ${available_disk_gb}GB available"
    if [[ $available_disk_gb -ge 50 ]]; then
        check_pass "Sufficient disk space ($available_disk_gb GB >= 50 GB)"
    elif [[ $available_disk_gb -ge 30 ]]; then
        check_warn "Limited disk space ($available_disk_gb GB). 50GB+ recommended"
    else
        check_fail "Insufficient disk space ($available_disk_gb GB < 30 GB minimum)"
    fi
    
    # CPU cores
    cpu_cores=$(nproc)
    echo "  CPU Cores: $cpu_cores"
    if [[ $cpu_cores -ge 2 ]]; then
        check_pass "Sufficient CPU cores ($cpu_cores >= 2)"
    else
        check_warn "Only $cpu_cores CPU core. 2+ recommended"
    fi
}

check_network() {
    section "Network Connectivity"
    
    # Check internet connectivity
    if ping -c 1 8.8.8.8 >/dev/null 2>&1; then
        check_pass "Internet connectivity available"
    else
        check_fail "No internet connectivity (required for Docker pulls)"
    fi
    
    # Check DNS resolution
    if host github.com >/dev/null 2>&1; then
        check_pass "DNS resolution working"
    else
        check_fail "DNS resolution failed"
    fi
    
    # Check if ports are available
    local ports_to_check=(7070 7080 7110 3000 9090 16686)
    local ports_in_use=()
    
    for port in "${ports_to_check[@]}"; do
        if netstat -tuln 2>/dev/null | grep -q ":$port "; then
            ports_in_use+=("$port")
        fi
    done
    
    if [[ ${#ports_in_use[@]} -eq 0 ]]; then
        check_pass "Required ports are available"
    else
        check_warn "Ports already in use: ${ports_in_use[*]}"
    fi
}

check_docker() {
    section "Docker Installation"
    
    if command -v docker >/dev/null 2>&1; then
        check_pass "Docker installed ($(docker --version))"
        
        # Check if Docker daemon is running
        if docker info >/dev/null 2>&1; then
            check_pass "Docker daemon is running"
        else
            check_fail "Docker daemon is not running"
        fi
        
        # Check Docker Compose
        if docker compose version >/dev/null 2>&1; then
            check_pass "Docker Compose v2 installed"
        elif command -v docker-compose >/dev/null 2>&1; then
            check_pass "Docker Compose v1 installed"
        else
            check_fail "Docker Compose not found"
        fi
    else
        check_warn "Docker not installed (will be installed during deployment)"
    fi
}

check_project_files() {
    section "Project Structure"
    
    local required_files=(
        "Dockerfile"
        "docker-compose-testnet.yml"
        "Cargo.toml"
        "config.example.toml"
    )
    
    for file in "${required_files[@]}"; do
        if [[ -f "$SCRIPT_DIR/$file" ]]; then
            check_pass "Found: $file"
        else
            check_fail "Missing: $file"
        fi
    done
    
    # Check for validator_keys directory
    if [[ -d "$SCRIPT_DIR/validator_keys" ]]; then
        check_pass "validator_keys directory exists"
        
        # Check for key files
        local missing_keys=()
        for i in 1 2 3 4; do
            if [[ ! -f "$SCRIPT_DIR/validator_keys/validator${i}.key" ]]; then
                missing_keys+=("validator${i}.key")
            fi
        done
        
        if [[ ${#missing_keys[@]} -eq 0 ]]; then
            check_pass "All 4 validator keys present"
        else
            check_fail "Missing validator keys: ${missing_keys[*]}"
            echo "    → Run generate-validator-keys.ps1 to create them"
        fi
    else
        check_fail "validator_keys directory not found"
        echo "    → Run generate-validator-keys.ps1 to create validator keys"
    fi
    
    # Check for monitoring config
    if [[ -f "$SCRIPT_DIR/monitoring/prometheus.yml" ]]; then
        check_pass "Monitoring configuration exists"
    else
        check_warn "monitoring/prometheus.yml not found (will be created)"
    fi
}

check_deployment_script() {
    section "Deployment Script"
    
    if [[ -f "$SCRIPT_DIR/deploy-ubuntu-testnet.sh" ]]; then
        check_pass "Deployment script found"
        
        if [[ -x "$SCRIPT_DIR/deploy-ubuntu-testnet.sh" ]]; then
            check_pass "Deployment script is executable"
        else
            check_warn "Deployment script not executable"
            echo "    → Run: chmod +x deploy-ubuntu-testnet.sh"
        fi
    else
        check_fail "deploy-ubuntu-testnet.sh not found"
    fi
}

check_firewall() {
    section "Firewall"
    
    if command -v ufw >/dev/null 2>&1; then
        check_pass "UFW firewall available"
        
        ufw_status=$(ufw status 2>/dev/null | head -n 1 || echo "Status: inactive")
        echo "  $ufw_status"
        
        if echo "$ufw_status" | grep -q "inactive"; then
            check_warn "Firewall is inactive (will be configured during deployment)"
        fi
    else
        check_warn "UFW not installed (will be installed during deployment)"
    fi
}

check_dependencies() {
    section "System Dependencies"
    
    local required_commands=(
        "curl"
        "git"
        "netstat"
        "awk"
        "grep"
    )
    
    for cmd in "${required_commands[@]}"; do
        if command -v "$cmd" >/dev/null 2>&1; then
            check_pass "$cmd available"
        else
            check_warn "$cmd not found (will be installed)"
        fi
    done
}

################################################################################
# Summary and Recommendations
################################################################################

print_summary() {
    echo ""
    echo -e "${BLUE}=========================================="
    echo "  Validation Summary"
    echo -e "==========================================${NC}"
    echo ""
    echo -e "  ${GREEN}Passed:${NC}   $CHECKS_PASSED"
    echo -e "  ${YELLOW}Warnings:${NC} $CHECKS_WARNING"
    echo -e "  ${RED}Failed:${NC}   $CHECKS_FAILED"
    echo ""
    
    if [[ $CHECKS_FAILED -eq 0 ]]; then
        echo -e "${GREEN}✓ All critical checks passed!${NC}"
        echo ""
        echo "You're ready to deploy. Run:"
        echo "  sudo ./deploy-ubuntu-testnet.sh"
        echo ""
        
        if [[ $CHECKS_WARNING -gt 0 ]]; then
            echo -e "${YELLOW}Note: There are $CHECKS_WARNING warning(s).${NC}"
            echo "Review them above. They may not prevent deployment but could affect performance."
            echo ""
        fi
        
        return 0
    else
        echo -e "${RED}✗ $CHECKS_FAILED critical issue(s) found${NC}"
        echo ""
        echo "Please fix the failed checks before deploying:"
        echo ""
        
        if [[ ! -d "$SCRIPT_DIR/validator_keys" ]] || [[ ! -f "$SCRIPT_DIR/validator_keys/validator1.key" ]]; then
            echo "  1. Generate validator keys:"
            echo "     On Windows: powershell -ExecutionPolicy Bypass -File generate-validator-keys.ps1"
            echo "     Then upload validator_keys/ directory to the server"
            echo ""
        fi
        
        if [[ ! -f "$SCRIPT_DIR/deploy-ubuntu-testnet.sh" ]]; then
            echo "  2. Ensure all project files are present"
            echo "     Re-upload or re-clone the repository"
            echo ""
        fi
        
        if ! ping -c 1 8.8.8.8 >/dev/null 2>&1; then
            echo "  3. Fix network connectivity"
            echo "     Ensure server has internet access"
            echo ""
        fi
        
        return 1
    fi
}

################################################################################
# Main
################################################################################

main() {
    print_header
    
    check_os
    check_resources
    check_network
    check_docker
    check_project_files
    check_deployment_script
    check_firewall
    check_dependencies
    
    print_summary
}

main "$@"
