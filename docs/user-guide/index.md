---
title: User Guide
description: Learn how to use Hafiz for your object storage needs
---

# User Guide

This guide covers everything you need to know about using Hafiz.

## Core Concepts

<div class="grid cards" markdown>

-   :material-bucket:{ .lg .middle } __Buckets__

    ---

    Create, configure, and manage storage containers.

    [:octicons-arrow-right-24: Buckets](buckets.md)

-   :material-file-multiple:{ .lg .middle } __Objects__

    ---

    Upload, download, and manage objects.

    [:octicons-arrow-right-24: Objects](objects.md)

-   :material-history:{ .lg .middle } __Versioning__

    ---

    Keep multiple versions of objects.

    [:octicons-arrow-right-24: Versioning](versioning.md)

-   :material-shield-lock:{ .lg .middle } __Access Control__

    ---

    Control who can access your data.

    [:octicons-arrow-right-24: Access Control](access-control.md)

-   :material-lock:{ .lg .middle } __Encryption__

    ---

    Encrypt data at rest and in transit.

    [:octicons-arrow-right-24: Encryption](encryption.md)

-   :material-lock-clock:{ .lg .middle } __Object Lock__

    ---

    WORM compliance for regulatory requirements.

    [:octicons-arrow-right-24: Object Lock](object-lock.md)

</div>

## Quick Examples

=== "Python"

    ```python
    import boto3

    s3 = boto3.client('s3',
        endpoint_url='http://localhost:9000',
        aws_access_key_id='hafizadmin',
        aws_secret_access_key='hafizadmin'
    )

    # Upload
    s3.put_object(Bucket='my-bucket', Key='file.txt', Body=b'Hello!')
    ```

=== "JavaScript"

    ```javascript
    import { S3Client, PutObjectCommand } from "@aws-sdk/client-s3";

    const client = new S3Client({
      endpoint: "http://localhost:9000",
      forcePathStyle: true,
    });

    await client.send(new PutObjectCommand({
      Bucket: "my-bucket",
      Key: "file.txt",
      Body: "Hello!",
    }));
    ```

=== "AWS CLI"

    ```bash
    aws --endpoint-url http://localhost:9000 \
        s3 cp file.txt s3://my-bucket/
    ```
