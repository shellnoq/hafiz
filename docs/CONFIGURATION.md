# Hafiz Configuration Reference

Complete reference for all Hafiz configuration options.

## Configuration Methods

1. **Environment Variables** (recommended for production)
2. **Configuration File** (`config.toml`)
3. **Command-line Arguments**

Priority: CLI args > Environment > Config file > Defaults

## Environment Variables

### Required

```bash
# Root credentials (required)
HAFIZ_ROOT_ACCESS_KEY=your-access-key-id
HAFIZ_ROOT_SECRET_KEY=your-secret-access-key
```

### Server Settings

```bash
# S3 API
HAFIZ_S3_PORT=9000                    # S3 API port
HAFIZ_S3_HOST=0.0.0.0                 # Bind address
HAFIZ_REGION=us-east-1                # Default region

# Admin API
HAFIZ_ADMIN_ENABLED=true              # Enable admin API
HAFIZ_ADMIN_PORT=9001                 # Admin API port
HAFIZ_ADMIN_USERNAME=admin            # Admin username
HAFIZ_ADMIN_PASSWORD=changeme         # Admin password

# Logging
HAFIZ_LOG_LEVEL=info                  # trace, debug, info, warn, error
HAFIZ_LOG_FORMAT=json                 # json, pretty
```

### Storage

```bash
# Filesystem storage
HAFIZ_STORAGE_TYPE=filesystem         # filesystem, s3
HAFIZ_STORAGE_BASE_PATH=/data         # Data directory
HAFIZ_COMPRESSION_ENABLED=true        # Enable compression
HAFIZ_COMPRESSION_LEVEL=6             # 1-9 (higher = smaller)
```

### Metadata Database

```bash
# SQLite (development)
HAFIZ_METADATA_TYPE=sqlite
HAFIZ_SQLITE_PATH=/data/metadata.db

# PostgreSQL (production)
HAFIZ_METADATA_TYPE=postgresql
HAFIZ_DATABASE_URL=postgres://user:pass@host:5432/hafiz
# Or individual settings:
HAFIZ_DATABASE_HOST=localhost
HAFIZ_DATABASE_PORT=5432
HAFIZ_DATABASE_NAME=hafiz
HAFIZ_DATABASE_USER=hafiz
HAFIZ_DATABASE_PASSWORD=secret
HAFIZ_DATABASE_POOL_SIZE=10
```

### Encryption

```bash
HAFIZ_ENCRYPTION_ENABLED=true
HAFIZ_ENCRYPTION_MASTER_KEY=base64-encoded-32-byte-key
# Generate: openssl rand -base64 32
```

### Cluster Mode

```bash
HAFIZ_CLUSTER_ENABLED=true
HAFIZ_CLUSTER_NODE_ID=node-1          # Unique node identifier
HAFIZ_CLUSTER_GOSSIP_PORT=7946        # Gossip protocol port
HAFIZ_CLUSTER_SYNC_PORT=7947          # Data sync port
HAFIZ_CLUSTER_PEERS=node2:7946,node3:7946
```

### LDAP Authentication

```bash
HAFIZ_LDAP_ENABLED=true
HAFIZ_LDAP_URL=ldap://ldap.example.com:389
HAFIZ_LDAP_BASE_DN=dc=example,dc=com
HAFIZ_LDAP_BIND_DN=cn=admin,dc=example,dc=com
HAFIZ_LDAP_BIND_PASSWORD=secret
HAFIZ_LDAP_USER_FILTER=(uid={username})
HAFIZ_LDAP_GROUP_FILTER=(member={dn})
HAFIZ_LDAP_TLS_ENABLED=true
HAFIZ_LDAP_TLS_SKIP_VERIFY=false
```

### TLS

```bash
HAFIZ_TLS_ENABLED=true
HAFIZ_TLS_CERT_PATH=/etc/hafiz/tls.crt
HAFIZ_TLS_KEY_PATH=/etc/hafiz/tls.key
```

### Event Notifications

