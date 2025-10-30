#!/usr/bin/env bash

# dchat Docker deployment script (Linux)
# - Builds local images (Rust multi-stage Dockerfile)
# - Pulls required third-party images
# - Starts the selected stack with Docker Compose
# - Waits for containers to be running/healthy and prints a status summary

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

STACK="prod"          # dev | testnet | prod
PROJECT_NAME="dchat"   # docker compose project name
NO_BUILD=0
NO_PULL=0
PRUNE=0
WAIT=1
LOGS=0
NO_CACHE=0
FORCE_RECREATE=0
LIST_IMAGES=0

usage() {
  cat <<EOF
Usage: $(basename "$0") [options]

Options:
  --stack <dev|testnet|prod>   Select compose stack (default: prod)
  -p, --project <name>         Compose project name (default: dchat)
  --no-build                   Skip docker compose build step
  --no-pull                    Skip pulling third-party images
  --no-wait                    Do not wait for health/running status
  --logs                       Tail logs after start (Ctrl+C to stop)
  --prune                      Prune dangling images after build
  --no-cache                   Build without using cache
  --force-recreate             Recreate containers even if config/image unchanged
  --list-images                Print third-party images referenced by this stack and exit
  -h, --help                   Show this help

Examples:
  $(basename "$0") --stack prod
  $(basename "$0") --stack testnet --project dchat-test --logs
EOF
}

fail() { echo "[ERROR] $*" >&2; exit 1; }
info() { echo "[INFO]  $*"; }
note() { echo "[NOTE]  $*"; }

# Detect docker compose command (v2 plugin or v1 binary)
detect_compose() {
  if command -v docker >/dev/null 2>&1; then
    if docker compose version >/dev/null 2>&1; then
      echo "docker compose"
      return 0
    fi
  fi
  if command -v docker-compose >/dev/null 2>&1; then
    echo "docker-compose"
    return 0
  fi
  return 1
}

COMPOSE_CMD=""

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --stack)
        STACK="${2:-}"; shift 2;;
      -p|--project)
        PROJECT_NAME="${2:-}"; shift 2;;
      --no-build)
        NO_BUILD=1; shift;;
      --no-pull)
        NO_PULL=1; shift;;
      --no-wait)
        WAIT=0; shift;;
      --logs)
        LOGS=1; shift;;
      --prune)
        PRUNE=1; shift;;
      --no-cache)
        NO_CACHE=1; shift;;
      --force-recreate)
        FORCE_RECREATE=1; shift;;
      --list-images)
        LIST_IMAGES=1; shift;;
      -h|--help)
        usage; exit 0;;
      *)
        usage; fail "Unknown argument: $1";;
    esac
  done
}

compose_file_for_stack() {
  case "$STACK" in
    dev) echo "$REPO_ROOT/docker-compose-dev.yml";;
    testnet) echo "$REPO_ROOT/docker-compose-testnet.yml";;
    prod|production) echo "$REPO_ROOT/docker-compose-production.yml";;
    *) fail "Unsupported stack: $STACK (expected dev|testnet|prod)";;
  esac
}

preflight() {
  info "Running preflight checks"
  command -v docker >/dev/null 2>&1 || fail "docker not found. Install Docker Engine first."
  docker info >/dev/null 2>&1 || fail "Docker daemon not available. Ensure 'docker' service is running and your user has permissions (add to docker group)."

  COMPOSE_CMD=$(detect_compose) || fail "Docker Compose not found. Install Docker Compose v2 (docker compose) or v1 (docker-compose)."
  info "Using Compose: $COMPOSE_CMD"

  local compose_file
  compose_file="$(compose_file_for_stack)"
  [[ -f "$compose_file" ]] || fail "Compose file not found: $compose_file"
  info "Compose file: $compose_file"

  # Validate compose config
  if ! $COMPOSE_CMD -p "$PROJECT_NAME" -f "$compose_file" config >/dev/null; then
    $COMPOSE_CMD -p "$PROJECT_NAME" -f "$compose_file" config || true
    fail "Compose configuration validation failed. See output above."
  fi

  validate_required_files "$compose_file"
}

gather_third_party_images() {
  # Parse image: lines from the rendered config (portable, no yq dependency)
  local compose_file="$1"
  $COMPOSE_CMD -p "$PROJECT_NAME" -f "$compose_file" config \
    | awk '/^[[:space:]]*image:[[:space:]]*/ {print $2}' \
    | sed 's/\r$//' \
    | sort -u
}

