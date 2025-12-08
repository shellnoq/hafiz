---
title: Architecture
description: Hafiz system architecture
---

# Architecture

<div class="grid cards" markdown>

-   :material-view-module:{ .lg .middle } __Components__

    ---

    Crate structure and responsibilities.

    [:octicons-arrow-right-24: Components](components.md)

-   :material-shield:{ .lg .middle } __Security__

    ---

    Security model and best practices.

    [:octicons-arrow-right-24: Security](security.md)

</div>

## Overview

```mermaid
graph TB
    Client[Client] --> LB[Load Balancer]
    LB --> H1[Hafiz Node 1]
    LB --> H2[Hafiz Node 2]
    LB --> H3[Hafiz Node 3]
    H1 --> PG[(PostgreSQL)]
    H2 --> PG
    H3 --> PG
    H1 --> D1[(Disk)]
    H2 --> D2[(Disk)]
    H3 --> D3[(Disk)]
```

## Key Design Principles

1. **S3 Compatible** - Works with existing tools
2. **Memory Safe** - Written in Rust
3. **Scalable** - Horizontal scaling
4. **Secure** - Encryption at rest and in transit
5. **Observable** - Prometheus metrics
