---
title: Docker Deployment
description: Running Hafiz with Docker
---

# Docker Deployment

## Single Container

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

## Docker Compose

### Basic Setup

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

### With PostgreSQL

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
      - HAFIZ_DATABASE_URL=postgres://hafiz:secret@postgres/hafiz
    depends_on:
      - postgres

  postgres:
    image: postgres:15
    volumes:
      - postgres-data:/var/lib/postgresql/data
    environment:
      - POSTGRES_USER=hafiz
      - POSTGRES_PASSWORD=secret
      - POSTGRES_DB=hafiz

volumes:
  hafiz-data:
  postgres-data:
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `HAFIZ_ROOT_ACCESS_KEY` | Yes | Root access key |
| `HAFIZ_ROOT_SECRET_KEY` | Yes | Root secret key |
| `HAFIZ_S3_PORT` | No | S3 port (9000) |
| `HAFIZ_ADMIN_PORT` | No | Admin port (9001) |
| `HAFIZ_DATABASE_URL` | No | PostgreSQL URL |

## Verify Installation

```bash
# Check health
curl http://localhost:9000/health

# Test S3 API
aws --endpoint-url http://localhost:9000 s3 ls
```
