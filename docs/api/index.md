---
title: API Reference
description: Hafiz S3 API documentation
---

# API Reference

Hafiz implements the Amazon S3 REST API, providing compatibility with existing tools and SDKs.

## Base URL

```
http://localhost:9000    # Development
https://s3.example.com   # Production
```

## Authentication

All requests must be authenticated using AWS Signature Version 4.

[:octicons-arrow-right-24: Authentication Details](authentication.md)

## API Sections

<div class="grid cards" markdown>

-   :material-bucket:{ .lg .middle } __Bucket Operations__

    ---

    Create, delete, list, and configure buckets.

    [:octicons-arrow-right-24: Bucket API](buckets.md)

-   :material-file:{ .lg .middle } __Object Operations__

    ---

    Upload, download, delete, and manage objects.

    [:octicons-arrow-right-24: Object API](objects.md)

-   :material-alert-circle:{ .lg .middle } __Error Codes__

    ---

    S3-compatible error responses.

    [:octicons-arrow-right-24: Error Codes](errors.md)

</div>

## Quick Reference

### Bucket Operations

| Operation | Method | Path |
|-----------|--------|------|
| List Buckets | `GET` | `/` |
| Create Bucket | `PUT` | `/{bucket}` |
| Delete Bucket | `DELETE` | `/{bucket}` |
| Head Bucket | `HEAD` | `/{bucket}` |
| Get Location | `GET` | `/{bucket}?location` |
| Get Versioning | `GET` | `/{bucket}?versioning` |
| Put Versioning | `PUT` | `/{bucket}?versioning` |
| Get Policy | `GET` | `/{bucket}?policy` |
| Put Policy | `PUT` | `/{bucket}?policy` |

### Object Operations

| Operation | Method | Path |
|-----------|--------|------|
| List Objects | `GET` | `/{bucket}?list-type=2` |
| Get Object | `GET` | `/{bucket}/{key}` |
| Put Object | `PUT` | `/{bucket}/{key}` |
| Delete Object | `DELETE` | `/{bucket}/{key}` |
| Head Object | `HEAD` | `/{bucket}/{key}` |
| Copy Object | `PUT` | `/{bucket}/{key}` + `x-amz-copy-source` |

### Multipart Upload

| Operation | Method | Path |
|-----------|--------|------|
| Create Upload | `POST` | `/{bucket}/{key}?uploads` |
| Upload Part | `PUT` | `/{bucket}/{key}?uploadId=X&partNumber=N` |
| Complete Upload | `POST` | `/{bucket}/{key}?uploadId=X` |
| Abort Upload | `DELETE` | `/{bucket}/{key}?uploadId=X` |

## Response Format

All responses use XML format:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult>
  <Name>my-bucket</Name>
  <Contents>
    <Key>file.txt</Key>
    <Size>1024</Size>
    <LastModified>2024-01-01T00:00:00.000Z</LastModified>
  </Contents>
</ListBucketResult>
```

## Common Headers

### Request Headers

| Header | Description |
|--------|-------------|
| `Authorization` | AWS Signature V4 |
| `x-amz-date` | Request timestamp |
| `x-amz-content-sha256` | Payload hash |
| `Content-Type` | MIME type |
| `Content-MD5` | Payload integrity |

### Response Headers

| Header | Description |
|--------|-------------|
| `ETag` | Object hash |
| `x-amz-request-id` | Request ID |
| `x-amz-version-id` | Object version |
| `Last-Modified` | Modification time |