pull_images_if_needed() {
  local compose_file="$1"
  if [[ "$NO_PULL" -eq 1 ]]; then
    note "Skipping image pull per --no-pull"
    return 0
  fi
  info "Checking and pulling third-party images (if missing)"
  local images
  images=$(gather_third_party_images "$compose_file") || images=""
  if [[ -z "$images" ]]; then
    note "No external images declared; nothing to pull"
  else
    while IFS= read -r img; do
      [[ -z "$img" ]] && continue
      if docker image inspect "$img" >/dev/null 2>&1; then
        note "Image present: $img"
      else
        info "Pulling: $img"
        docker pull "$img"
      fi
    done <<<"$images"
  fi
}

build_images() {
  local compose_file="$1"
  if [[ "$NO_BUILD" -eq 1 ]]; then
    note "Skipping build per --no-build"
    return 0
  fi
  info "Building local images (this may take a while on first run)"
  local build_flags=()
  [[ "$NO_CACHE" -eq 1 ]] && build_flags+=("--no-cache")
  # Prefer pulling latest base layers for security
  build_flags+=("--pull")
  $COMPOSE_CMD -p "$PROJECT_NAME" -f "$compose_file" build "${build_flags[@]}"
}

prune_dangling() {
  if [[ "$PRUNE" -eq 1 ]]; then
    info "Pruning dangling images"
    docker image prune -f >/dev/null || true
  fi
}

start_stack() {
  local compose_file="$1"
  info "Starting stack: $STACK (project: $PROJECT_NAME)"
  local up_flags=("-d" "--remove-orphans")
  [[ "$FORCE_RECREATE" -eq 1 ]] && up_flags+=("--force-recreate")
  $COMPOSE_CMD -p "$PROJECT_NAME" -f "$compose_file" up "${up_flags[@]}"
}

list_project_containers() {
  docker ps -a --filter "label=com.docker.compose.project=$PROJECT_NAME" --format '{{.ID}} {{.Names}}'
}

container_health() {
  local cid="$1"
  # Returns: healthy | starting | unhealthy | none (no healthcheck)
  local status
  status=$(docker inspect -f '{{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' "$cid") || echo "none"
  echo "$status"
}

container_running() {
  local cid="$1"
  docker inspect -f '{{.State.Running}}' "$cid" 2>/dev/null || echo "false"
}

wait_for_containers() {
  if [[ "$WAIT" -ne 1 ]]; then
    note "Not waiting for health/running per --no-wait"
    return 0
  fi
  info "Waiting for containers to be running/healthy"
  local timeout=600
  local interval=5
  local start_ts=$(date +%s)

  # Build a set of services that define explicit healthchecks in the rendered config
  # so we only enforce health=healthy when a healthcheck exists.
  local services_with_healthchecks
  services_with_healthchecks=$(services_with_healthcheck "$compose_file")

  while true; do
    local all_ok=1
    while IFS= read -r line; do
      [[ -z "$line" ]] && continue
      local cid name
      cid=$(awk '{print $1}' <<<"$line")
      name=$(awk '{print $2}' <<<"$line")
      local run
      run=$(container_running "$cid")
      if [[ "$run" != "true" ]]; then
        all_ok=0
        note "$name not running yet"
        continue
      fi
      local hc
      hc=$(container_health "$cid")
      # If this service has an explicit healthcheck, require healthy.
      # Otherwise, ignore Dockerfile-level healthchecks and consider running=true as sufficient.
      if service_has_healthcheck "$name" "$services_with_healthchecks"; then
        case "$hc" in
          healthy)
            : # ok
            ;;
          starting)
            all_ok=0
            note "$name health: starting"
            ;;
          unhealthy)
            all_ok=0
            note "$name health: UNHEALTHY"
            ;;
          *)
            all_ok=0
            note "$name health: unknown"
            ;;
        esac
      fi
    done < <(list_project_containers)

    if [[ "$all_ok" -eq 1 ]]; then
      break
    fi
    local now=$(date +%s)
    if (( now - start_ts > timeout )); then
      fail "Timeout waiting for containers to become healthy/running"
    fi
    sleep "$interval"
  done

  info "All containers are running (and healthy when defined)"
}

print_summary() {
  info "Containers summary (project: $PROJECT_NAME)"
  docker ps --filter "label=com.docker.compose.project=$PROJECT_NAME" --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}'
}

