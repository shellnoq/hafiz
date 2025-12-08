---
title: Monitoring
description: Monitoring Hafiz with Prometheus and Grafana
---

# Monitoring

## Prometheus Metrics

Hafiz exposes metrics at `/metrics` on port 9090.

### Enable Metrics

```bash
HAFIZ_METRICS_ENABLED=true
HAFIZ_METRICS_PORT=9090
```

### Key Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `hafiz_requests_total` | Counter | Total requests |
| `hafiz_request_duration_seconds` | Histogram | Request latency |
| `hafiz_objects_total` | Gauge | Object count |
| `hafiz_storage_bytes` | Gauge | Storage used |
| `hafiz_active_connections` | Gauge | Active connections |

### Prometheus Config

```yaml
scrape_configs:
  - job_name: 'hafiz'
    static_configs:
      - targets: ['hafiz:9090']
```

## Grafana Dashboard

Import the dashboard from:
```
deploy/grafana/dashboards/hafiz.json
```

## Alerts

Example PrometheusRule:

```yaml
groups:
  - name: hafiz
    rules:
      - alert: HafizHighErrorRate
        expr: rate(hafiz_requests_total{status="error"}[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
```

## Health Checks

```bash
# Health
curl http://localhost:9000/health

# Readiness
curl http://localhost:9000/ready

# Liveness
curl http://localhost:9000/live
```
