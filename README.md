<p align="center">
  <img src="docs/assets/logo.svg" alt="Hafiz Logo" width="200">
</p>

<h1 align="center">Hafiz</h1>

<p align="center">
  <strong>Enterprise-grade S3-compatible object storage written in Rust</strong>
</p>

<p align="center">
  <a href="https://github.com/shellnoq/hafiz/actions"><img src="https://github.com/shellnoq/hafiz/workflows/CI/badge.svg" alt="CI Status"></a>
  <a href="https://github.com/shellnoq/hafiz/releases"><img src="https://img.shields.io/github/v/release/shellnoq/hafiz" alt="Release"></a>
  <a href="https://github.com/shellnoq/hafiz/blob/main/LICENSE"><img src="https://img.shields.io/github/license/shellnoq/hafiz" alt="License"></a>
  <a href="https://hub.docker.com/r/shellnoq/hafiz"><img src="https://img.shields.io/docker/pulls/shellnoq/hafiz" alt="Docker Pulls"></a>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#documentation">Documentation</a> •
  <a href="#deployment">Deployment</a> •
  <a href="#contributing">Contributing</a>
</p>

---

**Hafiz** (حافظ - "Guardian" in Arabic/Turkish) is a high-performance, S3-compatible object storage system built from the ground up in Rust. Designed for enterprises that need reliable, secure, and scalable storage without vendor lock-in.

## Why Hafiz?

| Feature | Hafiz | MinIO | AWS S3 |
|---------|-------|-------|--------|
| S3 API Compatible | ✅ | ✅ | ✅ |
| Server-Side Encryption | ✅ AES-256-GCM | ✅ | ✅ |
| Object Lock (WORM) | ✅ SEC 17a-4 | ✅ | ✅ |
| LDAP Integration | ✅ | ✅ Enterprise | ❌ |
| Bucket Policies | ✅ Full IAM | ✅ | ✅ |
| Event Notifications | ✅ Webhook/Kafka/NATS | ✅ | ✅ |
| Written in Rust | ✅ Memory Safe | ❌ Go | ❌ |
| Open Source | ✅ Apache 2.0 | ⚠️ AGPL | ❌ |
| Self-Hosted | ✅ | ✅ | ❌ |

## Features

### 🚀 Core Storage
- **Full S3 API compatibility** - Works with AWS SDKs, CLI, and tools
- **Multi-part uploads** - Handle files up to 5TB
- **Versioning** - Keep object history with MFA delete protection
- **Lifecycle policies** - Automatic expiration and transitions
- **Storage classes** - STANDARD, INTELLIGENT_TIERING, GLACIER simulation

### 🔐 Enterprise Security
- **Server-side encryption** - AES-256-GCM with customer-managed keys
- **Object Lock (WORM)** - SEC 17a-4, FINRA, HIPAA, GDPR compliance
- **Bucket policies** - Fine-grained IAM-style access control
- **LDAP/Active Directory** - Enterprise identity integration
- **TLS everywhere** - End-to-end encryption in transit

### 📊 Operations
- **Admin UI** - Web-based management console
- **Prometheus metrics** - Full observability
- **Event notifications** - Webhook, Kafka, NATS, Redis, AMQP
- **Access logging** - Audit trail for compliance
- **Health checks** - Kubernetes-ready probes

### 🌐 Scalability
- **Cluster mode** - Horizontal scaling with gossip protocol
- **PostgreSQL backend** - Production-grade metadata storage
- **Helm chart** - One-command Kubernetes deployment
- **Docker support** - Container-ready from day one

## Quick Start

### Docker (Fastest)

```bash
# Single node
docker run -d \
  --name hafiz \
  -p 9000:9000 \
  -p 9001:9001 \
  -v hafiz-data:/data \
  -e HAFIZ_ROOT_ACCESS_KEY=minioadmin \
  -e HAFIZ_ROOT_SECRET_KEY=minioadmin \
  ghcr.io/shellnoq/hafiz:latest

# Access
echo "S3 API: http://localhost:9000"
echo "Admin UI: http://localhost:9001"
```

### Docker Compose (Recommended)

```bash
git clone https://github.com/shellnoq/hafiz.git
cd hafiz
docker-compose up -d

# With PostgreSQL and monitoring
docker-compose -f docker-compose.yml -f docker-compose.cluster.yml up -d
```

### Kubernetes (Production)

