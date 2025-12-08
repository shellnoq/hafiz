---
title: Object Lock
description: WORM compliance with Object Lock
---

# Object Lock

Object Lock provides Write-Once-Read-Many (WORM) protection for regulatory compliance.

## Overview

Object Lock prevents objects from being deleted or overwritten for a specified period or indefinitely.

!!! success "Compliance"
    Object Lock helps meet SEC 17a-4, FINRA 4511, HIPAA, and GDPR requirements.

## Retention Modes

### Governance Mode

- Can be overridden by users with special permissions
- Suitable for general data protection

### Compliance Mode

- **Cannot be overridden by anyone**, including root user
- Cannot be shortened, only extended
- Required for financial regulations (SEC 17a-4)

!!! danger "Compliance Mode is Permanent"
    Once set, Compliance mode retention cannot be reduced. Choose carefully!

## Enable Object Lock

Object Lock must be enabled when creating the bucket:

```bash
aws --endpoint-url http://localhost:9000 s3api create-bucket \
    --bucket compliance-bucket \
    --object-lock-enabled-for-bucket
```

## Set Default Retention

```bash
aws --endpoint-url http://localhost:9000 s3api put-object-lock-configuration \
    --bucket compliance-bucket \
    --object-lock-configuration '{
      "ObjectLockEnabled": "Enabled",
      "Rule": {
        "DefaultRetention": {
          "Mode": "COMPLIANCE",
          "Days": 365
        }
      }
    }'
```

## Per-Object Retention

```bash
# Set retention when uploading
aws --endpoint-url http://localhost:9000 s3api put-object \
    --bucket compliance-bucket \
    --key document.pdf \
    --body document.pdf \
    --object-lock-mode COMPLIANCE \
    --object-lock-retain-until-date 2025-12-31T00:00:00Z

# Update retention (can only extend)
aws --endpoint-url http://localhost:9000 s3api put-object-retention \
    --bucket compliance-bucket \
    --key document.pdf \
    --retention '{
      "Mode": "COMPLIANCE",
      "RetainUntilDate": "2026-12-31T00:00:00Z"
    }'
```

## Legal Hold

Legal hold prevents deletion regardless of retention settings:

```bash
# Enable legal hold
aws --endpoint-url http://localhost:9000 s3api put-object-legal-hold \
    --bucket compliance-bucket \
    --key document.pdf \
    --legal-hold '{"Status": "ON"}'

# Remove legal hold (requires permission)
aws --endpoint-url http://localhost:9000 s3api put-object-legal-hold \
    --bucket compliance-bucket \
    --key document.pdf \
    --legal-hold '{"Status": "OFF"}'
```

## Check Lock Status

```bash
# Get retention
aws --endpoint-url http://localhost:9000 s3api get-object-retention \
    --bucket compliance-bucket \
    --key document.pdf

# Get legal hold
aws --endpoint-url http://localhost:9000 s3api get-object-legal-hold \
    --bucket compliance-bucket \
    --key document.pdf
```

## Compliance Matrix

| Regulation | Mode | Retention | Notes |
|------------|------|-----------|-------|
| SEC 17a-4 | Compliance | As required | Financial records |
| FINRA 4511 | Compliance | 6+ years | Broker-dealer records |
| HIPAA | Either | 6+ years | Healthcare records |
| GDPR | Governance | Varies | Right to erasure considerations |

## Best Practices

1. **Test with Governance first** - Before using Compliance mode
2. **Document retention policies** - Know requirements before setting
3. **Use Legal Hold for litigation** - Preserve evidence
4. **Enable versioning** - Required for Object Lock
5. **Plan storage costs** - Locked objects consume space
