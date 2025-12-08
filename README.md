# Hafiz

<p align="center">
  <img src="docs/logo.png" alt="Hafiz Logo" width="200"/>
</p>

<p align="center">
  <strong>Enterprise S3-Compatible Object Storage</strong>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#installation">Installation</a> •
  <a href="#configuration">Configuration</a> •
  <a href="#api-compatibility">API Compatibility</a> •
  <a href="#admin-ui">Admin UI</a>
</p>

---

## Features

- 🚀 **High Performance** - Written in Rust for maximum performance and memory safety
- 📦 **S3 Compatible** - Drop-in replacement for Amazon S3 (27 endpoints)
- 🔒 **Secure** - AWS Signature V4 auth, TLS/mTLS, SSE-S3 & SSE-C encryption
- 🐳 **Docker Ready** - Production-ready multi-stage Docker builds
- 🎨 **Admin UI** - Modern web interface built with Leptos (Rust WASM)
- 📊 **Observability** - Prometheus metrics, Grafana dashboards, alerting
- 🗄️ **Flexible Storage** - SQLite (development) or PostgreSQL (production)
- ⚡ **Fast** - Async I/O with Tokio runtime

## Quick Start

### Using Docker (Recommended)

```bash
# Pull and run
docker run -d \
  --name hafiz \
  -p 9000:9000 \
  -v hafiz-data:/data/hafiz \
  -e HAFIZ_ROOT_ACCESS_KEY=minioadmin \
  -e HAFIZ_ROOT_SECRET_KEY=minioadmin \
  hafiz/hafiz:latest

# Test with AWS CLI
aws --endpoint-url http://localhost:9000 s3 mb s3://my-bucket
aws --endpoint-url http://localhost:9000 s3 cp file.txt s3://my-bucket/
aws --endpoint-url http://localhost:9000 s3 ls s3://my-bucket/
```

### Using Docker Compose

```bash
# Basic
docker-compose up -d

# With PostgreSQL
docker-compose --profile postgres up -d

# With full monitoring stack
docker-compose --profile monitoring up -d

# Everything
docker-compose --profile postgres --profile monitoring up -d
```

### Building from Source

```bash
# Prerequisites: Rust 1.75+
cargo build --release

# Run
./target/release/hafiz-cli serve
```

## Installation

### Docker

```bash
# Build locally
./scripts/docker-build.sh

# Or pull from registry
docker pull hafiz/hafiz:latest
```

### Kubernetes

```bash
kubectl apply -f deploy/kubernetes/hafiz.yaml
```

### From Source

```bash
# Clone repository
git clone https://github.com/yourorg/hafiz.git
cd hafiz

# Build
cargo build --release

# Install
cargo install --path crates/hafiz-cli
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HAFIZ_BIND_ADDRESS` | `0.0.0.0` | Server bind address |
| `HAFIZ_PORT` | `9000` | S3 API port |
| `HAFIZ_DATA_DIR` | `/data/hafiz` | Data storage directory |
| `HAFIZ_DATABASE_URL` | `sqlite:///data/hafiz/hafiz.db` | Database URL |
| `HAFIZ_ROOT_ACCESS_KEY` | `minioadmin` | Root access key |
| `HAFIZ_ROOT_SECRET_KEY` | `minioadmin` | Root secret key |
| `HAFIZ_LOG_LEVEL` | `info` | Log level |
| `HAFIZ_TLS_CERT` | - | TLS certificate path |
| `HAFIZ_TLS_KEY` | - | TLS private key path |
| `HAFIZ_ENCRYPTION_KEY` | - | Master encryption key (hex) |

### Configuration File

See `config/config.example.toml` for full configuration options.

