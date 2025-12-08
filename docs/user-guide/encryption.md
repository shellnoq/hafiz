---
title: Encryption
description: Server-side encryption in Hafiz
---

# Encryption

Hafiz supports server-side encryption (SSE) to protect your data at rest.

## Encryption Methods

| Method | Description | Key Management |
|--------|-------------|----------------|
| SSE-S3 | Hafiz-managed keys | Automatic |
| SSE-C | Customer-provided keys | You manage |

## Enable Encryption

### Server-Side Encryption (Default)

```bash
# Enable globally
HAFIZ_ENCRYPTION_ENABLED=true
HAFIZ_ENCRYPTION_MASTER_KEY=$(openssl rand -base64 32)
```

### Per-Object Encryption

```bash
aws --endpoint-url http://localhost:9000 s3 cp file.txt s3://my-bucket/ \
    --sse AES256
```

### Customer-Provided Keys (SSE-C)

```bash
# Generate a key
KEY=$(openssl rand -base64 32)
KEY_MD5=$(echo -n "$KEY" | openssl dgst -md5 -binary | base64)

# Upload with SSE-C
aws --endpoint-url http://localhost:9000 s3 cp file.txt s3://my-bucket/ \
    --sse-c AES256 \
    --sse-c-key "$KEY"

# Download (must provide same key)
aws --endpoint-url http://localhost:9000 s3 cp s3://my-bucket/file.txt . \
    --sse-c AES256 \
    --sse-c-key "$KEY"
```

## Encryption Details

### Algorithm

- **AES-256-GCM** - Authenticated encryption
- 256-bit keys
- Per-object unique nonce

### Key Derivation

```
Master Key (from config)
        │
        ▼
    HKDF-SHA256
        │
        ▼
Data Encryption Key (per object)
```

## Bucket Default Encryption

Set default encryption for all new objects:

```bash
aws --endpoint-url http://localhost:9000 s3api put-bucket-encryption \
    --bucket my-bucket \
    --server-side-encryption-configuration '{
      "Rules": [{
        "ApplyServerSideEncryptionByDefault": {
          "SSEAlgorithm": "AES256"
        }
      }]
    }'
```

## Verify Encryption

```bash
# Check object encryption
aws --endpoint-url http://localhost:9000 s3api head-object \
    --bucket my-bucket \
    --key file.txt

# Output includes:
# "ServerSideEncryption": "AES256"
```

## TLS (Encryption in Transit)

Enable TLS for network encryption:

```bash
HAFIZ_TLS_ENABLED=true
HAFIZ_TLS_CERT_PATH=/etc/hafiz/tls.crt
HAFIZ_TLS_KEY_PATH=/etc/hafiz/tls.key
```

## Best Practices

!!! tip "Recommendations"
    1. **Enable encryption globally** - Don't rely on per-object encryption
    2. **Secure master key** - Store in secrets management (Vault, K8s Secrets)
    3. **Use TLS** - Always encrypt data in transit
    4. **Rotate keys** - Plan for key rotation
    5. **Backup keys** - Encrypted data is lost if keys are lost

## Compliance

Hafiz encryption helps meet:

- **HIPAA** - Healthcare data protection
- **PCI-DSS** - Payment card security
- **GDPR** - European data privacy
- **SOC 2** - Security controls