```bash
# Add Helm repository
helm repo add hafiz https://shellnoq.github.io/hafiz
helm repo update

# Install
helm install hafiz hafiz/hafiz \
  --namespace hafiz \
  --create-namespace \
  -f values-production.yaml
```

### From Source

```bash
# Prerequisites: Rust 1.75+, PostgreSQL (optional)
git clone https://github.com/shellnoq/hafiz.git
cd hafiz

# Build
cargo build --release

# Run
./target/release/hafiz-server
```

## Usage

### AWS CLI

```bash
# Configure
aws configure set aws_access_key_id minioadmin
aws configure set aws_secret_access_key minioadmin

# Create bucket
aws --endpoint-url http://localhost:9000 s3 mb s3://my-bucket

# Upload file
aws --endpoint-url http://localhost:9000 s3 cp file.txt s3://my-bucket/

# List objects
aws --endpoint-url http://localhost:9000 s3 ls s3://my-bucket/
```

### Hafiz CLI

```bash
# Install
cargo install hafiz-cli

# Configure
hafiz configure
# Enter endpoint: http://localhost:9000
# Enter access key: minioadmin
# Enter secret key: minioadmin

# Use
hafiz ls s3://
hafiz mb s3://my-bucket
hafiz cp file.txt s3://my-bucket/
hafiz sync ./local/ s3://my-bucket/backup/
```

### Python (boto3)

```python
import boto3

s3 = boto3.client(
    's3',
    endpoint_url='http://localhost:9000',
    aws_access_key_id='minioadmin',
    aws_secret_access_key='minioadmin'
)

# Create bucket
s3.create_bucket(Bucket='my-bucket')

# Upload file
s3.upload_file('file.txt', 'my-bucket', 'file.txt')

# List objects
response = s3.list_objects_v2(Bucket='my-bucket')
for obj in response.get('Contents', []):
    print(obj['Key'])
```

### JavaScript (AWS SDK v3)

```javascript
import { S3Client, PutObjectCommand } from "@aws-sdk/client-s3";

const client = new S3Client({
  endpoint: "http://localhost:9000",
  region: "us-east-1",
  credentials: {
    accessKeyId: "minioadmin",
    secretAccessKey: "minioadmin",
  },
  forcePathStyle: true,
});

await client.send(new PutObjectCommand({
  Bucket: "my-bucket",
  Key: "hello.txt",
  Body: "Hello, World!",
}));
```

## Documentation

| Document | Description |
|----------|-------------|
| [Architecture](docs/ARCHITECTURE.md) | System design and components |
| [API Reference](docs/API.md) | Complete S3 API documentation |
| [Configuration](docs/CONFIGURATION.md) | All configuration options |
| [Deployment](docs/DEPLOYMENT.md) | Production deployment guide |
| [CLI Reference](docs/CLI.md) | Command-line interface guide |
| [Security](docs/SECURITY.md) | Security features and best practices |
| [Operations](docs/OPERATIONS.md) | Monitoring, backup, troubleshooting |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Load Balancer                          │
└─────────────────────────┬───────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
┌───────▼───────┐ ┌───────▼───────┐ ┌───────▼───────┐
│   Hafiz #1    │ │   Hafiz #2    │ │   Hafiz #3    │
│               │ │               │ │               │
│  ┌─────────┐  │ │  ┌─────────┐  │ │  ┌─────────┐  │
│  │ S3 API  │  │ │  │ S3 API  │  │ │  │ S3 API  │  │
│  ├─────────┤  │ │  ├─────────┤  │ │  ├─────────┤  │
│  │ Admin   │  │ │  │ Admin   │  │ │  │ Admin   │  │
│  ├─────────┤  │ │  ├─────────┤  │ │  ├─────────┤  │
│  │ Auth    │  │ │  │ Auth    │  │ │  │ Auth    │  │
│  ├─────────┤  │ │  ├─────────┤  │ │  ├─────────┤  │
│  │ Storage │  │ │  │ Storage │  │ │  │ Storage │  │
│  └────┬────┘  │ │  └────┬────┘  │ │  └────┬────┘  │
└───────┼───────┘ └───────┼───────┘ └───────┼───────┘
        │                 │                 │
        └─────────────────┼─────────────────┘
                          │
                 ┌────────▼────────┐
                 │   PostgreSQL    │
                 │   (Metadata)    │
                 └─────────────────┘
