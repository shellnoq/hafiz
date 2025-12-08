# Hafiz Helm Chart

Enterprise S3-Compatible Object Storage for Kubernetes.

## Prerequisites

- Kubernetes 1.19+
- Helm 3.2.0+
- PV provisioner support (for persistence)

## Installation

### Add Helm Repository

```bash
helm repo add hafiz https://hafiz.github.io/charts
helm repo update
```

### Install Chart

```bash
# Basic installation
helm install my-hafiz hafiz/hafiz

# With custom values
helm install my-hafiz hafiz/hafiz -f values.yaml

# In a specific namespace
helm install my-hafiz hafiz/hafiz -n storage --create-namespace
```

### Install from Source

```bash
git clone https://github.com/hafiz/hafiz.git
cd hafiz/deploy/helm
helm install my-hafiz ./hafiz
```

## Configuration

See [values.yaml](values.yaml) for the full list of configurable parameters.

### Common Configurations

#### Minimal Production Setup

```yaml
replicaCount: 1

persistence:
  enabled: true
  size: 100Gi

auth:
  rootAccessKey: "myaccesskey"
  rootSecretKey: "mysecretkey123"

resources:
  requests:
    cpu: 500m
    memory: 1Gi
  limits:
    cpu: 2000m
    memory: 4Gi
```

#### High Availability (Cluster Mode)

```yaml
replicaCount: 3

cluster:
  enabled: true

persistence:
  enabled: true
  size: 500Gi
  storageClass: "fast-ssd"

resources:
  requests:
    cpu: 1000m
    memory: 2Gi
  limits:
    cpu: 4000m
    memory: 8Gi
```

#### With Ingress (TLS)

```yaml
ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  hosts:
    - host: s3.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: hafiz-tls
      hosts:
        - s3.example.com

adminIngress:
  enabled: true
  className: nginx
  hosts:
    - host: admin.s3.example.com
      paths:
        - path: /
          pathType: Prefix
```

#### With PostgreSQL

```yaml
database:
  type: postgres
  postgres:
    host: "postgres.database.svc"
    port: 5432
    database: "hafiz"
    username: "hafiz"
    password: "secretpassword"
    sslMode: "require"
```

#### With LDAP Authentication

```yaml
ldap:
  enabled: true
  serverUrl: "ldaps://dc.example.com:636"
  serverType: "active_directory"
  bindDn: "CN=service,OU=Users,DC=example,DC=com"
  bindPassword: "ldappassword"
  userBaseDn: "OU=Users,DC=example,DC=com"
  userFilter: "(sAMAccountName={username})"
  groupBaseDn: "OU=Groups,DC=example,DC=com"
  groupPolicies: |
    {
      "Domain Admins": ["admin"],
      "S3 Users": ["readwrite"]
    }
```

#### With Encryption

```yaml
storage:
  encryption:
    enabled: true
    masterKey: "your-32-character-encryption-key"
```

### Using External Secrets

For production, use existing Kubernetes secrets:

```yaml
auth:
  existingSecret: "hafiz-credentials"
  accessKeyKey: "access-key"
  secretKeyKey: "secret-key"

database:
  postgres:
    existingSecret: "postgres-credentials"
    passwordKey: "password"
```

## Upgrading

```bash
helm upgrade my-hafiz hafiz/hafiz -f values.yaml
```

## Uninstalling

```bash
helm uninstall my-hafiz

# Note: PVCs are not deleted automatically
kubectl delete pvc -l app.kubernetes.io/instance=my-hafiz
```

## Parameters

### Global Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of replicas | `1` |
| `image.repository` | Image repository | `hafiz/hafiz` |
| `image.tag` | Image tag | `""` (uses appVersion) |
| `image.pullPolicy` | Image pull policy | `IfNotPresent` |

### Service Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `service.type` | Service type | `ClusterIP` |
| `service.port` | S3 API port | `9000` |
| `adminService.enabled` | Enable admin service | `true` |
| `adminService.port` | Admin console port | `9001` |

### Persistence Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `persistence.enabled` | Enable persistence | `true` |
| `persistence.storageClass` | Storage class | `""` |
| `persistence.size` | PVC size | `100Gi` |
| `persistence.accessModes` | Access modes | `["ReadWriteOnce"]` |

### Authentication Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `auth.rootAccessKey` | Root access key | `minioadmin` |
| `auth.rootSecretKey` | Root secret key | `minioadmin` |
| `auth.existingSecret` | Use existing secret | `""` |

### Cluster Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `cluster.enabled` | Enable cluster mode | `false` |
| `cluster.port` | Cluster communication port | `9100` |

### Metrics Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `metrics.enabled` | Enable Prometheus metrics | `true` |
| `metrics.serviceMonitor.enabled` | Create ServiceMonitor | `false` |

## Troubleshooting

### Check Pod Status

```bash
kubectl get pods -l app.kubernetes.io/instance=my-hafiz
kubectl logs -f deployment/my-hafiz
```

### Check Service

```bash
kubectl get svc -l app.kubernetes.io/instance=my-hafiz
```

### Port Forward for Testing

```bash
kubectl port-forward svc/my-hafiz 9000:9000
aws --endpoint-url http://localhost:9000 s3 ls
```

## License

Apache 2.0
