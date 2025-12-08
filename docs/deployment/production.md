---
title: Production Deployment
description: Best practices for production
---

# Production Deployment

## Checklist

- [ ] TLS enabled
- [ ] Encryption enabled
- [ ] Strong credentials
- [ ] PostgreSQL backend
- [ ] Resource limits set
- [ ] Monitoring enabled
- [ ] Backup configured
- [ ] Network policies

## Security

### TLS

```bash
HAFIZ_TLS_ENABLED=true
HAFIZ_TLS_CERT_PATH=/etc/hafiz/tls.crt
HAFIZ_TLS_KEY_PATH=/etc/hafiz/tls.key
```

### Encryption

```bash
HAFIZ_ENCRYPTION_ENABLED=true
HAFIZ_ENCRYPTION_MASTER_KEY=$(openssl rand -base64 32)
```

### Credentials

```bash
# Generate strong credentials
HAFIZ_ROOT_ACCESS_KEY=$(openssl rand -hex 16)
HAFIZ_ROOT_SECRET_KEY=$(openssl rand -hex 32)
```

## High Availability

### Multiple Replicas

```yaml
replicaCount: 5

affinity:
  podAntiAffinity:
    requiredDuringSchedulingIgnoredDuringExecution:
      - labelSelector:
          matchLabels:
            app: hafiz
        topologyKey: kubernetes.io/hostname
```

### Pod Disruption Budget

```yaml
podDisruptionBudget:
  enabled: true
  minAvailable: 3
```

## Monitoring

Enable Prometheus metrics:

```yaml
metrics:
  enabled: true
  serviceMonitor:
    enabled: true
```

## Backup

### Metadata

```bash
pg_dump -h postgres -U hafiz hafiz > backup.sql
```

### Data

```bash
# Use cloud provider snapshots
# Or replicate to another Hafiz cluster
```