```toml
[server]
bind_address = "0.0.0.0"
port = 9000

[tls]
enabled = true
cert_file = "/data/hafiz/certs/server.crt"
key_file = "/data/hafiz/certs/server.key"

[storage]
data_dir = "/data/hafiz"

[database]
# SQLite (development)
url = "sqlite:///data/hafiz/hafiz.db?mode=rwc"
# PostgreSQL (production)
# url = "postgresql://user:pass@localhost:5432/hafiz"

[encryption]
enabled = true
sse_s3_enabled = true
sse_c_enabled = true
master_key = "your-32-byte-hex-key"

[auth]
root_access_key = "your-access-key"
root_secret_key = "your-secret-key"
```

### TLS Setup

```bash
# Generate self-signed certificate for development
./scripts/tls-certs.sh generate-self-signed --domain localhost

# Generate CA and server cert for production
./scripts/tls-certs.sh generate-ca
./scripts/tls-certs.sh generate-server --domain storage.example.com

# mTLS: Generate client certificates
./scripts/tls-certs.sh generate-client --client-name app1
```

## API Compatibility

### Supported Operations (27 Endpoints)

#### Bucket Operations
| Operation | Status | Description |
|-----------|--------|-------------|
| `CreateBucket` | ✅ | Create a new bucket |
| `DeleteBucket` | ✅ | Delete an empty bucket |
| `HeadBucket` | ✅ | Check if bucket exists |
| `ListBuckets` | ✅ | List all buckets |
| `GetBucketVersioning` | ✅ | Get versioning status |
| `PutBucketVersioning` | ✅ | Enable/suspend versioning |
| `GetBucketLifecycle` | ✅ | Get lifecycle rules |
| `PutBucketLifecycle` | ✅ | Set lifecycle rules |
| `DeleteBucketLifecycle` | ✅ | Delete lifecycle rules |

#### Object Operations
| Operation | Status | Description |
|-----------|--------|-------------|
| `PutObject` | ✅ | Upload an object |
| `GetObject` | ✅ | Download an object |
| `HeadObject` | ✅ | Get object metadata |
| `DeleteObject` | ✅ | Delete an object |
| `DeleteObjects` | ✅ | Batch delete |
| `CopyObject` | ✅ | Copy object |
| `ListObjects` | ✅ | List objects in bucket |
| `ListObjectsV2` | ✅ | List objects (v2) |
| `ListObjectVersions` | ✅ | List object versions |

#### Multipart Upload
| Operation | Status | Description |
|-----------|--------|-------------|
| `CreateMultipartUpload` | ✅ | Start multipart upload |
| `UploadPart` | ✅ | Upload a part |
| `CompleteMultipartUpload` | ✅ | Complete upload |
| `AbortMultipartUpload` | ✅ | Abort upload |
| `ListMultipartUploads` | ✅ | List active uploads |
| `ListParts` | ✅ | List uploaded parts |

#### Tagging
| Operation | Status | Description |
|-----------|--------|-------------|
| `GetObjectTagging` | ✅ | Get object tags |
| `PutObjectTagging` | ✅ | Set object tags |
| `DeleteObjectTagging` | ✅ | Delete object tags |

### Encryption Support

| Type | Description |
|------|-------------|
| SSE-S3 | Server-managed AES-256-GCM encryption |
| SSE-C | Customer-provided encryption keys |

## Admin UI

Access the admin UI at `http://localhost:9000/admin`

### Features
- 📊 Dashboard with storage statistics
- 🪣 Bucket management (create, delete, versioning)
- 📁 Object browser with drag & drop upload
- 👤 User management (create, delete, enable/disable)
- ⚙️ Settings and configuration

### Screenshots

```
┌─────────────────────────────────────────────────────────────┐
│  🔵 Hafiz                              admin ▼  ⚙️  │
├─────────────────────────────────────────────────────────────┤
│  ┌──────┐  Dashboard                                       │
│  │ 📊  │                                                   │
│  │ 🪣  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐     │
│  │ 📁  │  │ 5      │ │ 1,234  │ │ 45.2GB │ │ 3      │     │
│  │ 👤  │  │Buckets │ │Objects │ │ Used   │ │ Users  │     │
│  │ ⚙️  │  └────────┘ └────────┘ └────────┘ └────────┘     │
│  └──────┘                                                   │
└─────────────────────────────────────────────────────────────┘
```

