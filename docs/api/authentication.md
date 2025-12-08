---
title: Authentication
description: S3 API authentication methods
---

# Authentication

Hafiz uses AWS Signature Version 4 for request authentication.

## AWS Signature V4

All requests must include a valid signature in the `Authorization` header.

### Header Format

```
Authorization: AWS4-HMAC-SHA256 
    Credential=ACCESS_KEY/DATE/REGION/s3/aws4_request,
    SignedHeaders=host;x-amz-date,
    Signature=CALCULATED_SIGNATURE
```

### Required Headers

| Header | Description | Example |
|--------|-------------|---------|
| `Authorization` | Signature | `AWS4-HMAC-SHA256 ...` |
| `x-amz-date` | Timestamp | `20240101T000000Z` |
| `x-amz-content-sha256` | Payload hash | `UNSIGNED-PAYLOAD` |
| `Host` | Server hostname | `s3.example.com` |

## Using AWS SDKs

SDKs handle signing automatically:

=== "Python"

    ```python
    import boto3

    s3 = boto3.client('s3',
        endpoint_url='http://localhost:9000',
        aws_access_key_id='YOUR_ACCESS_KEY',
        aws_secret_access_key='YOUR_SECRET_KEY',
        region_name='us-east-1'
    )
    ```

=== "JavaScript"

    ```javascript
    import { S3Client } from "@aws-sdk/client-s3";

    const client = new S3Client({
      endpoint: "http://localhost:9000",
      region: "us-east-1",
      credentials: {
        accessKeyId: "YOUR_ACCESS_KEY",
        secretAccessKey: "YOUR_SECRET_KEY",
      },
      forcePathStyle: true,
    });
    ```

=== "AWS CLI"

    ```bash
    aws configure set aws_access_key_id YOUR_ACCESS_KEY
    aws configure set aws_secret_access_key YOUR_SECRET_KEY
    aws configure set region us-east-1
    ```

## Presigned URLs

Generate time-limited URLs that don't require credentials:

```bash
# Generate presigned URL (valid 1 hour)
aws --endpoint-url http://localhost:9000 s3 presign \
    s3://my-bucket/file.txt --expires-in 3600
```

Result:
```
http://localhost:9000/my-bucket/file.txt?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=...
```

## Signature V2 (Legacy)

Hafiz also supports the older Signature V2 for compatibility:

```
Authorization: AWS ACCESS_KEY:SIGNATURE
```

!!! warning "Deprecated"
    Signature V2 is deprecated. Use Signature V4 for new applications.
