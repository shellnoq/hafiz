# Hafiz Architecture

This document describes the internal architecture of Hafiz, including its components, data flow, and design decisions.

## Overview

Hafiz is built as a modular Rust application with clear separation of concerns. Each major functionality is implemented as a separate crate, allowing for independent development, testing, and potential reuse.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              Clients                                    │
│         (AWS CLI, SDKs, Web Browser, Hafiz CLI, Applications)           │
└─────────────────────────────────┬───────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          Load Balancer                                  │
│                    (nginx, HAProxy, AWS ALB)                           │
└─────────────────────────────────┬───────────────────────────────────────┘
                                  │
              ┌───────────────────┼───────────────────┐
              │                   │                   │
              ▼                   ▼                   ▼
┌─────────────────────┐ ┌─────────────────────┐ ┌─────────────────────┐
│     Hafiz Node 1    │ │     Hafiz Node 2    │ │     Hafiz Node 3    │
│  ┌───────────────┐  │ │  ┌───────────────┐  │ │  ┌───────────────┐  │
│  │   S3 API      │  │ │  │   S3 API      │  │ │  │   S3 API      │  │
│  │   (Axum)      │  │ │  │   (Axum)      │  │ │  │   (Axum)      │  │
│  ├───────────────┤  │ │  ├───────────────┤  │ │  ├───────────────┤  │
│  │   Admin API   │  │ │  │   Admin API   │  │ │  │   Admin API   │  │
│  ├───────────────┤  │ │  ├───────────────┤  │ │  ├───────────────┤  │
│  │   Auth Layer  │  │ │  │   Auth Layer  │  │ │  │   Auth Layer  │  │
│  ├───────────────┤  │ │  ├───────────────┤  │ │  ├───────────────┤  │
│  │   Crypto      │  │ │  │   Crypto      │  │ │  │   Crypto      │  │
│  ├───────────────┤  │ │  ├───────────────┤  │ │  ├───────────────┤  │
│  │   Storage     │  │ │  │   Storage     │  │ │  │   Storage     │  │
│  └───────┬───────┘  │ │  └───────┬───────┘  │ │  └───────┬───────┘  │
└──────────┼──────────┘ └──────────┼──────────┘ └──────────┼──────────┘
           │                       │                       │
           └───────────────────────┼───────────────────────┘
                                   │
                          ┌────────▼────────┐
                          │   PostgreSQL    │
                          │   (Metadata)    │
                          └─────────────────┘
```

## Crate Structure

```
hafiz/
├── crates/
│   ├── hafiz-core/       # Core types, traits, utilities (2,500 lines)
│   ├── hafiz-s3-api/     # S3 API implementation (8,000 lines)
│   ├── hafiz-storage/    # Storage backends (1,800 lines)
│   ├── hafiz-metadata/   # Metadata repository (3,200 lines)
│   ├── hafiz-auth/       # Authentication (2,100 lines)
│   ├── hafiz-crypto/     # Encryption (800 lines)
│   ├── hafiz-cluster/    # Cluster coordination (1,500 lines)
│   ├── hafiz-admin/      # Admin API & UI (2,800 lines)
│   └── hafiz-cli/        # Command-line tool (3,800 lines)
```

## Component Details

### hafiz-core

Foundation crate with shared types:

- `Bucket`, `Object`, `ObjectMetadata` structs
- `HafizError` enum with S3 error codes
- `Config` for application configuration
- Common utilities and traits

### hafiz-s3-api

HTTP layer using Axum framework:

- 76+ S3 API endpoints
- XML request/response handling
- Middleware: auth, logging, metrics
- Streaming uploads/downloads

### hafiz-storage

Storage abstraction:

```rust
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn put_object(&self, bucket: &str, key: &str, data: Bytes) -> Result<()>;
    async fn get_object(&self, bucket: &str, key: &str) -> Result<Bytes>;
    async fn delete_object(&self, bucket: &str, key: &str) -> Result<()>;
}
```

Implementations:
- `FilesystemStorage` - Local disk storage
- `S3ProxyStorage` - Proxy to another S3

### hafiz-metadata

Database layer using SQLx:

- SQLite (development)
- PostgreSQL (production)
- 20+ tables for buckets, objects, policies, etc.

### hafiz-auth

Authentication & authorization:

- AWS Signature V4 verification
- AWS Signature V2 (legacy)
- LDAP/Active Directory integration
- Bucket policy evaluation

### hafiz-crypto

Cryptographic operations:

- AES-256-GCM encryption
- Key derivation (HKDF)
- MD5, SHA-256 hashing

### hafiz-cluster

Distributed coordination:

- Gossip protocol for discovery
- Leader election
- Metadata synchronization

## Data Flow

### PUT Object

```
Client → Auth → Policy Check → Encrypt → Store → Metadata → Events → Response
```

### GET Object

```
Client → Auth → Policy Check → Metadata → Retrieve → Decrypt → Stream Response
```

## Security Model

1. **Network**: TLS encryption in transit
2. **Authentication**: AWS SigV4, LDAP
3. **Authorization**: Bucket policies, IAM
4. **Data**: AES-256-GCM at rest
5. **Audit**: Access logging

## Performance

- Async I/O with Tokio
- Connection pooling
- Streaming (no full buffering)
- Prepared SQL statements

## Scalability

- Stateless API nodes
- Shared PostgreSQL metadata
- Horizontal scaling via load balancer
- Gossip-based cluster discovery
