# Hafiz Deployment Guide

This guide covers deploying Hafiz in various environments.

## Deployment Options

| Method | Use Case | Difficulty |
|--------|----------|------------|
| Docker | Development, small scale | ⭐ Easy |
| Docker Compose | Testing, staging | ⭐⭐ Medium |
| Kubernetes | Production | ⭐⭐⭐ Advanced |
| Binary | Bare metal | ⭐⭐ Medium |

---

## Docker (Quick Start)

### Single Node

```bash
docker run -d \
  --name hafiz \
  -p 9000:9000 \
  -p 9001:9001 \
  -v hafiz-data:/data \
  -e HAFIZ_ROOT_ACCESS_KEY=minioadmin \
  -e HAFIZ_ROOT_SECRET_KEY=minioadmin \
  ghcr.io/shellnoq/hafiz:latest
```

### With PostgreSQL

```bash
# Create network
docker network create hafiz-net

# Start PostgreSQL
docker run -d \
  --name postgres \
  --network hafiz-net \
  -e POSTGRES_USER=hafiz \
  -e POSTGRES_PASSWORD=secret \
  -e POSTGRES_DB=hafiz \
  -v postgres-data:/var/lib/postgresql/data \
  postgres:15

# Start Hafiz
docker run -d \
  --name hafiz \
  --network hafiz-net \
  -p 9000:9000 \
  -p 9001:9001 \
  -v hafiz-data:/data \
  -e HAFIZ_ROOT_ACCESS_KEY=minioadmin \
  -e HAFIZ_ROOT_SECRET_KEY=minioadmin \
  -e HAFIZ_DATABASE_URL=postgres://hafiz:secret@postgres/hafiz \
  ghcr.io/shellnoq/hafiz:latest
```

---

## Docker Compose

### Development

```bash
git clone https://github.com/shellnoq/hafiz.git
cd hafiz

# Start (single node, SQLite)
docker-compose up -d

# View logs
docker-compose logs -f
```

### Production Cluster

```bash
# Start 3-node cluster with PostgreSQL
docker-compose -f docker-compose.cluster.yml up -d

# Scale to 5 nodes
docker-compose -f docker-compose.cluster.yml up -d --scale hafiz=5
```

### docker-compose.yml (Simple)

```yaml
version: '3.8'
services:
  hafiz:
    image: ghcr.io/shellnoq/hafiz:latest
    ports:
      - "9000:9000"
      - "9001:9001"
    volumes:
      - hafiz-data:/data
    environment:
      - HAFIZ_ROOT_ACCESS_KEY=minioadmin
      - HAFIZ_ROOT_SECRET_KEY=minioadmin

volumes:
  hafiz-data:
```

---

## Kubernetes (Production)

### Prerequisites

- Kubernetes 1.23+
- Helm 3.8+
- kubectl configured

### Installation

```bash
# Add Helm repository
helm repo add hafiz https://shellnoq.github.io/hafiz
helm repo update

# Create namespace
kubectl create namespace hafiz

# Create secrets
kubectl create secret generic hafiz-credentials \
  --namespace hafiz \
  --from-literal=access-key=$(openssl rand -hex 10) \
  --from-literal=secret-key=$(openssl rand -hex 20)

kubectl create secret generic hafiz-encryption \
  --namespace hafiz \
  --from-literal=master-key=$(openssl rand -base64 32)

# Install
helm install hafiz hafiz/hafiz \
  --namespace hafiz \
  --set hafiz.auth.existingSecret=hafiz-credentials \
  --set hafiz.encryption.existingSecret=hafiz-encryption \
  --set replicaCount=3
```

### Production Values

```yaml
# values-production.yaml
replicaCount: 5

resources:
  limits:
    cpu: 4000m
    memory: 8Gi
  requests:
    cpu: 1000m
    memory: 2Gi

persistence:
  enabled: true
  storageClassName: fast-ssd
  size: 500Gi

ingress:
  enabled: true
  className: nginx
  hosts:
    - host: s3.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: hafiz-tls
      hosts:
        - s3.example.com

postgresql:
  enabled: false
  external:
    host: postgres.database.svc
    database: hafiz
    existingSecret: hafiz-postgres

autoscaling:
  enabled: true
  minReplicas: 5
  maxReplicas: 20

affinity:
  podAntiAffinity:
    requiredDuringSchedulingIgnoredDuringExecution:
      - labelSelector:
          matchLabels:
            app.kubernetes.io/name: hafiz
        topologyKey: kubernetes.io/hostname
```

