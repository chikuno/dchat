# Prometheus Operator Configuration for dchat Monitoring

## Installation

Install Prometheus Operator using Helm:

```bash
# Add prometheus-community Helm repository
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update

# Install kube-prometheus-stack (includes Prometheus, Grafana, AlertManager)
helm install monitoring prometheus-community/kube-prometheus-stack \
  --namespace monitoring \
  --create-namespace \
  --values monitoring/prometheus-values.yaml
```

## Components

- **Prometheus**: Metrics collection and storage
- **Grafana**: Visualization and dashboards
- **AlertManager**: Alert routing and notifications
- **ServiceMonitor**: Automatic dchat metrics discovery

## Configuration Files

- `prometheus-values.yaml`: Prometheus Operator Helm values
- `alert-rules.yaml`: Alert rules for dchat monitoring
- `dashboards/`: Grafana dashboard definitions
- `pagerduty-integration.yaml`: PagerDuty integration for alerting

## Access

### Prometheus UI
```bash
kubectl port-forward -n monitoring svc/monitoring-kube-prometheus-prometheus 9090:9090
# Visit http://localhost:9090
```

### Grafana UI
```bash
kubectl port-forward -n monitoring svc/monitoring-grafana 3000:80
# Visit http://localhost:3000
# Default credentials: admin / prom-operator
```

### AlertManager UI
```bash
kubectl port-forward -n monitoring svc/monitoring-kube-prometheus-alertmanager 9093:9093
# Visit http://localhost:9093
```

## Metrics Endpoints

dchat exposes Prometheus metrics on port 9090:
- `/metrics` - Application metrics (message throughput, latency, errors)
- `/health` - Health check endpoint
- `/ready` - Readiness check endpoint

## Key Metrics

- `dchat_messages_sent_total` - Total messages sent
- `dchat_messages_received_total` - Total messages received
- `dchat_message_latency_seconds` - Message delivery latency histogram
- `dchat_relay_connections_active` - Active relay connections
- `dchat_database_query_duration_seconds` - Database query latency
- `dchat_error_rate` - Error rate by type
