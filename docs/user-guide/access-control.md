---
title: Access Control
description: Managing access to buckets and objects
---

# Access Control

Hafiz provides multiple layers of access control to secure your data.

## Authentication

### AWS Signature V4

All requests must be signed using AWS Signature Version 4:

```bash
# AWS CLI handles this automatically
aws --endpoint-url http://localhost:9000 s3 ls

# Or set credentials
export AWS_ACCESS_KEY_ID=your-access-key
export AWS_SECRET_ACCESS_KEY=your-secret-key
```

### LDAP Integration

Connect to enterprise directory services:

```bash
# Configure LDAP
HAFIZ_LDAP_ENABLED=true
HAFIZ_LDAP_URL=ldap://ldap.example.com:389
HAFIZ_LDAP_BASE_DN=dc=example,dc=com
```

## Bucket Policies

IAM-style policies for fine-grained access control.

### Policy Structure

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "AllowPublicRead",
      "Effect": "Allow",
      "Principal": "*",
      "Action": "s3:GetObject",
      "Resource": "arn:aws:s3:::my-bucket/*"
    }
  ]
}
```

### Apply Policy

```bash
aws --endpoint-url http://localhost:9000 s3api put-bucket-policy \
    --bucket my-bucket \
    --policy file://policy.json
```

### Common Policy Examples

#### Public Read Access

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": "*",
      "Action": "s3:GetObject",
      "Resource": "arn:aws:s3:::public-bucket/*"
    }
  ]
}
```

#### Restrict by IP

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": "*",
      "Action": "s3:*",
      "Resource": "arn:aws:s3:::my-bucket/*",
      "Condition": {
        "IpAddress": {
          "aws:SourceIp": "192.168.1.0/24"
        }
      }
    }
  ]
}
```

#### Read-Only User

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": {"AWS": "arn:aws:iam::user/readonly"},
      "Action": [
        "s3:GetObject",
        "s3:ListBucket"
      ],
      "Resource": [
        "arn:aws:s3:::my-bucket",
        "arn:aws:s3:::my-bucket/*"
      ]
    }
  ]
}
```

## Supported Actions

| Action | Description |
|--------|-------------|
| `s3:GetObject` | Download objects |
| `s3:PutObject` | Upload objects |
| `s3:DeleteObject` | Delete objects |
| `s3:ListBucket` | List bucket contents |
| `s3:GetBucketPolicy` | Read bucket policy |
| `s3:PutBucketPolicy` | Write bucket policy |
| `s3:*` | All actions |

## Condition Keys

| Key | Description |
|-----|-------------|
| `aws:SourceIp` | Client IP address |
| `aws:CurrentTime` | Current time |
| `aws:SecureTransport` | HTTPS required |
| `s3:prefix` | Object key prefix |
| `s3:max-keys` | Maximum list results |

## User Management

Create and manage users via the Admin API:

```bash
# Create user
curl -X POST http://localhost:9001/api/v1/users \
    -H "Authorization: Bearer $TOKEN" \
    -d '{"username": "alice", "access_key": "...", "secret_key": "..."}'
```

## Best Practices

1. **Principle of least privilege** - Grant only necessary permissions
2. **Use bucket policies** - Prefer policies over public access
3. **Enable TLS** - Always use HTTPS in production
4. **Rotate credentials** - Regular key rotation
5. **Audit access** - Enable access logging
