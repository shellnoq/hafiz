# Hafiz

<div align="center">

```
    _   _         __ _     
   | | | |  __ _ / _(_)____
   | |_| | / _` | |_| |_  /
   |  _  || (_| |  _| |/ / 
   |_| |_| \__,_|_| |_/___|
```

**Enterprise S3-Compatible Object Storage**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](https://hub.docker.com/r/hafiz/hafiz)

[Features](#features) • [Quick Start](#quick-start) • [Documentation](#documentation) • [API Reference](#api-reference) • [Contributing](#contributing)

</div>

---

## Overview

Hafiz is a high-performance, S3-compatible object storage server written in Rust. It provides a drop-in replacement for Amazon S3 with enterprise features including encryption, clustering, LDAP integration, and regulatory compliance (WORM/Object Lock).

### Why Hafiz?

- **🚀 High Performance**: Built with Rust and async I/O for maximum throughput
- **🔒 Enterprise Security**: Server-side encryption, TLS, mTLS, LDAP/AD integration
- **📦 S3 Compatible**: Works with AWS SDKs, CLI, and existing S3 tools
- **🏢 Compliance Ready**: Object Lock (WORM) for SEC 17a-4, FINRA, HIPAA
- **☁️ Cloud Native**: Kubernetes-ready with Helm charts, Prometheus metrics
- **🔄 Distributed**: Optional clustering for high availability

## Features

### Core S3 Features
- ✅ Bucket operations (Create, Delete, List, Head)
- ✅ Object operations (Put, Get, Delete, Copy, Head)
- ✅ Multipart uploads (large file support)
- ✅ Versioning (full version history)
- ✅ Presigned URLs (temporary access)
- ✅ CORS (browser access)
- ✅ Object tagging

### Enterprise Features
- ✅ **Server-Side Encryption** (AES-256-GCM)
- ✅ **Access Control** (IAM policies, bucket policies)
- ✅ **LDAP/Active Directory** integration
- ✅ **Object Lock / WORM** (regulatory compliance)
- ✅ **Event Notifications** (webhooks, Kafka, NATS)
- ✅ **Replication** (bucket-level async replication)
- ✅ **Lifecycle Rules** (automatic expiration, transitions)

### Operations
- ✅ **Prometheus Metrics** (observability)
- ✅ **Admin UI** (web-based management)
- ✅ **CLI Tool** (command-line client)
- ✅ **Helm Charts** (Kubernetes deployment)
- ✅ **Docker Support** (containerized deployment)

## Quick Start

### Using Docker

```bash
# Run with default settings
docker run -d \
  -p 9000:9000 \
  -p 9001:9001 \
  -v hafiz-data:/data \
  -e HAFIZ_ROOT_ACCESS_KEY=minioadmin \
  -e HAFIZ_ROOT_SECRET_KEY=minioadmin \
  hafiz/hafiz

# Access
# S3 API: http://localhost:9000
# Admin UI: http://localhost:9001
```

### Using Docker Compose

```bash
git clone https://github.com/hafiz/hafiz.git
cd hafiz
docker-compose up -d
```

### Using Helm (Kubernetes)

```bash
helm repo add hafiz https://hafiz.github.io/charts
helm install my-hafiz hafiz/hafiz \
  --set auth.rootAccessKey=myaccesskey \
  --set auth.rootSecretKey=mysecretkey \
  --set persistence.size=100Gi
```

### From Source

```bash
# Prerequisites: Rust 1.75+
git clone https://github.com/hafiz/hafiz.git
cd hafiz
cargo build --release

# Run
./target/release/hafiz server
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `HAFIZ_BIND_ADDRESS` | Server bind address | `0.0.0.0` |
| `HAFIZ_PORT` | S3 API port | `9000` |
| `HAFIZ_ADMIN_PORT` | Admin UI port | `9001` |
| `HAFIZ_DATA_DIR` | Data storage directory | `./data` |
| `HAFIZ_ROOT_ACCESS_KEY` | Root access key | `minioadmin` |
| `HAFIZ_ROOT_SECRET_KEY` | Root secret key | `minioadmin` |
| `HAFIZ_DATABASE_URL` | Database connection | `sqlite://./data/metadata.db` |
| `HAFIZ_LOG_LEVEL` | Log level | `info` |

### Configuration File

Create `config.toml`:

```toml
[server]
bind_address = "0.0.0.0"
port = 9000
admin_port = 9001

[storage]
data_dir = "/data/hafiz"

[storage.encryption]
enabled = true
master_key_path = "/etc/hafiz/master.key"

[database]
url = "sqlite:///data/hafiz/metadata.db"

[auth]
root_access_key = "minioadmin"
root_secret_key = "minioadmin"

[tls]
enabled = true
cert_path = "/etc/hafiz/tls/server.crt"
key_path = "/etc/hafiz/tls/server.key"

[ldap]
enabled = true
server_url = "ldaps://dc.example.com:636"
bind_dn = "CN=service,OU=Users,DC=example,DC=com"
bind_password = "password"
user_base_dn = "OU=Users,DC=example,DC=com"
```

Run with config:
```bash
hafiz server --config config.toml
```

## Using Hafiz

### AWS CLI

```bash
# Configure
aws configure set aws_access_key_id minioadmin
aws configure set aws_secret_access_key minioadmin

# Create bucket
aws --endpoint-url http://localhost:9000 s3 mb s3://mybucket

# Upload file
aws --endpoint-url http://localhost:9000 s3 cp myfile.txt s3://mybucket/

# List objects
aws --endpoint-url http://localhost:9000 s3 ls s3://mybucket/

# Download file
aws --endpoint-url http://localhost:9000 s3 cp s3://mybucket/myfile.txt ./downloaded.txt
```

### Hafiz CLI

```bash
# Configure alias
hafiz alias set local http://localhost:9000 minioadmin minioadmin

# List buckets
hafiz ls local

# Create bucket
hafiz mb local/mybucket

# Upload file
hafiz cp myfile.txt local/mybucket/

# Upload directory
hafiz cp -r ./mydir local/mybucket/backup/

# Download file
hafiz cp local/mybucket/myfile.txt ./

# View file content
hafiz cat local/mybucket/config.json

# Delete file
hafiz rm local/mybucket/myfile.txt

# Delete bucket (with all contents)
hafiz rb --force local/mybucket
```

### AWS SDK (Python)

```python
import boto3

s3 = boto3.client(
    's3',
    endpoint_url='http://localhost:9000',
    aws_access_key_id='minioadmin',
    aws_secret_access_key='minioadmin'
)

# Create bucket
s3.create_bucket(Bucket='mybucket')

# Upload file
s3.upload_file('myfile.txt', 'mybucket', 'myfile.txt')

# Download file
s3.download_file('mybucket', 'myfile.txt', 'downloaded.txt')

# List objects
response = s3.list_objects_v2(Bucket='mybucket')
for obj in response.get('Contents', []):
    print(obj['Key'], obj['Size'])
```

### AWS SDK (JavaScript/Node.js)

```javascript
const { S3Client, PutObjectCommand, GetObjectCommand } = require('@aws-sdk/client-s3');

const client = new S3Client({
  endpoint: 'http://localhost:9000',
  region: 'us-east-1',
  credentials: {
    accessKeyId: 'minioadmin',
    secretAccessKey: 'minioadmin'
  },
  forcePathStyle: true
});

// Upload
await client.send(new PutObjectCommand({
  Bucket: 'mybucket',
  Key: 'myfile.txt',
  Body: 'Hello, World!'
}));

// Download
const response = await client.send(new GetObjectCommand({
  Bucket: 'mybucket',
  Key: 'myfile.txt'
}));
const body = await response.Body.transformToString();
```

## Enterprise Features

### Server-Side Encryption

Enable encryption in config:

```toml
[storage.encryption]
enabled = true
```

All objects are automatically encrypted with AES-256-GCM.

### Object Lock (WORM)

For regulatory compliance (SEC 17a-4, FINRA, HIPAA):

```bash
# Enable Object Lock on bucket (at creation)
aws --endpoint-url http://localhost:9000 s3api create-bucket \
  --bucket compliance-bucket \
  --object-lock-enabled-for-bucket

# Set default retention
aws --endpoint-url http://localhost:9000 s3api put-object-lock-configuration \
  --bucket compliance-bucket \
  --object-lock-configuration '{
    "ObjectLockEnabled": "Enabled",
    "Rule": {
      "DefaultRetention": {
        "Mode": "COMPLIANCE",
        "Days": 2555
      }
    }
  }'

# Set legal hold on object
aws --endpoint-url http://localhost:9000 s3api put-object-legal-hold \
  --bucket compliance-bucket \
  --key evidence.pdf \
  --legal-hold '{"Status": "ON"}'
```

### LDAP Integration

```toml
[ldap]
enabled = true
server_url = "ldaps://dc.example.com:636"
server_type = "active_directory"
bind_dn = "CN=hafiz-svc,OU=Service Accounts,DC=example,DC=com"
bind_password = "service-password"
user_base_dn = "OU=Users,DC=example,DC=com"
user_filter = "(sAMAccountName={username})"
group_base_dn = "OU=Groups,DC=example,DC=com"

[ldap.group_policies]
"Domain Admins" = ["admin"]
"S3 Admins" = ["admin"]
"S3 Users" = ["readwrite"]
"S3 Readers" = ["readonly"]
```

### Event Notifications

```bash
# Configure webhook notifications
aws --endpoint-url http://localhost:9000 s3api put-bucket-notification-configuration \
  --bucket mybucket \
  --notification-configuration '{
    "QueueConfigurations": [{
      "QueueArn": "arn:aws:sqs:us-east-1:000000000000:my-queue",
      "Events": ["s3:ObjectCreated:*", "s3:ObjectRemoved:*"]
    }]
  }'
```

## Monitoring

### Prometheus Metrics

Metrics available at `http://localhost:9000/metrics`:

```
# Request metrics
hafiz_http_requests_total{method="GET", path="/", status="200"}
hafiz_http_request_duration_seconds{method="GET", quantile="0.99"}

# Storage metrics
hafiz_storage_bytes_total
hafiz_objects_total
hafiz_buckets_total

# Operation metrics
hafiz_put_object_duration_seconds
hafiz_get_object_duration_seconds
```

### Grafana Dashboard

Import the dashboard from `deploy/grafana/dashboards/hafiz.json`.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Hafiz Server                           │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   S3 API    │  │  Admin API  │  │   Cluster Comms     │  │
│  │  (Port 9000)│  │ (Port 9001) │  │    (Port 9100)      │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
│         │                │                     │             │
│  ┌──────┴────────────────┴─────────────────────┴──────────┐ │
│  │                  Request Router                         │ │
│  └──────────────────────────┬──────────────────────────────┘ │
│                             │                                │
│  ┌──────────────────────────┴──────────────────────────────┐ │
│  │                  Authentication                          │ │
│  │        (AWS SigV4, IAM Policies, LDAP)                  │ │
│  └──────────────────────────┬──────────────────────────────┘ │
│                             │                                │
│  ┌────────────┐  ┌──────────┴──────────┐  ┌──────────────┐  │
│  │ Metadata   │  │   Business Logic    │  │   Events     │  │
│  │ (SQLite/   │  │   (Versioning,      │  │ (Webhooks,   │  │
│  │  Postgres) │  │    Lifecycle, etc)  │  │  Kafka, NATS)│  │
│  └──────┬─────┘  └──────────┬──────────┘  └──────────────┘  │
│         │                   │                                │
│  ┌──────┴───────────────────┴───────────────────────────────┐│
│  │              Storage Backend (Encrypted)                  ││
│  │                    (File System)                          ││
│  └───────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

## Project Structure

```
hafiz/
├── crates/
│   ├── hafiz-core/        # Core types, config, errors
│   ├── hafiz-storage/     # Storage backend
│   ├── hafiz-metadata/    # Metadata repository
│   ├── hafiz-crypto/      # Encryption
│   ├── hafiz-auth/        # Authentication, IAM
│   ├── hafiz-s3-api/      # S3 API implementation
│   ├── hafiz-cluster/     # Clustering
│   ├── hafiz-admin/       # Admin UI (WebAssembly)
│   └── hafiz-cli/         # CLI tool
├── deploy/
│   ├── docker/            # Docker files
│   ├── helm/              # Helm charts
│   ├── prometheus/        # Prometheus config
│   └── grafana/           # Grafana dashboards
├── docs/                  # Documentation
├── Cargo.toml             # Workspace manifest
├── docker-compose.yml     # Docker Compose
└── README.md
```

## API Reference

See [API Documentation](docs/API.md) for complete S3 API reference.

## Performance

Benchmarks on AMD EPYC 7763, 64GB RAM, NVMe SSD:

| Operation | Throughput | Latency (p99) |
|-----------|------------|---------------|
| PUT (1MB) | 2,500 ops/s | 15ms |
| GET (1MB) | 4,000 ops/s | 8ms |
| LIST (1000 objects) | 500 ops/s | 50ms |
| DELETE | 10,000 ops/s | 2ms |

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md).

```bash
# Development setup
git clone https://github.com/hafiz/hafiz.git
cd hafiz
cargo build
cargo test
```

## License

Apache License 2.0. See [LICENSE](LICENSE) for details.

## Acknowledgments

- Inspired by MinIO, SeaweedFS, and other great object storage projects
- Built with amazing Rust ecosystem: tokio, axum, sqlx
- Thanks to all contributors!

---

<div align="center">
Made with ❤️ by the Hafiz Team
</div>
