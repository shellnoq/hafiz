# Changelog

All notable changes to Hafiz will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release preparation

## [0.1.0] - 2024-12-08

### Added

#### Core Features
- Full S3 API compatibility (76+ endpoints)
- Bucket operations: create, delete, list, head
- Object operations: put, get, delete, copy, head
- Multi-part uploads with parallel processing
- Object versioning with MFA delete protection
- Lifecycle policies with expiration rules
- Storage class support (STANDARD, REDUCED_REDUNDANCY, etc.)

#### Security
- AWS Signature V4 authentication
- AWS Signature V2 (legacy) support
- Server-side encryption (AES-256-GCM)
- Bucket policies with IAM-style syntax
- LDAP/Active Directory integration
- TLS encryption in transit
- Object Lock (WORM) for compliance
  - Governance mode
  - Compliance mode (SEC 17a-4)
  - Legal holds

#### Operations
- Admin web UI
- Admin REST API
- Prometheus metrics
- Health check endpoints
- Event notifications (Webhook, Kafka, NATS, Redis, AMQP)
- Access logging

#### Infrastructure
- Cluster mode with gossip protocol
- PostgreSQL metadata backend
- SQLite for development
- Docker support
- Docker Compose configurations
- Kubernetes Helm chart
- Horizontal Pod Autoscaler
- Pod Disruption Budget

#### CLI
- `hafiz ls` - List buckets and objects
- `hafiz cp` - Copy files
- `hafiz mv` - Move files
- `hafiz sync` - Synchronize directories
- `hafiz rm` - Remove objects
- `hafiz mb` - Make bucket
- `hafiz rb` - Remove bucket
- `hafiz head` - Get metadata
- `hafiz cat` - Stream content
- `hafiz du` - Disk usage
- `hafiz presign` - Generate presigned URLs
- `hafiz configure` - Manage configuration

#### API Endpoints
- Bucket: create, delete, list, head, location
- Object: put, get, delete, copy, head
- Multipart: create, upload part, complete, abort, list
- Versioning: get, put, list versions
- Lifecycle: get, put, delete
- Policy: get, put, delete
- ACL: get, put
- CORS: get, put, delete
- Tagging: get, put, delete
- Object Lock: configuration, retention, legal hold
- Encryption: get, put

### Technical Details

| Metric | Value |
|--------|-------|
| Rust Lines of Code | 33,000+ |
| Crates | 9 |
| API Endpoints | 76+ |
| Database Tables | 20+ |
| Helm Chart Templates | 15 |

### Dependencies
- Rust 1.75+
- Tokio async runtime
- Axum web framework
- SQLx database layer
- AWS SDK for Rust (S3 client)

---

## Future Releases

### [0.2.0] - Planned
- S3 Select (SQL queries)
- Cross-region replication
- Web UI improvements
- Performance optimizations

### [0.3.0] - Planned
- Erasure coding
- Tiered storage
- Terraform provider

### [1.0.0] - Planned
- Production-ready release
- Long-term support
- Enterprise features

---

[Unreleased]: https://github.com/shellnoq/hafiz/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/shellnoq/hafiz/releases/tag/v0.1.0
