---
title: Configuration
description: Configure Hafiz for your environment
---

# Configuration

Hafiz can be configured via environment variables or configuration file.

## Environment Variables

### Required

```bash
HAFIZ_ROOT_ACCESS_KEY=your-access-key
HAFIZ_ROOT_SECRET_KEY=your-secret-key
```

### Server

```bash
HAFIZ_S3_PORT=9000
HAFIZ_ADMIN_PORT=9001
HAFIZ_REGION=us-east-1
HAFIZ_LOG_LEVEL=info  # trace, debug, info, warn, error
```

### Storage

```bash
HAFIZ_STORAGE_TYPE=filesystem
HAFIZ_STORAGE_BASE_PATH=/data
```

### Database

```bash
# PostgreSQL (production)
HAFIZ_DATABASE_URL=postgres://user:pass@host/hafiz

# SQLite (development)
HAFIZ_METADATA_TYPE=sqlite
HAFIZ_SQLITE_PATH=/data/metadata.db
```

### Encryption

```bash
HAFIZ_ENCRYPTION_ENABLED=true
HAFIZ_ENCRYPTION_MASTER_KEY=$(openssl rand -base64 32)
```

### Cluster

```bash
HAFIZ_CLUSTER_ENABLED=true
HAFIZ_CLUSTER_PEERS=node2:7946,node3:7946
```

## Configuration File

Location: `/etc/hafiz/config.toml`

```toml
[server]
s3_port = 9000
admin_port = 9001
region = "us-east-1"

[storage]
type = "filesystem"
base_path = "/data"

[metadata]
type = "postgresql"
database_url = "postgres://hafiz:secret@localhost/hafiz"

[encryption]
enabled = true

[cluster]
enabled = true
peers = ["node2:7946", "node3:7946"]
```

## All Options

| Variable | Default | Description |
|----------|---------|-------------|
| `HAFIZ_ROOT_ACCESS_KEY` | - | Root access key (required) |
| `HAFIZ_ROOT_SECRET_KEY` | - | Root secret key (required) |
| `HAFIZ_S3_PORT` | 9000 | S3 API port |
| `HAFIZ_ADMIN_PORT` | 9001 | Admin API port |
| `HAFIZ_REGION` | us-east-1 | Default region |
| `HAFIZ_LOG_LEVEL` | info | Log level |
| `HAFIZ_STORAGE_BASE_PATH` | /data | Data directory |
| `HAFIZ_DATABASE_URL` | - | PostgreSQL connection |
| `HAFIZ_ENCRYPTION_ENABLED` | false | Enable encryption |
| `HAFIZ_CLUSTER_ENABLED` | false | Enable clustering |
