# Security

This document describes security features and best practices for Hafiz.

## Security Features

### Authentication

#### AWS Signature V4
- Industry-standard request signing
- Time-limited signatures (5-15 minutes)
- Protection against replay attacks

#### LDAP/Active Directory
- Enterprise identity integration
- Group-based access control
- TLS-secured connections

### Authorization

#### Bucket Policies
- IAM-style policy language
- Fine-grained permissions
- Condition-based access (IP, time, MFA)

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": {"AWS": "arn:aws:iam::user/alice"},
      "Action": ["s3:GetObject"],
      "Resource": "arn:aws:s3:::my-bucket/*",
      "Condition": {
        "IpAddress": {"aws:SourceIp": "192.168.1.0/24"}
      }
    }
  ]
}
```

### Encryption

#### In Transit (TLS)
- TLS 1.2/1.3 required
- Strong cipher suites only
- Certificate validation

#### At Rest (AES-256-GCM)
- Per-object encryption keys
- Key derivation from master key
- Secure key storage

### Object Lock (WORM)

#### Governance Mode
- Can be overridden with special permission
- Suitable for general compliance

#### Compliance Mode
- Cannot be overridden by anyone
- Required for SEC 17a-4, FINRA
- Immutable until retention expires

### Audit Logging

- All API requests logged
- User, action, resource, timestamp
- Integration with SIEM systems

## Security Best Practices

### Deployment

```bash
# ✅ Use TLS in production
HAFIZ_TLS_ENABLED=true
HAFIZ_TLS_CERT_PATH=/etc/hafiz/tls.crt
HAFIZ_TLS_KEY_PATH=/etc/hafiz/tls.key

# ✅ Enable encryption
HAFIZ_ENCRYPTION_ENABLED=true
HAFIZ_ENCRYPTION_MASTER_KEY=<secure-key>

# ✅ Use strong credentials
HAFIZ_ROOT_ACCESS_KEY=$(openssl rand -hex 16)
HAFIZ_ROOT_SECRET_KEY=$(openssl rand -hex 32)

# ❌ Don't use default credentials
# HAFIZ_ROOT_ACCESS_KEY=minioadmin  # BAD!
```

### Credentials

1. **Never commit credentials** to version control
2. **Rotate keys** regularly (every 90 days)
3. **Use IAM users** instead of root credentials
4. **Use secrets management** (Vault, K8s Secrets)

### Network

1. **Use TLS** for all connections
2. **Restrict network access** with firewalls
3. **Use private networks** for cluster communication
4. **Enable Network Policies** in Kubernetes

### Access Control

1. **Principle of least privilege**
2. **Use bucket policies** for access control
3. **Enable MFA delete** for critical buckets
4. **Regular access reviews**

## Vulnerability Reporting

### Reporting Process

**DO NOT** open public issues for security vulnerabilities.

1. Email: security@shellnoq.com
2. Include:
   - Description of vulnerability
   - Steps to reproduce
   - Potential impact
   - Any suggested fixes

### Response Timeline

- **24 hours**: Acknowledgment
- **72 hours**: Initial assessment
- **30 days**: Fix development
- **90 days**: Public disclosure

### Bug Bounty

We appreciate responsible disclosure. Significant vulnerabilities may be eligible for recognition.

## Security Checklist

### Before Production

- [ ] TLS enabled
- [ ] Encryption enabled
- [ ] Strong credentials set
- [ ] Default credentials removed
- [ ] Network access restricted
- [ ] Audit logging enabled
- [ ] Backup encryption keys stored securely
- [ ] Security updates applied

### Ongoing

- [ ] Monitor audit logs
- [ ] Rotate credentials quarterly
- [ ] Review access permissions
- [ ] Apply security updates
- [ ] Test backup restoration

## Compliance

| Standard | Feature | Status |
|----------|---------|--------|
| SEC 17a-4 | Object Lock (WORM) | ✅ |
| FINRA 4511 | Immutable records | ✅ |
| HIPAA | Encryption, audit logs | ✅ |
| GDPR | Encryption, access control | ✅ |
| SOC 2 | Audit logging | ✅ |
| PCI-DSS | Encryption | ✅ |

## Dependencies

Hafiz uses audited, well-maintained dependencies:

- `rustls` - TLS implementation
- `aes-gcm` - Encryption
- `sha2`, `hmac` - Cryptographic hashing
- `sqlx` - Database access

Run `cargo audit` to check for known vulnerabilities.

## Security Updates

Subscribe to releases:
- GitHub: Watch → Releases only
- RSS: https://github.com/shellnoq/hafiz/releases.atom

---

For questions, contact: security@shellnoq.com