## Monitoring

### Prometheus Metrics

Available at `/metrics`:

```
# HTTP metrics
hafiz_http_requests_total{method, status}
hafiz_http_request_duration_seconds{method}

# S3 operations
hafiz_s3_operations_total{operation, status}
hafiz_s3_operation_duration_seconds{operation}

# Storage
hafiz_storage_bytes_read_total
hafiz_storage_bytes_written_total
hafiz_storage_objects_total
hafiz_storage_buckets_total
```

### Grafana Dashboard

Pre-built dashboard included at `deploy/grafana/hafiz-dashboard.json`

### Alerting

Alert rules at `deploy/prometheus/hafiz_alerts.yml`:
- High error rate (>5%)
- High latency (p99 > 5s)
- Service down
- SLO breaches

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                       Hafiz                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   S3 API    │  │  Admin API  │  │  Admin UI   │         │
│  │   (Axum)    │  │    REST     │  │  (Leptos)   │         │
│  └──────┬──────┘  └──────┬──────┘  └─────────────┘         │
│         │                │                                  │
│  ┌──────┴────────────────┴──────┐                          │
│  │         Middleware           │                          │
│  │  (Auth, Metrics, TLS, CORS)  │                          │
│  └──────────────┬───────────────┘                          │
│                 │                                           │
│  ┌──────────────┴───────────────┐                          │
│  │        Core Services         │                          │
│  └──┬─────────┬─────────┬───────┘                          │
│     │         │         │                                   │
│  ┌──┴──┐  ┌───┴───┐  ┌──┴──┐                               │
│  │Store│  │  Meta │  │Crypto│                               │
│  │(FS) │  │SQLite/│  │AES   │                               │
│  │     │  │ Postgres│ │GCM  │                               │
│  └─────┘  └───────┘  └─────┘                               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Project Structure

```
hafiz/
├── crates/
│   ├── hafiz-core/      # Core types, config, errors
│   ├── hafiz-crypto/    # Encryption (AES-256-GCM)
│   ├── hafiz-storage/   # Filesystem storage engine
│   ├── hafiz-metadata/  # SQLite/PostgreSQL metadata
│   ├── hafiz-auth/      # AWS Signature V4
│   ├── hafiz-s3-api/    # S3 API server (Axum)
│   ├── hafiz-admin/     # Admin UI (Leptos WASM)
│   └── hafiz-cli/       # CLI binary
├── config/              # Example configurations
├── deploy/
│   ├── prometheus/      # Prometheus config & alerts
│   ├── grafana/         # Dashboards & provisioning
│   ├── alertmanager/    # Alerting configuration
│   ├── kubernetes/      # K8s manifests
│   └── postgres/        # PostgreSQL init scripts
├── scripts/             # Build & utility scripts
├── Dockerfile           # Multi-stage production build
├── docker-compose.yml   # Production deployment
└── docker-compose.dev.yml # Development environment
```

## Roadmap

### Completed ✅
- [x] S3 API (27 endpoints)
- [x] Multipart upload
- [x] Object versioning
- [x] Lifecycle policies
- [x] Server-side encryption (SSE-S3, SSE-C)
- [x] Admin REST API
- [x] Admin Web UI
- [x] PostgreSQL support
- [x] Prometheus metrics
- [x] TLS/HTTPS support
- [x] Docker deployment

### Planned
- [ ] Bucket policies & ACLs
- [ ] Erasure coding
- [ ] Cluster mode & replication
- [ ] LDAP/AD integration
- [ ] Pre-signed URLs
- [ ] Event notifications (S3 Events)

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) first.

## License

Apache License 2.0 - see [LICENSE](LICENSE) for details.

---

<p align="center">
  Made with ❤️ in Rust
</p>
