---
title: Quick Start
description: Get Hafiz running in 5 minutes
---

# Quick Start

Get Hafiz running in under 5 minutes.

## Prerequisites

- Docker (recommended) or
- Rust 1.75+ (for building from source)

## Docker

The fastest way to get started:

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

That's it! Hafiz is now running:

- **S3 API**: http://localhost:9000
- **Admin UI**: http://localhost:9001

## Verify Installation

### Using AWS CLI

```bash
# Configure
aws configure set aws_access_key_id minioadmin
aws configure set aws_secret_access_key minioadmin

# Create bucket
aws --endpoint-url http://localhost:9000 s3 mb s3://my-bucket

# Upload file
echo "Hello, Hafiz!" > test.txt
aws --endpoint-url http://localhost:9000 s3 cp test.txt s3://my-bucket/

# List objects
aws --endpoint-url http://localhost:9000 s3 ls s3://my-bucket/
```

### Using Python

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

# Upload
s3.put_object(Bucket='my-bucket', Key='hello.txt', Body=b'Hello!')

# Download
response = s3.get_object(Bucket='my-bucket', Key='hello.txt')
print(response['Body'].read())
```

### Using Hafiz CLI

```bash
# Install
cargo install hafiz-cli

# Configure
hafiz configure
# Endpoint: http://localhost:9000
# Access Key: minioadmin
# Secret Key: minioadmin

# Use
hafiz ls s3://
hafiz mb s3://my-bucket
hafiz cp file.txt s3://my-bucket/
```

## Docker Compose

For a more complete setup with PostgreSQL:

```yaml title="docker-compose.yml"
version: '3.8'
services:
  hafiz:
    image: ghcr.io/shellnoq/hafiz:latest
    ports:
      - "9000:9000"
      - "9001:9001"
    environment:
      - HAFIZ_ROOT_ACCESS_KEY=minioadmin
      - HAFIZ_ROOT_SECRET_KEY=minioadmin
      - HAFIZ_DATABASE_URL=postgres://hafiz:hafiz@postgres/hafiz
    volumes:
      - hafiz-data:/data
    depends_on:
      - postgres

  postgres:
    image: postgres:15
    environment:
      - POSTGRES_USER=hafiz
      - POSTGRES_PASSWORD=hafiz
      - POSTGRES_DB=hafiz
    volumes:
      - postgres-data:/var/lib/postgresql/data

volumes:
  hafiz-data:
  postgres-data:
```

```bash
docker-compose up -d
```

## Next Steps

- [Configuration](configuration.md) - Customize your deployment
- [User Guide](../user-guide/index.md) - Learn about buckets and objects
- [Encryption](../user-guide/encryption.md) - Enable server-side encryption
- [Access Control](../user-guide/access-control.md) - Configure permissions
