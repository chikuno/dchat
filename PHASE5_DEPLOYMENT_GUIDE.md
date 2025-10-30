# Phase 5 Production Deployment Guide

**Version**: 1.0  
**Status**: Ready for MVP Deployment  
**Date**: October 28, 2025

---

## Table of Contents

1. [Deployment Architecture](#1-deployment-architecture)
2. [Docker Configuration](#2-docker-configuration)
3. [Kubernetes Manifests](#3-kubernetes-manifests)
4. [Environment Configuration](#4-environment-configuration)
5. [Monitoring & Alerting](#5-monitoring--alerting)
6. [Disaster Recovery](#6-disaster-recovery)
7. [Operational Runbooks](#7-operational-runbooks)

---

## 1. Deployment Architecture

### 1.1 MVP Deployment (Single Server)

```
┌─────────────────────────────────┐
│     Production Server (2vCPU)   │
│                                 │
│  ┌───────────────────────────┐  │
│  │   dchat-core (Binary)     │  │
│  │                           │  │
│  │ ┌─────────────────────┐   │  │
│  │ │ Marketplace         │   │  │
│  │ │ Bridge              │   │  │
│  │ │ Observability       │   │  │
│  │ │ Accessibility       │   │  │
│  │ └─────────────────────┘   │  │
│  │                           │  │
│  └───────────────────────────┘  │
│                                 │
│  SQLite: /data/dchat.db         │
│  Config: /etc/dchat/config.toml │
│  Logs: /var/log/dchat/          │
└─────────────────────────────────┘
     │
     ├─→ Chat Chain (via libp2p)
     ├─→ Currency Chain (RPC)
     └─→ Relay Network
```

**Capacity**: 1,000-5,000 active users

### 1.2 Scalable Deployment (Multi-Server)

```
┌─────────────────────────────────────────────────────────┐
│           Kubernetes Cluster (3+ nodes)                 │
│                                                         │
│  ┌────────────────────────────────────────────────┐    │
│  │  API Gateway / Load Balancer                   │    │
│  │  (nginx / HAProxy with TLS termination)        │    │
│  └────────────────────────────────────────────────┘    │
│           │           │           │                    │
│  ┌────────▼───┐ ┌────▼────┐ ┌────▼────┐              │
│  │   Pod 1    │ │  Pod 2  │ │  Pod 3  │              │
│  │ dchat-core │ │dchat-co │ │dchat-c  │              │
│  │            │ │  re     │ │  ore    │              │
│  └────────────┘ └─────────┘ └─────────┘              │
│           │           │           │                    │
│  ┌────────▼───────────▼───────────▼────┐              │
│  │  Persistent Volume (PostgreSQL)     │              │
│  │  - Marketplace data                 │              │
│  │  - Bridge transactions              │              │
│  │  - Accessibility config             │              │
│  └─────────────────────────────────────┘              │
│                                                         │
│  ┌────────────────────────────────────────────────┐    │
│  │  Observability Stack                          │    │
│  │  - Prometheus (metrics)                       │    │
│  │  - Jaeger (tracing)                           │    │
│  │  - Grafana (dashboards)                       │    │
│  │  - AlertManager (alerting)                    │    │
│  └────────────────────────────────────────────────┘    │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**Capacity**: 10,000-100,000+ active users

---

## 2. Docker Configuration

### 2.1 Dockerfile (Multi-stage)

```dockerfile
# Stage 1: Build
FROM rust:1.70 as builder

WORKDIR /build

# Copy source
COPY . .

# Build release
RUN cargo build --release \
    --package dchat \
    --package dchat-marketplace \
    --package dchat-observability \
    --package dchat-bridge \
    --package dchat-accessibility \
    --package dchat-testing

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 dchat

# Copy binary from builder
COPY --from=builder /build/target/release/dchat /usr/local/bin/

# Create directories
RUN mkdir -p /data /var/log/dchat /etc/dchat && \
    chown -R dchat:dchat /data /var/log/dchat /etc/dchat

# Switch to app user
USER dchat

# Expose ports
EXPOSE 9000    # API
EXPOSE 9001    # Metrics
EXPOSE 7000    # P2P (libp2p)

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:9001/health || exit 1

# Run
CMD ["dchat", "--config", "/etc/dchat/config.toml"]
```

### 2.2 Docker Compose (Development)

```yaml
version: '3.8'

services:
  dchat-core:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: dchat-core
    ports:
      - "9000:9000"    # API
      - "9001:9001"    # Metrics
      - "7000:7000"    # P2P
    volumes:
      - dchat-data:/data
      - ./config.toml:/etc/dchat/config.toml:ro
      - dchat-logs:/var/log/dchat
    environment:
      RUST_LOG: info
      DCHAT_ENV: development
    depends_on:
      - postgres
      - jaeger

  postgres:
    image: postgres:15-alpine
    container_name: dchat-postgres
    environment:
      POSTGRES_DB: dchat
      POSTGRES_USER: dchat
      POSTGRES_PASSWORD: changeme
    volumes:
      - postgres-data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  prometheus:
    image: prom/prometheus:latest
    container_name: dchat-prometheus
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    ports:
      - "9090:9090"
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'

  grafana:
    image: grafana/grafana:latest
    container_name: dchat-grafana
    environment:
      GF_SECURITY_ADMIN_PASSWORD: admin
    volumes:
      - grafana-data:/var/lib/grafana
    ports:
      - "3000:3000"
    depends_on:
      - prometheus

  jaeger:
    image: jaegertracing/all-in-one:latest
    container_name: dchat-jaeger
    ports:
      - "6831:6831/udp"  # Jaeger agent
      - "16686:16686"    # UI
    environment:
      COLLECTOR_ZIPKIN_HOST_PORT: ":9411"

volumes:
  dchat-data:
  dchat-logs:
  postgres-data:
  prometheus-data:
  grafana-data:
```

### 2.3 Docker Build & Push

```bash
#!/bin/bash
# build-and-push.sh

REGISTRY="ghcr.io"
IMAGE_NAME="yourorg/dchat"
VERSION="5.0.0"

# Build for multiple architectures
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t $REGISTRY/$IMAGE_NAME:$VERSION \
  -t $REGISTRY/$IMAGE_NAME:latest \
  --push \
  .

# Tag separately for marketplace, bridge, etc. if needed
docker tag $REGISTRY/$IMAGE_NAME:$VERSION $REGISTRY/$IMAGE_NAME:v5
docker push $REGISTRY/$IMAGE_NAME:v5
```

---

## 3. Kubernetes Manifests

### 3.1 Namespace & RBAC

```yaml
---
apiVersion: v1
kind: Namespace
metadata:
  name: dchat
  labels:
    name: dchat

---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: dchat
  namespace: dchat

---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: dchat
rules:
  - apiGroups: [""]
    resources: ["pods", "services"]
    verbs: ["get", "list", "watch"]
  - apiGroups: [""]
    resources: ["configmaps"]
    verbs: ["get"]

---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: dchat
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: dchat
subjects:
  - kind: ServiceAccount
    name: dchat
    namespace: dchat
```

### 3.2 ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: dchat-config
  namespace: dchat
data:
  config.toml: |
    [server]
    host = "0.0.0.0"
    api_port = 9000
    metrics_port = 9001
    p2p_port = 7000
    
    [database]
    url = "postgresql://dchat:password@postgres:5432/dchat"
    max_connections = 20
    
    [chain]
    chat_chain_rpc = "http://chat-chain:8545"
    currency_chain_rpc = "http://currency-chain:8545"
    
    [relay]
    bootstrap_nodes = [
      "/dns/relay1.dchat.dev/tcp/7000",
      "/dns/relay2.dchat.dev/tcp/7000",
    ]
    
    [observability]
    jaeger_endpoint = "http://jaeger:6831"
    prometheus_pushgateway = "http://prometheus:9091"
    
    [logging]
    level = "info"
    format = "json"
```

### 3.3 Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: dchat-core
  namespace: dchat
  labels:
    app: dchat
    component: core
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: dchat
      component: core
  template:
    metadata:
      labels:
        app: dchat
        component: core
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9001"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: dchat
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 1000
      containers:
        - name: dchat
          image: ghcr.io/yourorg/dchat:5.0.0
          imagePullPolicy: Always
          ports:
            - name: api
              containerPort: 9000
              protocol: TCP
            - name: metrics
              containerPort: 9001
              protocol: TCP
            - name: p2p
              containerPort: 7000
              protocol: TCP
          
          # Environment variables
          env:
            - name: RUST_LOG
              value: "info"
            - name: DCHAT_ENV
              value: "production"
            - name: DATABASE_URL
              valueFrom:
                configMapKeyRef:
                  name: dchat-config
                  key: database_url
          
          # Liveness & readiness probes
          livenessProbe:
            httpGet:
              path: /health
              port: metrics
            initialDelaySeconds: 10
            periodSeconds: 10
            timeoutSeconds: 5
            failureThreshold: 3
          
          readinessProbe:
            httpGet:
              path: /ready
              port: metrics
            initialDelaySeconds: 5
            periodSeconds: 5
            timeoutSeconds: 3
            failureThreshold: 2
          
          # Resource limits
          resources:
            requests:
              cpu: 1000m
              memory: 1Gi
            limits:
              cpu: 2000m
              memory: 2Gi
          
          # Volume mounts
          volumeMounts:
            - name: config
              mountPath: /etc/dchat
              readOnly: true
            - name: data
              mountPath: /data
          
          # Security context
          securityContext:
            allowPrivilegeEscalation: false
            capabilities:
              drop:
                - ALL
            readOnlyRootFilesystem: true
      
      # Volumes
      volumes:
        - name: config
          configMap:
            name: dchat-config
        - name: data
          persistentVolumeClaim:
            claimName: dchat-data
      
      # Affinity
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
            - weight: 100
              podAffinityTerm:
                labelSelector:
                  matchExpressions:
                    - key: app
                      operator: In
                      values:
                        - dchat
                topologyKey: kubernetes.io/hostname
```

### 3.4 Service & Ingress

```yaml
---
apiVersion: v1
kind: Service
metadata:
  name: dchat-api
  namespace: dchat
  labels:
    app: dchat
spec:
  type: ClusterIP
  selector:
    app: dchat
  ports:
    - name: api
      port: 9000
      targetPort: api
    - name: metrics
      port: 9001
      targetPort: metrics

---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: dchat
  namespace: dchat
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  ingressClassName: nginx
  tls:
    - hosts:
        - api.dchat.dev
      secretName: dchat-tls
  rules:
    - host: api.dchat.dev
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: dchat-api
                port:
                  number: 9000
```

### 3.5 PersistentVolume & PersistentVolumeClaim

```yaml
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: dchat-data
  namespace: dchat
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 100Gi
  storageClassName: fast-ssd

---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: dchat-postgres
  namespace: dchat
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 500Gi
  storageClassName: fast-ssd
```

---

## 4. Environment Configuration

### 4.1 Environment Variables

```bash
# Production environment variables

# Server
export DCHAT_ENV=production
export DCHAT_HOST=0.0.0.0
export DCHAT_API_PORT=9000
export DCHAT_METRICS_PORT=9001
export DCHAT_P2P_PORT=7000

# Database
export DATABASE_URL="postgresql://dchat:PASSWORD@postgres.dchat.svc.cluster.local:5432/dchat"
export DATABASE_MAX_CONNECTIONS=20
export DATABASE_STATEMENT_CACHE_SIZE=100

# Chain RPC Endpoints
export CHAT_CHAIN_RPC="https://chat-chain-rpc.dchat.dev"
export CURRENCY_CHAIN_RPC="https://currency-chain-rpc.dchat.dev"

# Relay Network
export RELAY_BOOTSTRAP_NODES="/dns/relay1.dchat.dev/tcp/7000,/dns/relay2.dchat.dev/tcp/7000"
export RELAY_EXTERNAL_ADDRESSES="/dns/mynode.dchat.dev/tcp/7000"

# Observability
export JAEGER_ENDPOINT="http://jaeger:6831"
export PROMETHEUS_PUSHGATEWAY="http://prometheus:9091"
export RUST_LOG=info

# TLS
export TLS_CERT_PATH="/etc/tls/certs/tls.crt"
export TLS_KEY_PATH="/etc/tls/certs/tls.key"

# Security
export JWT_SECRET="your-256-bit-secret-key-base64-encoded"
export CORS_ALLOWED_ORIGINS="https://app.dchat.dev,https://www.dchat.dev"

# Performance
export TOKIO_WORKER_THREADS=8
export MARKETPLACE_CACHE_SIZE=10000
export BRIDGE_TX_TIMEOUT_SECS=300
```

### 4.2 Configuration File (config.toml)

```toml
[server]
host = "0.0.0.0"
api_port = 9000
metrics_port = 9001
p2p_port = 7000
worker_threads = 8
request_timeout_secs = 30

[database]
url = "postgresql://dchat:password@postgres:5432/dchat"
max_connections = 20
statement_cache_size = 100
connection_timeout_secs = 10
idle_timeout_secs = 300

[chain]
chat_chain_rpc = "https://chat-chain-rpc.dchat.dev"
currency_chain_rpc = "https://currency-chain-rpc.dchat.dev"
rpc_timeout_secs = 30

[relay]
bootstrap_nodes = [
  "/dns/relay1.dchat.dev/tcp/7000",
  "/dns/relay2.dchat.dev/tcp/7000",
  "/dns/relay3.dchat.dev/tcp/7000",
]
external_addresses = ["/dns/mynode.dchat.dev/tcp/7000"]
nat_upnp = true
nat_turn_servers = ["stun:stun.l.google.com:19302"]

[observability]
jaeger_endpoint = "http://jaeger:6831"
jaeger_sample_rate = 0.1
prometheus_pushgateway = "http://prometheus:9091"
prometheus_push_interval_secs = 60
health_check_interval_secs = 30

[marketplace]
cache_size = 10000
escrow_timeout_secs = 86400  # 24 hours
payment_confirmation_timeout_secs = 300

[bridge]
transaction_timeout_secs = 300
finality_proof_timeout_secs = 600
validator_quorum = 2  # 2-of-3
state_snapshot_interval = 1000

[accessibility]
wcag_strict_mode = true
keyboard_shortcut_conflict_check = true

[logging]
level = "info"
format = "json"
output = "stdout"
file_path = "/var/log/dchat/dchat.log"
max_file_size_mb = 100
max_backups = 10

[security]
tls_enabled = true
tls_cert_path = "/etc/tls/certs/tls.crt"
tls_key_path = "/etc/tls/certs/tls.key"
cors_allowed_origins = ["https://app.dchat.dev", "https://www.dchat.dev"]
rate_limit_enabled = true
rate_limit_requests_per_sec = 1000
```

---

## 5. Monitoring & Alerting

### 5.1 Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    monitor: 'dchat-prod'

alerting:
  alertmanagers:
    - static_configs:
        - targets:
            - alertmanager:9093

rule_files:
  - '/etc/prometheus/rules/*.yml'

scrape_configs:
  - job_name: 'dchat-core'
    static_configs:
      - targets:
          - 'dchat-core:9001'
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
      - source_labels: [__scheme__]
        target_label: scheme

  - job_name: 'kubernetes-pods'
    kubernetes_sd_configs:
      - role: pod
        namespaces:
          names:
            - dchat
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
        action: replace
        target_label: __metrics_path__
        regex: (.+)
```

### 5.2 Alert Rules

```yaml
# alert-rules.yml
groups:
  - name: dchat-alerts
    interval: 30s
    rules:
      # Marketplace alerts
      - alert: MarketplacePurchaseLatencyHigh
        expr: histogram_quantile(0.99, marketplace_purchase_latency_ms) > 100
        for: 5m
        annotations:
          summary: "Marketplace purchase latency high"
          description: "P99 purchase latency > 100ms"

      # Bridge alerts
      - alert: BridgeFinality ProofTimeout
        expr: bridge_finality_proof_timeout_total > 5
        for: 2m
        annotations:
          summary: "Bridge finality proofs timing out"
          description: "Multiple bridge transactions stuck in finality"

      # Observability alerts
      - alert: MetricsRecordingError
        expr: increase(observability_errors_total[5m]) > 10
        for: 1m
        annotations:
          summary: "Metrics recording errors"

      # Accessibility alerts
      - alert: AccessibilityValidationFailure
        expr: increase(accessibility_validation_failures_total[5m]) > 20
        for: 5m
        annotations:
          summary: "UI accessibility validation failures"

      # System alerts
      - alert: HighErrorRate
        expr: rate(dchat_errors_total[5m]) > 0.05
        for: 5m
        annotations:
          summary: "Error rate > 5%"
          description: "System experiencing high error rate"

      - alert: PodMemoryUsageHigh
        expr: container_memory_usage_bytes{pod=~"dchat-.*"} > 1.5e9
        for: 5m
        annotations:
          summary: "Pod memory usage > 1.5GB"

      - alert: DatabaseConnectionPoolExhausted
        expr: database_connections_active >= 18
        for: 2m
        annotations:
          summary: "Database connection pool near capacity"
```

### 5.3 Grafana Dashboards

**Key Metrics to Display**:

```
Dashboard: Overview
├─ System Health (CPU, Memory, Disk)
├─ Request Rate & Latency
├─ Error Rate (by component)
└─ P99 Latency (by operation)

Dashboard: Marketplace
├─ Purchases/sec
├─ Average transaction value
├─ Purchase latency (p50, p99)
└─ Creator activity

Dashboard: Bridge
├─ Initiated transactions
├─ Finality proofs submitted
├─ Transaction success rate
├─ Finality latency
└─ Validator consensus health

Dashboard: Observability
├─ Metrics recorded/sec
├─ Trace span count
├─ Health check status
└─ Alerting activity

Dashboard: Infrastructure
├─ Pod restart count
├─ PVC usage
├─ Network I/O
└─ Deployment events
```

---

## 6. Disaster Recovery

### 6.1 Backup Strategy

```bash
#!/bin/bash
# backup.sh - Automated daily backup

BACKUP_DIR="/backups/dchat"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Backup database
pg_dump -h postgres -U dchat dchat | gzip > "$BACKUP_DIR/db-$TIMESTAMP.sql.gz"

# Backup marketplace data
tar -czf "$BACKUP_DIR/marketplace-$TIMESTAMP.tar.gz" /data/marketplace

# Backup configs
tar -czf "$BACKUP_DIR/config-$TIMESTAMP.tar.gz" /etc/dchat

# Upload to S3
aws s3 cp "$BACKUP_DIR/db-$TIMESTAMP.sql.gz" s3://dchat-backups/
aws s3 cp "$BACKUP_DIR/marketplace-$TIMESTAMP.tar.gz" s3://dchat-backups/
aws s3 cp "$BACKUP_DIR/config-$TIMESTAMP.tar.gz" s3://dchat-backups/

# Keep local backups for 7 days
find "$BACKUP_DIR" -name "*.gz" -mtime +7 -delete

echo "Backup completed: $TIMESTAMP"
```

### 6.2 Recovery Procedures

#### Database Recovery

```bash
#!/bin/bash
# restore-database.sh

BACKUP_FILE=$1

if [ -z "$BACKUP_FILE" ]; then
  echo "Usage: ./restore-database.sh <backup.sql.gz>"
  exit 1
fi

# Decompress
gunzip -c "$BACKUP_FILE" > /tmp/restore.sql

# Restore
psql -h postgres -U dchat dchat < /tmp/restore.sql

# Verify
psql -h postgres -U dchat -d dchat -c "SELECT COUNT(*) FROM marketplace_listings;"

echo "Database restore completed"
rm /tmp/restore.sql
```

#### Marketplace Data Recovery

```bash
#!/bin/bash
# restore-marketplace.sh

BACKUP_FILE=$1

# Extract backup
tar -xzf "$BACKUP_FILE" -C /

# Verify integrity
dchat-verify-marketplace --data-dir /data/marketplace

echo "Marketplace data restored"
```

### 6.3 Disaster Recovery Plan

| Scenario | RTO | RPO | Recovery Steps |
|----------|-----|-----|-----------------|
| **Single pod crash** | <1 min | 0 | Kubernetes restarts pod |
| **Database failure** | <5 min | <1 min | Restore from backup, replay logs |
| **Node failure** | <2 min | 0 | Pods migrate to another node |
| **Network partition** | <10 min | <5 min | Manual failover, data sync |
| **Datacenter loss** | <30 min | <5 min | Restore from S3 backups, redeploy |

---

## 7. Operational Runbooks

### 7.1 Scaling the Deployment

```bash
#!/bin/bash
# scale-deployment.sh

REPLICAS=$1

if [ -z "$REPLICAS" ]; then
  echo "Usage: ./scale-deployment.sh <replicas>"
  exit 1
fi

# Scale deployment
kubectl -n dchat scale deployment dchat-core --replicas=$REPLICAS

# Wait for rollout
kubectl -n dchat rollout status deployment/dchat-core

echo "Scaled to $REPLICAS replicas"
```

### 7.2 Rolling Update

```bash
#!/bin/bash
# rolling-update.sh

NEW_VERSION=$1

# Update image
kubectl -n dchat set image deployment/dchat-core \
  dchat=ghcr.io/yourorg/dchat:$NEW_VERSION \
  --record

# Wait for rollout
kubectl -n dchat rollout status deployment/dchat-core

# Check if successful
if [ $? -eq 0 ]; then
  echo "Rollout successful to version $NEW_VERSION"
else
  echo "Rollout failed, rolling back..."
  kubectl -n dchat rollout undo deployment/dchat-core
fi
```

### 7.3 Debugging

```bash
# View logs
kubectl -n dchat logs -f deployment/dchat-core --tail=100

# Shell into pod
kubectl -n dchat exec -it <pod-name> -- /bin/bash

# Check metrics
kubectl top pods -n dchat

# Describe pod
kubectl -n dchat describe pod <pod-name>

# View events
kubectl -n dchat get events --sort-by='.lastTimestamp'

# Port forward for local debugging
kubectl -n dchat port-forward svc/dchat-api 9000:9000
```

### 7.4 On-Call Playbook

**Alert: High Error Rate**
1. Check logs: `kubectl logs -f dchat-core:9001`
2. Review recent deployments: `kubectl rollout history deployment/dchat-core`
3. Check database connectivity: `kubectl exec dchat-0 -- psql -h postgres -c "SELECT 1"`
4. Scale if CPU-bound: `./scale-deployment.sh 5`
5. Investigate metrics in Grafana
6. If critical: `kubectl rollout undo deployment/dchat-core`

**Alert: Database Connection Pool Exhausted**
1. Check active connections: `kubectl logs deployment/dchat-core | grep connections`
2. Increase pool size in config
3. Redeploy: `kubectl rollout restart deployment/dchat-core`
4. Monitor recovery: `kubectl top pods -n dchat`

**Alert: Bridge Finality Timeout**
1. Check validator health: `curl dchat-api:9001/health/bridge`
2. Verify chain RPC endpoints responding
3. Check consensus state: `kubectl logs deployment/dchat-core | grep "validator\|consensus"`
4. If validator down, notify validator operator
5. May need manual bridge pause/resume

---

## Deployment Checklist

### Pre-Deployment

- [ ] Code reviewed and tested
- [ ] Security audit passed
- [ ] Secrets configured in vault
- [ ] Database migrations tested
- [ ] Backup system verified
- [ ] Monitoring & alerting configured
- [ ] Runbooks prepared
- [ ] On-call rotation established

### Deployment

- [ ] Run pre-flight checks: `kubectl cluster-info`
- [ ] Verify storage available: `kubectl get pvc -n dchat`
- [ ] Create/update ConfigMap: `kubectl apply -f configmap.yaml`
- [ ] Deploy: `kubectl apply -f deployment.yaml`
- [ ] Verify pods running: `kubectl get pods -n dchat`
- [ ] Check endpoints: `kubectl get endpoints -n dchat`
- [ ] Test API: `curl https://api.dchat.dev/health`

### Post-Deployment

- [ ] Monitor for errors: Watch Grafana/Prometheus
- [ ] Check logs: `kubectl logs deployment/dchat-core`
- [ ] Verify metrics flowing: Check Prometheus targets
- [ ] Run smoke tests
- [ ] Announce to team: Status page update
- [ ] Schedule post-mortem if issues

---

## Production SLOs

```
Availability: 99.5%  (4.38 hours downtime/month allowed)
P99 Latency: <500ms
Error Rate: <0.1%
```

**Monitored by**: Prometheus/Grafana + custom metrics

---

**Deployment Status**: ✅ READY FOR MVP  
**Next Review**: After Phase 6 advanced features  
**Escalation**: Page on-call engineer if critical alert fired

