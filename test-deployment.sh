#!/usr/bin/env bash

################################################################################
# dchat Post-Deployment Test Suite
# 
# Validates that the deployed testnet is functioning correctly
# Run this after deployment to verify everything works
#
# Usage: ./test-deployment.sh [--skip-load-test]
################################################################################

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Test configuration
SKIP_LOAD_TEST=0
TESTS_PASSED=0
TESTS_FAILED=0

################################################################################
# Helper Functions
################################################################################

log() {
    echo -e "${BLUE}[TEST]${NC} $*"
}

pass() {
    echo -e "${GREEN}✓ PASS:${NC} $*"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

fail() {
    echo -e "${RED}✗ FAIL:${NC} $*"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

warn() {
    echo -e "${YELLOW}⚠ WARN:${NC} $*"
}

section() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

wait_for_container() {
    local container=$1
    local max_wait=60
    local elapsed=0
    
    while [[ $elapsed -lt $max_wait ]]; do
        if docker ps --format '{{.Names}}' | grep -q "^${container}$"; then
            local status=$(docker inspect -f '{{.State.Status}}' "$container")
            if [[ "$status" == "running" ]]; then
                return 0
            fi
        fi
        sleep 2
        elapsed=$((elapsed + 2))
    done
    
    return 1
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --skip-load-test)
                SKIP_LOAD_TEST=1
                shift
                ;;
            *)
                echo "Unknown option: $1"
                exit 1
                ;;
        esac
    done
}

################################################################################
# Test: Container Status
################################################################################

test_containers_running() {
    section "Container Status Tests"
    
    local expected_containers=(
        "dchat-validator1"
        "dchat-validator2"
        "dchat-validator3"
        "dchat-validator4"
        "dchat-relay1"
        "dchat-relay2"
        "dchat-relay3"
        "dchat-relay4"
        "dchat-relay5"
        "dchat-relay6"
        "dchat-relay7"
        "dchat-user1"
        "dchat-user2"
        "dchat-user3"
        "dchat-prometheus"
        "dchat-grafana"
        "dchat-jaeger"
    )
    
    log "Checking if all expected containers are running..."
    
    for container in "${expected_containers[@]}"; do
        if docker ps --format '{{.Names}}' | grep -q "^${container}$"; then
            local status=$(docker inspect -f '{{.State.Status}}' "$container")
            if [[ "$status" == "running" ]]; then
                pass "Container $container is running"
            else
                fail "Container $container exists but is not running (status: $status)"
            fi
        else
            fail "Container $container not found"
        fi
    done
}

test_container_health() {
    section "Container Health Tests"
    
    local containers_with_health=(
        "dchat-validator1"
        "dchat-validator2"
        "dchat-validator3"
        "dchat-validator4"
        "dchat-user1"
        "dchat-user2"
        "dchat-user3"
    )
    
    log "Checking container health status..."
    
    for container in "${containers_with_health[@]}"; do
        if docker ps --format '{{.Names}}' | grep -q "^${container}$"; then
            local health=$(docker inspect -f '{{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' "$container")
            case "$health" in
                healthy)
                    pass "Container $container is healthy"
                    ;;
                none)
                    warn "Container $container has no health check"
                    ;;
                *)
                    fail "Container $container health: $health"
                    ;;
            esac
        fi
    done
}

################################################################################
# Test: Network Connectivity
################################################################################

test_network_endpoints() {
    section "Network Endpoint Tests"
    
    log "Testing HTTP endpoints..."
    
    # Validator endpoints
    for port in 7071 7073 7075 7077; do
        if curl -sf "http://localhost:$port/health" >/dev/null 2>&1; then
            pass "Validator endpoint on port $port is responding"
        else
            fail "Validator endpoint on port $port is not responding"
        fi
    done
    
    # Monitoring endpoints
    if curl -sf "http://localhost:9090/-/healthy" >/dev/null 2>&1; then
        pass "Prometheus is responding"
    else
        fail "Prometheus is not responding"
    fi
    
    if curl -sf "http://localhost:3000/api/health" >/dev/null 2>&1; then
        pass "Grafana is responding"
    else
        fail "Grafana is not responding"
    fi
    
    if curl -sf "http://localhost:16686/" >/dev/null 2>&1; then
        pass "Jaeger UI is responding"
    else
        fail "Jaeger UI is not responding"
    fi
}

