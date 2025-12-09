---
title: Deployment
description: Deploying Hafiz in various environments
---

# Deployment

Choose the deployment method that fits your needs.

<div class="grid cards" markdown>

-   :material-docker:{ .lg .middle } __Docker__

    ---

    Quickest way to get started.

    [:octicons-arrow-right-24: Docker Guide](docker.md)

-   :material-kubernetes:{ .lg .middle } __Kubernetes__

    ---

    Production-ready with Helm.

    [:octicons-arrow-right-24: Kubernetes Guide](kubernetes.md)

-   :material-rocket-launch:{ .lg .middle } __Production__

    ---

    Best practices for production.

    [:octicons-arrow-right-24: Production Guide](production.md)

</div>

## Quick Comparison

| Method | Use Case | Complexity |
|--------|----------|------------|
| Docker | Development | ⭐ Easy |
| Docker Compose | Testing | ⭐⭐ Medium |
| Kubernetes | Production | ⭐⭐⭐ Advanced |
| Binary | Bare metal | ⭐⭐ Medium |

## Minimum Requirements

| Resource | Development | Production |
|----------|-------------|------------|
| CPU | 1 core | 4+ cores |
| Memory | 512 MB | 4+ GB |
| Disk | 1 GB | 100+ GB |
| Network | - | 1 Gbps+ |

## Quick Start

```bash
docker run -d \
  --name hafiz \
  -p 9000:9000 \
  -p 9001:9001 \
  -e HAFIZ_ROOT_ACCESS_KEY=hafizadmin \
  -e HAFIZ_ROOT_SECRET_KEY=hafizadmin \
  ghcr.io/shellnoq/hafiz:latest
```

Access:

- **S3 API:** http://localhost:9000
- **Admin UI:** http://localhost:9001