```

## Project Structure

```
hafiz/
├── crates/
│   ├── hafiz-core/       # Core types, traits, utilities
│   ├── hafiz-s3-api/     # S3 API implementation (Axum)
│   ├── hafiz-storage/    # Storage backends
│   ├── hafiz-metadata/   # Metadata repository
│   ├── hafiz-auth/       # Authentication & authorization
│   ├── hafiz-crypto/     # Encryption & signing
│   ├── hafiz-cluster/    # Cluster coordination
│   ├── hafiz-admin/      # Admin API & UI
│   └── hafiz-cli/        # Command-line interface
├── deploy/
│   ├── helm/             # Kubernetes Helm chart
│   ├── docker/           # Docker configurations
│   ├── prometheus/       # Monitoring configs
│   └── grafana/          # Dashboards
├── docs/                 # Documentation
├── scripts/              # Build & deployment scripts
└── tests/                # Integration tests
```

## Configuration

### Environment Variables

```bash
# Required
HAFIZ_ROOT_ACCESS_KEY=your-access-key
HAFIZ_ROOT_SECRET_KEY=your-secret-key

# Optional
HAFIZ_S3_PORT=9000
HAFIZ_ADMIN_PORT=9001
HAFIZ_LOG_LEVEL=info
HAFIZ_STORAGE_BASE_PATH=/data
HAFIZ_DATABASE_URL=postgres://user:pass@host/db

# Encryption
HAFIZ_ENCRYPTION_ENABLED=true
HAFIZ_ENCRYPTION_MASTER_KEY=base64-encoded-32-byte-key

# Cluster
HAFIZ_CLUSTER_ENABLED=true
HAFIZ_CLUSTER_PEERS=node1:7946,node2:7946,node3:7946
```

See [Configuration Reference](docs/CONFIGURATION.md) for all options.

## Deployment Options

| Method | Use Case | Complexity |
|--------|----------|------------|
| Docker | Development, small deployments | ⭐ |
| Docker Compose | Testing, staging | ⭐⭐ |
| Kubernetes (Helm) | Production | ⭐⭐⭐ |
| Binary | Bare metal, custom setups | ⭐⭐ |

See [Deployment Guide](docs/DEPLOYMENT.md) for detailed instructions.

## Compliance

Hafiz is designed to meet regulatory requirements:

| Regulation | Feature | Status |
|------------|---------|--------|
| **SEC 17a-4** | Object Lock (WORM) | ✅ |
| **FINRA 4511** | Immutable records | ✅ |
| **HIPAA** | Encryption, audit logs | ✅ |
| **GDPR** | Data encryption, access control | ✅ |
| **SOC 2** | Access logging, encryption | ✅ |

## Performance

Benchmarks on AWS c5.2xlarge (8 vCPU, 16GB RAM):

| Operation | Throughput | Latency (p99) |
|-----------|------------|---------------|
| PUT (1MB) | 850 ops/s | 12ms |
| GET (1MB) | 1,200 ops/s | 8ms |
| LIST (1000 objects) | 500 ops/s | 25ms |
| DELETE | 2,000 ops/s | 5ms |

## Roadmap

### v0.2.0 (Q1 2025)
- [ ] S3 Select (SQL queries on objects)
- [ ] Cross-region replication
- [ ] Web UI improvements

### v0.3.0 (Q2 2025)
- [ ] Erasure coding
- [ ] Tiered storage
- [ ] Terraform provider

### v1.0.0 (Q3 2025)
- [ ] Production-ready release
- [ ] Long-term support
- [ ] Enterprise features

See [ROADMAP.md](ROADMAP.md) for full details.

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

```bash
# Clone
git clone https://github.com/shellnoq/hafiz.git
cd hafiz

# Setup development environment
make dev-setup

# Run tests
make test

# Run locally
make run
```

## Community

- 💬 [GitHub Discussions](https://github.com/shellnoq/hafiz/discussions)
- 🐛 [Issue Tracker](https://github.com/shellnoq/hafiz/issues)
- 📧 [Email](mailto:hello@shellnoq.com)

## License

Hafiz is licensed under the [Apache License 2.0](LICENSE).

```
Copyright 2024 Shellnoq

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0
```

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) 🦀
- Inspired by [MinIO](https://min.io/) and [SeaweedFS](https://github.com/seaweedfs/seaweedfs)
- S3 API specification by [Amazon Web Services](https://docs.aws.amazon.com/s3/)

---

<p align="center">
  Made with ❤️ by <a href="https://github.com/shellnoq">Shellnoq</a>
</p>