tail_logs() {
  info "Tailing logs (Ctrl+C to stop)"
  $COMPOSE_CMD -p "$PROJECT_NAME" -f "$1" logs -f --tail=200 || true
}

# Extract list of service names that define a healthcheck in the rendered Compose config
services_with_healthcheck() {
  local compose_file="$1"
  # Render config, then print service names that contain a 'healthcheck:' section.
  # This avoids depending on external tools like yq.
  $COMPOSE_CMD -p "$PROJECT_NAME" -f "$compose_file" config 2>/dev/null \
    | awk '
      $1=="services:" {in_services=1; next}
      in_services && NF==0 {next}
      in_services && $1!~/^[[:space:]]/ && $1!="services:" {next}
      in_services {
        # capture service name lines like "  name:"
        if ($1=="" && $2~/^[^:]+:$/) {
          svc=$2; sub(":$","",svc)
          in_hc=0
        }
        if ($1=="" && $2=="healthcheck:") {
          in_hc=1
        }
        if (in_hc && svc!="") {
          print svc
          in_hc=0
        }
      }
    ' \
    | sed 's/^\s*//;s/\s*$//' \
    | sort -u
}

# Check if given container name corresponds to a service that has a healthcheck
service_has_healthcheck() {
  local container_name="$1"
  local svc_list="$2"
  # container_name usually equals the explicit container_name in compose.
  # We map by checking if any service name appears as a substring in the container name.
  # This is heuristic but works with explicit names like dchat-relay1, dchat-validator1, etc.
  while IFS= read -r svc; do
    [[ -z "$svc" ]] && continue
    if [[ "$container_name" == *"$svc"* ]]; then
      return 0
    fi
  done <<<"$svc_list"
  return 1
}

# Validate that required host files for bind mounts exist to prevent compose errors
validate_bind_mounts() {
  local compose_file="$1"
  local missing=0
  # Extract bind sources from rendered config
  local sources
  sources=$($COMPOSE_CMD -p "$PROJECT_NAME" -f "$compose_file" config 2>/dev/null \
    | awk '
        /type: bind/ { bind=1; next }
        bind && /source:/ { gsub("source:", ""); gsub(/^\s+|\s+$/,"", $0); print $0; bind=0 }
      ')
  if [[ -n "$sources" ]]; then
    while IFS= read -r path; do
      [[ -z "$path" ]] && continue
      if [[ ! -e "$path" ]]; then
        echo "[ERROR] Missing bind mount source: $path" >&2
        missing=1
      fi
    done <<<"$sources"
  fi
  return $missing
}

# Additional stack-specific validations (keys, configs, etc.)
validate_stack_specific() {
  local compose_file="$1"
  case "$STACK" in
    prod|production|testnet)
      # Ensure validator keys exist if referenced
      local keydir="$REPO_ROOT/validator_keys"
      if grep -q "validator_keys" "$compose_file" 2>/dev/null || $COMPOSE_CMD -p "$PROJECT_NAME" -f "$compose_file" config | grep -q "/validator_keys"; then
        for i in 1 2 3 4; do
          if [[ ! -f "$keydir/validator${i}.key" ]]; then
            echo "[ERROR] Missing validator key: $keydir/validator${i}.key" >&2
            echo "        Generate keys and place them in validator_keys/ (see generate-validator-keys.ps1)" >&2
            return 1
          fi
        done
      fi
      ;;
  esac
}

validate_required_files() {
  local compose_file="$1"
  info "Validating required files for bind mounts and stack"
  if ! validate_bind_mounts "$compose_file"; then
    fail "Required bind mount sources are missing. Create the files/directories listed above and retry."
  fi
  if ! validate_stack_specific "$compose_file"; then
    fail "Stack-specific validation failed (e.g., missing validator keys)."
  fi
}

main() {
  parse_args "$@"
  preflight

  local compose_file
  compose_file="$(compose_file_for_stack)"

  pull_images_if_needed "$compose_file"
  if [[ "$LIST_IMAGES" -eq 1 ]]; then
    info "Third-party images referenced in $STACK stack:"
    gather_third_party_images "$compose_file" || true
    exit 0
  fi
  build_images "$compose_file"
  prune_dangling
  start_stack "$compose_file"
  wait_for_containers
  print_summary
  if [[ "$LOGS" -eq 1 ]]; then
    tail_logs "$compose_file"
  fi
}

main "$@"