```bash
helm install hafiz hafiz/hafiz \
  --namespace hafiz \
  -f values-production.yaml
```

### Verify Installation

```bash
# Check pods
kubectl get pods -n hafiz

# Check services
kubectl get svc -n hafiz

# Get credentials
kubectl get secret hafiz-credentials -n hafiz -o jsonpath='{.data.access-key}' | base64 -d
kubectl get secret hafiz-credentials -n hafiz -o jsonpath='{.data.secret-key}' | base64 -d

# Port forward for testing
kubectl port-forward svc/hafiz 9000:9000 -n hafiz
```

---

## Binary Installation

### Download

```bash
# Linux amd64
curl -LO https://github.com/shellnoq/hafiz/releases/latest/download/hafiz-linux-amd64.tar.gz
tar xzf hafiz-linux-amd64.tar.gz
sudo mv hafiz-server /usr/local/bin/
sudo mv hafiz /usr/local/bin/
```

### Build from Source

```bash
# Install Rust 1.75+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/shellnoq/hafiz.git
cd hafiz
cargo build --release

# Install
sudo cp target/release/hafiz-server /usr/local/bin/
sudo cp target/release/hafiz /usr/local/bin/
```

### Systemd Service

```ini
# /etc/systemd/system/hafiz.service
[Unit]
Description=Hafiz S3-compatible Object Storage
After=network.target postgresql.service

[Service]
Type=simple
User=hafiz
Group=hafiz
ExecStart=/usr/local/bin/hafiz-server
Restart=always
RestartSec=5

Environment=HAFIZ_ROOT_ACCESS_KEY=your-access-key
Environment=HAFIZ_ROOT_SECRET_KEY=your-secret-key
Environment=HAFIZ_STORAGE_BASE_PATH=/var/lib/hafiz
Environment=HAFIZ_DATABASE_URL=postgres://hafiz:secret@localhost/hafiz

[Install]
WantedBy=multi-user.target
```

```bash
# Create user
sudo useradd -r -s /bin/false hafiz
sudo mkdir -p /var/lib/hafiz
sudo chown hafiz:hafiz /var/lib/hafiz

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable hafiz
sudo systemctl start hafiz
```

---

## Load Balancer Setup

### nginx

```nginx
upstream hafiz {
    server hafiz1:9000;
    server hafiz2:9000;
    server hafiz3:9000;
}

server {
    listen 443 ssl http2;
    server_name s3.example.com;

    ssl_certificate /etc/nginx/ssl/cert.pem;
    ssl_certificate_key /etc/nginx/ssl/key.pem;

    client_max_body_size 5G;
    proxy_request_buffering off;

    location / {
        proxy_pass http://hafiz;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 300s;
        proxy_send_timeout 300s;
    }
}
```

---

## Health Checks

```bash
# Health endpoint
curl http://localhost:9000/health

# Readiness
curl http://localhost:9000/ready

# Liveness
curl http://localhost:9000/live
```

---

## Monitoring Setup

### Prometheus

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'hafiz'
    static_configs:
      - targets: ['hafiz:9090']
```

### Grafana Dashboard

Import dashboard from `deploy/grafana/dashboards/hafiz.json`

---

## Backup & Restore

### Backup

```bash
# Metadata (PostgreSQL)
pg_dump -h localhost -U hafiz hafiz > backup.sql

# Data (filesystem)
tar czf hafiz-data-$(date +%Y%m%d).tar.gz /data
```

### Restore

```bash
# Metadata
psql -h localhost -U hafiz hafiz < backup.sql

# Data
tar xzf hafiz-data-20240101.tar.gz -C /
```

---

## Troubleshooting

### Common Issues

**Connection refused:**
```bash
# Check if service is running
systemctl status hafiz

# Check port binding
ss -tlnp | grep 9000
```

**Authentication failed:**
```bash
# Verify credentials
echo $HAFIZ_ROOT_ACCESS_KEY
echo $HAFIZ_ROOT_SECRET_KEY
```

**Database connection:**
```bash
# Test PostgreSQL connection
psql -h localhost -U hafiz -d hafiz -c "SELECT 1"
```

### Logs

```bash
# Docker
docker logs hafiz

# Kubernetes
kubectl logs -f deployment/hafiz -n hafiz

# Systemd
journalctl -u hafiz -f
```
