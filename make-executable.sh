#!/usr/bin/env bash

################################################################################
# dchat - Make All Scripts Executable
# 
# Quick utility to make all deployment scripts executable
# Run this first after uploading to server
#
# Usage: bash make-executable.sh
################################################################################

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Making deployment scripts executable..."

# List of scripts to make executable
SCRIPTS=(
    "deploy-ubuntu-testnet.sh"
    "pre-deployment-check.sh"
    "test-deployment.sh"
    "start-testnet.sh"
    "stop-testnet.sh"
    "logs-testnet.sh"
    "status-testnet.sh"
    "scripts/deploy-docker.sh"
)

for script in "${SCRIPTS[@]}"; do
    if [[ -f "$SCRIPT_DIR/$script" ]]; then
        chmod +x "$SCRIPT_DIR/$script"
        echo "✓ $script"
    else
        echo "⚠ $script not found (may be generated during deployment)"
    fi
done

echo ""
echo "All scripts are now executable!"
echo ""
echo "Next steps:"
echo "  1. Run pre-flight check:  ./pre-deployment-check.sh"
echo "  2. Deploy testnet:        sudo ./deploy-ubuntu-testnet.sh"
echo "  3. Verify deployment:     ./test-deployment.sh"
echo ""