test_internal_connectivity() {
    section "Internal Network Connectivity Tests"
    
    log "Testing container-to-container connectivity..."
    
    # Test user1 can reach validator1
    if docker exec dchat-user1 ping -c 1 validator1 >/dev/null 2>&1; then
        pass "user1 can reach validator1"
    else
        fail "user1 cannot reach validator1"
    fi
    
    # Test relay1 can reach validator1
    if docker exec dchat-relay1 ping -c 1 validator1 >/dev/null 2>&1; then
        pass "relay1 can reach validator1"
    else
        fail "relay1 cannot reach validator1"
    fi
    
    # Test user1 can reach relay1
    if docker exec dchat-user1 ping -c 1 relay1 >/dev/null 2>&1; then
        pass "user1 can reach relay1"
    else
        fail "user1 cannot reach relay1"
    fi
}

################################################################################
# Test: Data Persistence
################################################################################

test_data_volumes() {
    section "Data Volume Tests"
    
    log "Checking data volumes..."
    
    local volumes=(
        "dchat-testnet_validator1_data"
        "dchat-testnet_validator2_data"
        "dchat-testnet_validator3_data"
        "dchat-testnet_validator4_data"
        "dchat-testnet_relay1_data"
        "dchat-testnet_prometheus_data"
        "dchat-testnet_grafana_data"
    )
    
    for volume in "${volumes[@]}"; do
        if docker volume ls --format '{{.Name}}' | grep -q "^${volume}$"; then
            pass "Volume $volume exists"
        else
            warn "Volume $volume not found (may use bind mounts)"
        fi
    done
}

################################################################################
# Test: Monitoring Stack
################################################################################

test_prometheus_targets() {
    section "Prometheus Targets Test"
    
    log "Checking Prometheus scrape targets..."
    
    local targets=$(curl -s "http://localhost:9090/api/v1/targets" | jq -r '.data.activeTargets | length' 2>/dev/null || echo "0")
    
    if [[ "$targets" -gt 0 ]]; then
        pass "Prometheus has $targets active targets"
        
        # Check for healthy targets
        local healthy=$(curl -s "http://localhost:9090/api/v1/targets" | jq -r '[.data.activeTargets[] | select(.health=="up")] | length' 2>/dev/null || echo "0")
        if [[ "$healthy" -gt 0 ]]; then
            pass "Prometheus has $healthy healthy targets"
        else
            warn "No healthy Prometheus targets found"
        fi
    else
        fail "Prometheus has no active targets"
    fi
}

test_grafana_datasource() {
    section "Grafana Datasource Test"
    
    log "Checking Grafana datasources..."
    
    local datasources=$(curl -s "http://admin:admin@localhost:3000/api/datasources" 2>/dev/null)
    
    if echo "$datasources" | jq -e '. | length > 0' >/dev/null 2>&1; then
        local prometheus_ds=$(echo "$datasources" | jq -r '.[] | select(.type=="prometheus") | .name' 2>/dev/null || echo "")
        if [[ -n "$prometheus_ds" ]]; then
            pass "Grafana has Prometheus datasource: $prometheus_ds"
        else
            warn "No Prometheus datasource found in Grafana"
        fi
    else
        warn "Could not query Grafana datasources (check credentials)"
    fi
}

################################################################################
# Test: Validator Consensus
################################################################################

test_validator_consensus() {
    section "Validator Consensus Tests"
    
    log "Checking validator block heights..."
    
    local heights=()
    local ports=(7071 7073 7075 7077)
    
    for port in "${ports[@]}"; do
        local height=$(curl -s "http://localhost:$port/status" 2>/dev/null | jq -r '.block_height // 0' 2>/dev/null || echo "0")
        heights+=("$height")
        log "Validator on port $port: block height $height"
    done
    
    # Check if all validators have similar heights (within 5 blocks)
    local max_height=0
    local min_height=999999999
    
    for height in "${heights[@]}"; do
        if [[ $height -gt $max_height ]]; then
            max_height=$height
        fi
        if [[ $height -lt $min_height ]] && [[ $height -gt 0 ]]; then
            min_height=$height
        fi
    done
    
    local diff=$((max_height - min_height))
    
    if [[ $max_height -gt 0 ]]; then
        if [[ $diff -le 5 ]]; then
            pass "Validators are in consensus (height difference: $diff blocks)"
        else
            warn "Validators may be out of sync (height difference: $diff blocks)"
        fi
    else
        fail "No validators are producing blocks"
    fi
}

