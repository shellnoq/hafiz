# Hafiz Helm Chart

A Helm chart for deploying Hafiz S3-compatible object storage on Kubernetes.

## Prerequisites

- Kubernetes 1.23+
- Helm 3.8+
- PV provisioner support in the cluster (for persistence)

## Installation

### Quick Start (Development)

```bash
# Add the Hafiz Helm repository (when published)
helm repo add hafiz https://yourorg.github.io/hafiz
helm repo update

# Install with development values
helm install hafiz hafiz/hafiz -f values-development.yaml
```

### Production Installation

```bash
# Create namespace
kubectl create namespace hafiz

# Create secrets first
kubectl create secret generic hafiz-root-credentials \
  --namespace hafiz \
  --from-literal=access-key=$(openssl rand -hex 10) \
  --from-literal=secret-key=$(openssl rand -hex 20)

kubectl create secret generic hafiz-admin-credentials \
  --namespace hafiz \
  --from-literal=admin-password=$(openssl rand -hex 12)

kubectl create secret generic hafiz-encryption-key \
  --namespace hafiz \
  --from-literal=master-key=$(openssl rand -base64 32)

# Install with production values
helm install hafiz hafiz/hafiz \
  --namespace hafiz \
  -f values-production.yaml
```

### Local Development (from source)

```bash
cd deploy/helm
helm install hafiz ./hafiz -f hafiz/values-development.yaml
```

## Configuration

See `values.yaml` for full configuration options. Key parameters:

### Hafiz Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `hafiz.logLevel` | Log level (trace, debug, info, warn, error) | `info` |
| `hafiz.s3.port` | S3 API port | `9000` |
| `hafiz.s3.region` | AWS region | `us-east-1` |
| `hafiz.admin.enabled` | Enable admin API | `true` |
| `hafiz.storage.type` | Storage backend (filesystem, s3) | `filesystem` |
| `hafiz.metadata.type` | Metadata backend (sqlite, postgresql) | `postgresql` |
| `hafiz.encryption.enabled` | Enable server-side encryption | `true` |
| `hafiz.cluster.enabled` | Enable cluster mode | `true` |

### Kubernetes Resources

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of replicas | `3` |
| `resources.limits.cpu` | CPU limit | `2000m` |
| `resources.limits.memory` | Memory limit | `4Gi` |
| `persistence.enabled` | Enable persistence | `true` |
| `persistence.size` | Storage size | `100Gi` |
| `autoscaling.enabled` | Enable HPA | `true` |

### Ingress

| Parameter | Description | Default |
|-----------|-------------|---------|
| `ingress.enabled` | Enable ingress | `false` |
| `ingress.className` | Ingress class | `nginx` |
| `ingress.hosts[0].host` | Hostname | `s3.example.com` |

### PostgreSQL

| Parameter | Description | Default |
|-----------|-------------|---------|
| `postgresql.enabled` | Enable PostgreSQL subchart | `true` |
| `postgresql.auth.database` | Database name | `hafiz` |
| `postgresql.external.host` | External PostgreSQL host | `""` |

## Upgrading

```bash
helm upgrade hafiz hafiz/hafiz --namespace hafiz -f values-production.yaml
```

## Uninstalling

```bash
helm uninstall hafiz --namespace hafiz

# Optional: Remove PVCs
kubectl delete pvc -l app.kubernetes.io/name=hafiz --namespace hafiz
```

## Architecture

```
                    ┌─────────────┐
                    │   Ingress   │
                    └──────┬──────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
        ┌─────▼─────┐┌─────▼─────┐┌─────▼─────┐
        │  Hafiz-0  ││  Hafiz-1  ││  Hafiz-2  │
        │           ││           ││           │
        │  S3 API   ││  S3 API   ││  S3 API   │
        │  Admin    ││  Admin    ││  Admin    │
        │  Gossip   ││  Gossip   ││  Gossip   │
        └─────┬─────┘└─────┬─────┘└─────┬─────┘
              │            │            │
              └────────────┼────────────┘
                           │
                    ┌──────▼──────┐
                    │ PostgreSQL  │
                    └─────────────┘
```

## Security

### Secrets Management

For production, use external secrets management:

```yaml
hafiz:
  auth:
    existingSecret: "vault-hafiz-auth"
  encryption:
    existingSecret: "vault-hafiz-encryption"
```

### Network Policies

Enable network policies for production:

```yaml
networkPolicy:
  enabled: true
```

### Pod Security

The chart runs with restricted security context by default:
- Non-root user (UID 1000)
- Read-only root filesystem
- No privilege escalation
- Dropped capabilities

## Monitoring

### Prometheus Metrics

Enable ServiceMonitor for Prometheus Operator:

```yaml
metrics:
  enabled: true
  serviceMonitor:
    enabled: true
```

### Grafana Dashboard

Import the Hafiz dashboard from `deploy/grafana/dashboards/`.

## Troubleshooting

### Check pod logs
```bash
kubectl logs -f -l app.kubernetes.io/name=hafiz --namespace hafiz
```

### Check cluster status
```bash
kubectl exec -it hafiz-0 --namespace hafiz -- /bin/sh -c "curl localhost:9000/cluster/status"
```

### Debug mode
```yaml
hafiz:
  logLevel: debug
```

## License

Apache 2.0
