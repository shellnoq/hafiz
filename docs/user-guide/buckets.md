---
title: Buckets
description: Creating and managing buckets in Hafiz
---

# Buckets

Buckets are containers for storing objects. Every object in Hafiz must belong to a bucket.

## Creating a Bucket

=== "AWS CLI"

    ```bash
    aws --endpoint-url http://localhost:9000 s3 mb s3://my-bucket
    ```

=== "Hafiz CLI"

    ```bash
    hafiz mb s3://my-bucket
    ```

=== "Python"

    ```python
    s3.create_bucket(Bucket='my-bucket')
    ```

## Listing Buckets

```bash
# AWS CLI
aws --endpoint-url http://localhost:9000 s3 ls

# Hafiz CLI
hafiz ls s3://
```

## Deleting a Bucket

!!! warning "Bucket must be empty"
    You must delete all objects before deleting a bucket.

```bash
# Delete empty bucket
aws --endpoint-url http://localhost:9000 s3 rb s3://my-bucket

# Force delete (delete all objects first)
hafiz rb s3://my-bucket --force
```

## Bucket Naming Rules

- 3-63 characters long
- Lowercase letters, numbers, and hyphens only
- Must start with a letter or number
- Cannot be formatted as an IP address

!!! success "Valid names"
    `my-bucket`, `data-2024`, `backup-logs`

!!! failure "Invalid names"
    `My-Bucket`, `my_bucket`, `192.168.1.1`

## Bucket Properties

### Versioning

Enable versioning to keep multiple versions of objects:

```bash
aws --endpoint-url http://localhost:9000 s3api put-bucket-versioning \
    --bucket my-bucket \
    --versioning-configuration Status=Enabled
```

### Lifecycle Rules

Automatically expire or transition objects:

```bash
aws --endpoint-url http://localhost:9000 s3api put-bucket-lifecycle-configuration \
    --bucket my-bucket \
    --lifecycle-configuration file://lifecycle.json
```

### Bucket Policy

Control access with IAM-style policies:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": "*",
      "Action": "s3:GetObject",
      "Resource": "arn:aws:s3:::my-bucket/*"
    }
  ]
}
```

## Bucket Information

```bash
# Get bucket location
aws --endpoint-url http://localhost:9000 s3api get-bucket-location \
    --bucket my-bucket

# Get versioning status
aws --endpoint-url http://localhost:9000 s3api get-bucket-versioning \
    --bucket my-bucket

# Hafiz CLI - detailed info
hafiz info s3://my-bucket
```