```bash
HAFIZ_EVENTS_ENABLED=true

# Webhook
HAFIZ_EVENTS_WEBHOOK_URL=https://example.com/webhook
HAFIZ_EVENTS_WEBHOOK_SECRET=signing-secret

# Kafka
HAFIZ_EVENTS_KAFKA_BROKERS=kafka1:9092,kafka2:9092
HAFIZ_EVENTS_KAFKA_TOPIC=hafiz-events

# NATS
HAFIZ_EVENTS_NATS_URL=nats://localhost:4222
HAFIZ_EVENTS_NATS_SUBJECT=hafiz.events

# Redis
HAFIZ_EVENTS_REDIS_URL=redis://localhost:6379
HAFIZ_EVENTS_REDIS_CHANNEL=hafiz-events

# AMQP
HAFIZ_EVENTS_AMQP_URL=amqp://guest:guest@localhost:5672
HAFIZ_EVENTS_AMQP_EXCHANGE=hafiz-events
```

### Metrics

```bash
HAFIZ_METRICS_ENABLED=true
HAFIZ_METRICS_PORT=9090
HAFIZ_METRICS_PATH=/metrics
```

### Request Limits

```bash
HAFIZ_MAX_BODY_SIZE=5368709120        # 5GB max upload
HAFIZ_REQUEST_TIMEOUT=300             # Seconds
HAFIZ_MULTIPART_MAX_PARTS=10000       # Max parts per upload
HAFIZ_MULTIPART_MIN_PART_SIZE=5242880 # 5MB minimum part
```

## Configuration File

Location: `~/.hafiz/config.toml` or `/etc/hafiz/config.toml`

```toml
[server]
s3_port = 9000
admin_port = 9001
region = "us-east-1"
log_level = "info"

[storage]
type = "filesystem"
base_path = "/data"
compression = true

[metadata]
type = "postgresql"
database_url = "postgres://hafiz:secret@localhost/hafiz"

[encryption]
enabled = true
# master_key from environment variable

[cluster]
enabled = true
node_id = "node-1"
gossip_port = 7946
peers = ["node2:7946", "node3:7946"]

[ldap]
enabled = false
url = "ldap://ldap.example.com"
base_dn = "dc=example,dc=com"

[tls]
enabled = true
cert_path = "/etc/hafiz/tls.crt"
key_path = "/etc/hafiz/tls.key"

[metrics]
enabled = true
port = 9090
```

## CLI Configuration

Hafiz CLI uses `~/.hafiz/config.toml`:

```toml
[default]
endpoint = "http://localhost:9000"
access_key = "minioadmin"
secret_key = "minioadmin"
region = "us-east-1"

[production]
endpoint = "https://s3.example.com"
access_key = "prod-key"
secret_key = "prod-secret"
```

Use profile: `hafiz --profile production ls s3://`

## Docker Environment

```yaml
# docker-compose.yml
services:
  hafiz:
    image: ghcr.io/shellnoq/hafiz:latest
    environment:
      - HAFIZ_ROOT_ACCESS_KEY=minioadmin
      - HAFIZ_ROOT_SECRET_KEY=minioadmin
      - HAFIZ_DATABASE_URL=postgres://hafiz:secret@postgres/hafiz
      - HAFIZ_ENCRYPTION_ENABLED=true
      - HAFIZ_ENCRYPTION_MASTER_KEY=${ENCRYPTION_KEY}
```

## Kubernetes ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: hafiz-config
data:
  log-level: "info"
  region: "us-east-1"
  storage-type: "filesystem"
  metadata-type: "postgresql"
```

## Validation

Test configuration:

```bash
# Check config file
hafiz-server --config /path/to/config.toml --validate

# Check environment
hafiz-server --check-config
```

## Defaults

| Setting | Default | Notes |
|---------|---------|-------|
| S3 Port | 9000 | |
| Admin Port | 9001 | |
| Region | us-east-1 | |
| Log Level | info | |
| Storage Type | filesystem | |
| Metadata Type | sqlite | |
| Compression | true | |
| Encryption | false | Enable for production |
| Cluster | false | |
| Metrics | true | |
