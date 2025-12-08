---
title: Error Codes
description: S3 API error responses
---

# Error Codes

All errors return XML responses:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<Error>
  <Code>NoSuchBucket</Code>
  <Message>The specified bucket does not exist</Message>
  <Resource>/my-bucket</Resource>
  <RequestId>request-id</RequestId>
</Error>
```

## Common Errors

### Client Errors (4xx)

| Code | HTTP | Description |
|------|------|-------------|
| `AccessDenied` | 403 | Access denied |
| `BucketAlreadyExists` | 409 | Bucket name taken |
| `BucketAlreadyOwnedByYou` | 409 | You own this bucket |
| `BucketNotEmpty` | 409 | Bucket has objects |
| `InvalidAccessKeyId` | 403 | Unknown access key |
| `InvalidArgument` | 400 | Invalid parameter |
| `InvalidBucketName` | 400 | Invalid bucket name |
| `InvalidRange` | 416 | Invalid byte range |
| `MalformedXML` | 400 | Bad XML |
| `MissingContentLength` | 411 | Missing header |
| `NoSuchBucket` | 404 | Bucket not found |
| `NoSuchKey` | 404 | Object not found |
| `NoSuchUpload` | 404 | Upload not found |
| `SignatureDoesNotMatch` | 403 | Invalid signature |

### Server Errors (5xx)

| Code | HTTP | Description |
|------|------|-------------|
| `InternalError` | 500 | Server error |
| `ServiceUnavailable` | 503 | Temporarily unavailable |

## Error Response Headers

| Header | Description |
|--------|-------------|
| `x-amz-request-id` | Request identifier |
| `x-amz-id-2` | Extended request ID |
| `Content-Type` | `application/xml` |

## Handling Errors

=== "Python"

    ```python
    from botocore.exceptions import ClientError

    try:
        s3.get_object(Bucket='bucket', Key='key')
    except ClientError as e:
        error_code = e.response['Error']['Code']
        if error_code == 'NoSuchKey':
            print("Object not found")
    ```

=== "JavaScript"

    ```javascript
    try {
      await client.send(new GetObjectCommand({...}));
    } catch (error) {
      if (error.name === 'NoSuchKey') {
        console.log('Object not found');
      }
    }
    ```