################################################################################
# Test: Message Functionality
################################################################################

test_message_propagation() {
    section "Message Propagation Tests"
    
    log "Testing message sending between users..."
    
    # This is a basic connectivity test
    # In a real deployment, you'd test actual message sending
    
    warn "Message propagation test requires manual verification"
    warn "Run: docker exec -it dchat-user1 dchat send --to user2@dchat.local --message 'Test'"
    warn "Then: docker exec -it dchat-user2 dchat inbox"
}

################################################################################
# Test: Resource Usage
################################################################################

test_resource_usage() {
    section "Resource Usage Tests"
    
    log "Checking container resource usage..."
    
    # Get stats for all dchat containers
    local stats=$(docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}" | grep dchat || echo "")
    
    if [[ -n "$stats" ]]; then
        pass "Container resource stats available"
        echo "$stats"
    else
        warn "Could not retrieve container stats"
    fi
    
    # Check overall system resources
    log "System resource usage:"
    free -h | grep -E "Mem:|Swap:"
    df -h "$SCRIPT_DIR" | grep -v "Filesystem"
}

################################################################################
# Test: Load Test (Optional)
################################################################################

test_load_performance() {
    if [[ $SKIP_LOAD_TEST -eq 1 ]]; then
        section "Load Test (Skipped)"
        warn "Load test skipped (--skip-load-test flag)"
        return
    fi
    
    section "Load Performance Test"
    
    log "Running basic load test..."
    warn "This test is currently a placeholder"
    warn "In production, run: docker exec -it dchat-relay1 dchat benchmark --duration 60 --messages 1000"
}

################################################################################
# Summary
################################################################################

print_summary() {
    echo ""
    section "Test Summary"
    
    echo ""
    echo -e "  ${GREEN}Tests Passed:${NC}  $TESTS_PASSED"
    echo -e "  ${RED}Tests Failed:${NC}  $TESTS_FAILED"
    echo ""
    
    if [[ $TESTS_FAILED -eq 0 ]]; then
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${GREEN}  ✓ All tests passed!${NC}"
        echo -e "${GREEN}  Testnet is operational and healthy.${NC}"
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""
        echo "Next steps:"
        echo "  • Access Grafana: http://$(hostname -I | awk '{print $1}'):3000"
        echo "  • View logs: ./logs-testnet.sh"
        echo "  • Test messaging: docker exec -it dchat-user1 bash"
        echo ""
        return 0
    else
        echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${RED}  ✗ Some tests failed${NC}"
        echo -e "${RED}  Please investigate the failures above.${NC}"
        echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""
        echo "Troubleshooting:"
        echo "  • Check logs: docker logs <container-name>"
        echo "  • Check status: ./status-testnet.sh"
        echo "  • View full guide: cat TESTNET_DEPLOYMENT_UBUNTU.md"
        echo ""
        return 1
    fi
}

################################################################################
# Main
################################################################################

main() {
    echo -e "${BLUE}"
    echo "=========================================="
    echo "  dchat Post-Deployment Test Suite"
    echo "=========================================="
    echo -e "${NC}"
    echo ""
    
    parse_args "$@"
    
    # Wait a bit for services to stabilize
    log "Waiting 10 seconds for services to stabilize..."
    sleep 10
    
    # Run all test suites
    test_containers_running
    test_container_health
    test_network_endpoints
    test_internal_connectivity
    test_data_volumes
    test_prometheus_targets
    test_grafana_datasource
    test_validator_consensus
    test_message_propagation
    test_resource_usage
    test_load_performance
    
    # Print summary
    print_summary
}

main "$@"
