# Hafiz Roadmap

This document outlines the planned features and improvements for Hafiz.

## Current Status: v0.1.0 (Alpha)

Hafiz is currently in alpha stage. Core S3 functionality is complete and ready for testing.

---

## v0.1.x (Current - Bug Fixes & Stability)

### Goals
- Bug fixes and stability improvements
- Documentation enhancements
- Community feedback incorporation

### Planned
- [ ] Integration test suite
- [ ] Performance benchmarks
- [ ] CI/CD pipeline (GitHub Actions)
- [ ] Docker image optimization
- [ ] ARM64 support

---

## v0.2.0 (Q1 2025 - Enhanced Features)

### S3 Select
Query objects using SQL without downloading.

```sql
SELECT s.name, s.age FROM S3Object s WHERE s.age > 30
```

### Cross-Region Replication
Replicate objects to another Hafiz cluster.

```yaml
replication:
  rules:
    - destination: s3://backup-bucket
      prefix: important/
```

### Web UI Improvements
- Object browser with preview
- Drag-and-drop upload
- Bucket policy editor
- Real-time metrics dashboard

### Additional Features
- [ ] Inventory reports
- [ ] Batch operations
- [ ] Object Lambda (transform on read)
- [ ] Improved CORS handling

---

## v0.3.0 (Q2 2025 - Enterprise Features)

### Erasure Coding
Data redundancy without full replication.

```
Data blocks: [D1] [D2] [D3] [D4]
Parity:      [P1] [P2]
Tolerance:   2 disk failures
```

### Tiered Storage
Automatic data movement based on access patterns.

```yaml
tiers:
  - name: hot
    storage: ssd
    after: 0 days
  - name: warm
    storage: hdd
    after: 30 days
  - name: cold
    storage: glacier
    after: 90 days
```

### Terraform Provider

```hcl
resource "hafiz_bucket" "example" {
  name = "my-bucket"
  
  versioning {
    enabled = true
  }
  
  lifecycle_rule {
    prefix = "logs/"
    expiration_days = 30
  }
}
```

### Additional Features
- [ ] Quotas and rate limiting
- [ ] Multi-tenancy
- [ ] Audit log export (CloudTrail format)
- [ ] SSO integration (SAML, OIDC)

---

## v1.0.0 (Q3 2025 - Production Ready)

### Goals
- Production stability
- Comprehensive documentation
- Enterprise support readiness

### Requirements
- [ ] 99.99% API compatibility
- [ ] Performance benchmarks published
- [ ] Security audit completed
- [ ] Disaster recovery tested
- [ ] Migration tools available

### Long-Term Support
- 2-year LTS commitment
- Security patches
- Critical bug fixes

---

## Future Considerations (v2.0+)

### Potential Features
- **S3 Express One Zone** - Ultra-low latency storage class
- **Intelligent Tiering** - ML-based data placement
- **Global Namespace** - Unified view across regions
- **Object Federation** - Query across buckets
- **Event Bridge** - Advanced event routing
- **Data Catalog** - Automatic schema discovery

### Performance Goals
- 10,000+ requests/second per node
- Sub-millisecond metadata operations
- 10 Gbps+ throughput per node

### Scalability Goals
- 1000+ node clusters
- Exabyte-scale storage
- Billions of objects

---

## How to Contribute

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md).

### Priority Areas
1. **Testing** - Integration tests, load tests
2. **Documentation** - Examples, tutorials
3. **Performance** - Profiling, optimization
4. **Compatibility** - S3 API edge cases

### Feature Requests

Open a [Discussion](https://github.com/shellnoq/hafiz/discussions) to propose new features.

---

## Release Schedule

| Version | Target Date | Status |
|---------|-------------|--------|
| v0.1.0 | Dec 2024 | ✅ Released |
| v0.1.x | Jan 2025 | 🔄 In Progress |
| v0.2.0 | Mar 2025 | 📋 Planned |
| v0.3.0 | Jun 2025 | 📋 Planned |
| v1.0.0 | Sep 2025 | 📋 Planned |

---

*This roadmap is subject to change based on community feedback and priorities.*
