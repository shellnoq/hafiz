# Hafiz S3 API Reference

Hafiz implements the Amazon S3 REST API. This document covers all supported operations.

## Authentication

Hafiz supports AWS Signature Version 4 (SigV4) authentication.

### Headers

```
Authorization: AWS4-HMAC-SHA256 Credential=<access-key>/<date>/<region>/s3/aws4_request,
               SignedHeaders=<signed-headers>,
               Signature=<signature>
X-Amz-Date: <ISO8601 timestamp>
X-Amz-Content-SHA256: <payload hash or UNSIGNED-PAYLOAD>
```

### Presigned URLs

For temporary access without credentials:

```
https://hafiz.example.com/bucket/key?X-Amz-Algorithm=AWS4-HMAC-SHA256
  &X-Amz-Credential=<access-key>/<date>/<region>/s3/aws4_request
  &X-Amz-Date=<timestamp>
  &X-Amz-Expires=<seconds>
  &X-Amz-SignedHeaders=host
  &X-Amz-Signature=<signature>
```

---

## Service Operations

### ListBuckets

List all buckets owned by the authenticated user.

**Request:**
```http
GET / HTTP/1.1
Host: hafiz.example.com
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<ListAllMyBucketsResult>
  <Owner>
    <ID>owner-id</ID>
    <DisplayName>owner-name</DisplayName>
  </Owner>
  <Buckets>
    <Bucket>
      <Name>bucket-name</Name>
      <CreationDate>2024-01-15T12:00:00.000Z</CreationDate>
    </Bucket>
  </Buckets>
</ListAllMyBucketsResult>
```

---

## Bucket Operations

### CreateBucket

Create a new bucket.

**Request:**
```http
PUT /bucket-name HTTP/1.1
Host: hafiz.example.com
```

**Optional Body (for location constraint):**
```xml
<CreateBucketConfiguration>
  <LocationConstraint>us-west-2</LocationConstraint>
</CreateBucketConfiguration>
```

**Response:** `200 OK` with `Location` header

### DeleteBucket

Delete an empty bucket.

**Request:**
```http
DELETE /bucket-name HTTP/1.1
Host: hafiz.example.com
```

**Response:** `204 No Content`

### HeadBucket

Check if a bucket exists and you have access.

**Request:**
```http
HEAD /bucket-name HTTP/1.1
Host: hafiz.example.com
```

**Response:** `200 OK` or `404 Not Found`

### GetBucketLocation

Get the region of a bucket.

**Request:**
```http
GET /bucket-name?location HTTP/1.1
Host: hafiz.example.com
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<LocationConstraint>us-east-1</LocationConstraint>
```

### GetBucketVersioning

Get versioning status.

**Request:**
```http
GET /bucket-name?versioning HTTP/1.1
Host: hafiz.example.com
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<VersioningConfiguration>
  <Status>Enabled</Status>
</VersioningConfiguration>
```

### PutBucketVersioning

Enable or suspend versioning.

**Request:**
```http
PUT /bucket-name?versioning HTTP/1.1
Host: hafiz.example.com
Content-Type: application/xml

<VersioningConfiguration>
  <Status>Enabled</Status>
</VersioningConfiguration>
```

### GetBucketPolicy

Get bucket policy.

**Request:**
```http
GET /bucket-name?policy HTTP/1.1
Host: hafiz.example.com
```

**Response:** JSON policy document

### PutBucketPolicy

Set bucket policy.

**Request:**
```http
PUT /bucket-name?policy HTTP/1.1
Host: hafiz.example.com
Content-Type: application/json

{
  "Version": "2012-10-17",
  "Statement": [...]
}
```

### DeleteBucketPolicy

Delete bucket policy.

**Request:**
```http
DELETE /bucket-name?policy HTTP/1.1
Host: hafiz.example.com
```

### GetBucketCors

Get CORS configuration.

**Request:**
```http
GET /bucket-name?cors HTTP/1.1
Host: hafiz.example.com
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<CORSConfiguration>
  <CORSRule>
    <AllowedOrigin>*</AllowedOrigin>
    <AllowedMethod>GET</AllowedMethod>
    <AllowedMethod>PUT</AllowedMethod>
    <AllowedHeader>*</AllowedHeader>
    <MaxAgeSeconds>3600</MaxAgeSeconds>
  </CORSRule>
</CORSConfiguration>
```

### PutBucketCors

Set CORS configuration.

**Request:**
```http
PUT /bucket-name?cors HTTP/1.1
Host: hafiz.example.com
Content-Type: application/xml

<CORSConfiguration>
  <CORSRule>
    <AllowedOrigin>https://example.com</AllowedOrigin>
    <AllowedMethod>GET</AllowedMethod>
    <AllowedMethod>PUT</AllowedMethod>
    <AllowedHeader>*</AllowedHeader>
    <MaxAgeSeconds>3600</MaxAgeSeconds>
  </CORSRule>
</CORSConfiguration>
```

### DeleteBucketCors

Delete CORS configuration.

**Request:**
```http
DELETE /bucket-name?cors HTTP/1.1
Host: hafiz.example.com
```

### GetBucketLifecycleConfiguration

Get lifecycle rules.

**Request:**
```http
GET /bucket-name?lifecycle HTTP/1.1
Host: hafiz.example.com
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<LifecycleConfiguration>
  <Rule>
    <ID>expire-old-objects</ID>
    <Status>Enabled</Status>
    <Filter>
      <Prefix>logs/</Prefix>
    </Filter>
    <Expiration>
      <Days>90</Days>
    </Expiration>
  </Rule>
</LifecycleConfiguration>
```

### PutBucketLifecycleConfiguration

Set lifecycle rules.

**Request:**
```http
PUT /bucket-name?lifecycle HTTP/1.1
Host: hafiz.example.com
Content-Type: application/xml

<LifecycleConfiguration>
  <Rule>
    <ID>expire-old-objects</ID>
    <Status>Enabled</Status>
    <Filter>
      <Prefix>logs/</Prefix>
    </Filter>
    <Expiration>
      <Days>90</Days>
    </Expiration>
  </Rule>
</LifecycleConfiguration>
```

### GetBucketNotificationConfiguration

Get event notifications.

**Request:**
```http
GET /bucket-name?notification HTTP/1.1
Host: hafiz.example.com
```

### PutBucketNotificationConfiguration

Set event notifications.

**Request:**
```http
PUT /bucket-name?notification HTTP/1.1
Host: hafiz.example.com
Content-Type: application/xml

<NotificationConfiguration>
  <QueueConfiguration>
    <Id>webhook-1</Id>
    <Queue>arn:aws:sqs:us-east-1:000000000000:my-queue</Queue>
    <Event>s3:ObjectCreated:*</Event>
  </QueueConfiguration>
</NotificationConfiguration>
```

### GetBucketReplication

Get replication configuration.

**Request:**
```http
GET /bucket-name?replication HTTP/1.1
Host: hafiz.example.com
```

### PutBucketReplication

Set replication configuration.

**Request:**
```http
PUT /bucket-name?replication HTTP/1.1
Host: hafiz.example.com
Content-Type: application/xml

<ReplicationConfiguration>
  <Role>arn:aws:iam::000000000000:role/replication</Role>
  <Rule>
    <ID>replicate-all</ID>
    <Status>Enabled</Status>
    <Destination>
      <Bucket>arn:aws:s3:::destination-bucket</Bucket>
    </Destination>
  </Rule>
</ReplicationConfiguration>
```

---

## Object Operations

### ListObjectsV2

List objects in a bucket.

**Request:**
```http
GET /bucket-name?list-type=2&prefix=photos/&delimiter=/&max-keys=100 HTTP/1.1
Host: hafiz.example.com
```

**Query Parameters:**
- `list-type=2` (required for V2)
- `prefix` - Filter by prefix
- `delimiter` - Group by delimiter (usually `/`)
- `max-keys` - Maximum results (default: 1000)
- `continuation-token` - Pagination token
- `start-after` - Start listing after this key

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult>
  <Name>bucket-name</Name>
  <Prefix>photos/</Prefix>
  <Delimiter>/</Delimiter>
  <MaxKeys>100</MaxKeys>
  <IsTruncated>false</IsTruncated>
  <Contents>
    <Key>photos/image1.jpg</Key>
    <LastModified>2024-01-15T12:00:00.000Z</LastModified>
    <ETag>"abc123"</ETag>
    <Size>12345</Size>
    <StorageClass>STANDARD</StorageClass>
  </Contents>
  <CommonPrefixes>
    <Prefix>photos/2024/</Prefix>
  </CommonPrefixes>
</ListBucketResult>
```

### ListObjectVersions

List all versions of objects.

**Request:**
```http
GET /bucket-name?versions&prefix=docs/ HTTP/1.1
Host: hafiz.example.com
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<ListVersionsResult>
  <Name>bucket-name</Name>
  <Version>
    <Key>docs/file.txt</Key>
    <VersionId>v2</VersionId>
    <IsLatest>true</IsLatest>
    <LastModified>2024-01-15T12:00:00.000Z</LastModified>
    <ETag>"def456"</ETag>
    <Size>5678</Size>
  </Version>
  <Version>
    <Key>docs/file.txt</Key>
    <VersionId>v1</VersionId>
    <IsLatest>false</IsLatest>
    <LastModified>2024-01-14T12:00:00.000Z</LastModified>
    <ETag>"abc123"</ETag>
    <Size>1234</Size>
  </Version>
  <DeleteMarker>
    <Key>docs/deleted.txt</Key>
    <VersionId>dm1</VersionId>
    <IsLatest>true</IsLatest>
    <LastModified>2024-01-15T11:00:00.000Z</LastModified>
  </DeleteMarker>
</ListVersionsResult>
```

### GetObject

Download an object.

**Request:**
```http
GET /bucket-name/key HTTP/1.1
Host: hafiz.example.com
```

**Optional Headers:**
- `Range: bytes=0-1023` - Partial download
- `If-Modified-Since` - Conditional GET
- `If-None-Match` - Conditional GET

**Query Parameters:**
- `versionId` - Specific version

**Response Headers:**
- `Content-Type`
- `Content-Length`
- `ETag`
- `Last-Modified`
- `x-amz-version-id`

### PutObject

Upload an object.

**Request:**
```http
PUT /bucket-name/key HTTP/1.1
Host: hafiz.example.com
Content-Type: application/octet-stream
Content-Length: 12345
x-amz-meta-custom: value

<binary data>
```

**Optional Headers:**
- `Content-Type` - MIME type
- `x-amz-meta-*` - Custom metadata
- `x-amz-tagging` - URL-encoded tags
- `x-amz-storage-class` - Storage class

**Response Headers:**
- `ETag` - MD5 hash of content
- `x-amz-version-id` - Version ID (if versioning enabled)

### DeleteObject

Delete an object.

**Request:**
```http
DELETE /bucket-name/key HTTP/1.1
Host: hafiz.example.com
```

**Query Parameters:**
- `versionId` - Delete specific version

**Response:** `204 No Content`

### DeleteObjects

Delete multiple objects.

**Request:**
```http
POST /bucket-name?delete HTTP/1.1
Host: hafiz.example.com
Content-Type: application/xml

<Delete>
  <Quiet>false</Quiet>
  <Object>
    <Key>key1</Key>
  </Object>
  <Object>
    <Key>key2</Key>
    <VersionId>v1</VersionId>
  </Object>
</Delete>
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<DeleteResult>
  <Deleted>
    <Key>key1</Key>
  </Deleted>
  <Deleted>
    <Key>key2</Key>
    <VersionId>v1</VersionId>
  </Deleted>
</DeleteResult>
```

### HeadObject

Get object metadata without downloading.

**Request:**
```http
HEAD /bucket-name/key HTTP/1.1
Host: hafiz.example.com
```

**Response:** Headers only (same as GetObject)

### CopyObject

Copy an object.

**Request:**
```http
PUT /dest-bucket/dest-key HTTP/1.1
Host: hafiz.example.com
x-amz-copy-source: /source-bucket/source-key
```

**Optional Headers:**
- `x-amz-copy-source-if-match`
- `x-amz-copy-source-if-modified-since`
- `x-amz-metadata-directive: REPLACE` - Replace metadata

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<CopyObjectResult>
  <LastModified>2024-01-15T12:00:00.000Z</LastModified>
  <ETag>"abc123"</ETag>
</CopyObjectResult>
```

### GetObjectTagging

Get object tags.

**Request:**
```http
GET /bucket-name/key?tagging HTTP/1.1
Host: hafiz.example.com
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<Tagging>
  <TagSet>
    <Tag>
      <Key>environment</Key>
      <Value>production</Value>
    </Tag>
  </TagSet>
</Tagging>
```

### PutObjectTagging

Set object tags.

**Request:**
```http
PUT /bucket-name/key?tagging HTTP/1.1
Host: hafiz.example.com
Content-Type: application/xml

<Tagging>
  <TagSet>
    <Tag>
      <Key>environment</Key>
      <Value>production</Value>
    </Tag>
  </TagSet>
</Tagging>
```

### DeleteObjectTagging

Delete all tags.

**Request:**
```http
DELETE /bucket-name/key?tagging HTTP/1.1
Host: hafiz.example.com
```

---

## Multipart Upload Operations

### CreateMultipartUpload

Initiate multipart upload.

**Request:**
```http
POST /bucket-name/key?uploads HTTP/1.1
Host: hafiz.example.com
Content-Type: video/mp4
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<InitiateMultipartUploadResult>
  <Bucket>bucket-name</Bucket>
  <Key>key</Key>
  <UploadId>upload-id-12345</UploadId>
</InitiateMultipartUploadResult>
```

### UploadPart

Upload a part.

**Request:**
```http
PUT /bucket-name/key?partNumber=1&uploadId=upload-id-12345 HTTP/1.1
Host: hafiz.example.com
Content-Length: 5242880

<binary data>
```

**Response Headers:**
- `ETag` - Part ETag (required for completion)

### CompleteMultipartUpload

Complete the upload.

**Request:**
```http
POST /bucket-name/key?uploadId=upload-id-12345 HTTP/1.1
Host: hafiz.example.com
Content-Type: application/xml

<CompleteMultipartUpload>
  <Part>
    <PartNumber>1</PartNumber>
    <ETag>"part1-etag"</ETag>
  </Part>
  <Part>
    <PartNumber>2</PartNumber>
    <ETag>"part2-etag"</ETag>
  </Part>
</CompleteMultipartUpload>
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<CompleteMultipartUploadResult>
  <Location>https://hafiz.example.com/bucket-name/key</Location>
  <Bucket>bucket-name</Bucket>
  <Key>key</Key>
  <ETag>"final-etag"</ETag>
</CompleteMultipartUploadResult>
```

### AbortMultipartUpload

Cancel the upload.

**Request:**
```http
DELETE /bucket-name/key?uploadId=upload-id-12345 HTTP/1.1
Host: hafiz.example.com
```

### ListParts

List uploaded parts.

**Request:**
```http
GET /bucket-name/key?uploadId=upload-id-12345 HTTP/1.1
Host: hafiz.example.com
```

### ListMultipartUploads

List in-progress uploads.

**Request:**
```http
GET /bucket-name?uploads HTTP/1.1
Host: hafiz.example.com
```

---

## Object Lock Operations

### GetObjectLockConfiguration

Get bucket Object Lock configuration.

**Request:**
```http
GET /bucket-name?object-lock HTTP/1.1
Host: hafiz.example.com
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<ObjectLockConfiguration>
  <ObjectLockEnabled>Enabled</ObjectLockEnabled>
  <Rule>
    <DefaultRetention>
      <Mode>COMPLIANCE</Mode>
      <Days>2555</Days>
    </DefaultRetention>
  </Rule>
</ObjectLockConfiguration>
```

### PutObjectLockConfiguration

Set bucket Object Lock configuration.

**Request:**
```http
PUT /bucket-name?object-lock HTTP/1.1
Host: hafiz.example.com
Content-Type: application/xml

<ObjectLockConfiguration>
  <ObjectLockEnabled>Enabled</ObjectLockEnabled>
  <Rule>
    <DefaultRetention>
      <Mode>COMPLIANCE</Mode>
      <Days>2555</Days>
    </DefaultRetention>
  </Rule>
</ObjectLockConfiguration>
```

### GetObjectRetention

Get object retention settings.

**Request:**
```http
GET /bucket-name/key?retention HTTP/1.1
Host: hafiz.example.com
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<Retention>
  <Mode>COMPLIANCE</Mode>
  <RetainUntilDate>2031-12-31T23:59:59Z</RetainUntilDate>
</Retention>
```

### PutObjectRetention

Set object retention.

**Request:**
```http
PUT /bucket-name/key?retention HTTP/1.1
Host: hafiz.example.com
Content-Type: application/xml

<Retention>
  <Mode>COMPLIANCE</Mode>
  <RetainUntilDate>2031-12-31T23:59:59Z</RetainUntilDate>
</Retention>
```

### GetObjectLegalHold

Get legal hold status.

**Request:**
```http
GET /bucket-name/key?legal-hold HTTP/1.1
Host: hafiz.example.com
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<LegalHold>
  <Status>ON</Status>
</LegalHold>
```

### PutObjectLegalHold

Set legal hold.

**Request:**
```http
PUT /bucket-name/key?legal-hold HTTP/1.1
Host: hafiz.example.com
Content-Type: application/xml

<LegalHold>
  <Status>ON</Status>
</LegalHold>
```

---

## Error Responses

All errors return XML:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<Error>
  <Code>NoSuchBucket</Code>
  <Message>The specified bucket does not exist.</Message>
  <BucketName>nonexistent-bucket</BucketName>
  <RequestId>request-id-12345</RequestId>
</Error>
```

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `AccessDenied` | 403 | Access denied |
| `BucketAlreadyExists` | 409 | Bucket already exists |
| `BucketNotEmpty` | 409 | Bucket not empty |
| `EntityTooLarge` | 400 | Object too large |
| `InvalidAccessKeyId` | 403 | Invalid access key |
| `InvalidBucketName` | 400 | Invalid bucket name |
| `InvalidPart` | 400 | Invalid part |
| `InvalidPartOrder` | 400 | Parts not in order |
| `NoSuchBucket` | 404 | Bucket not found |
| `NoSuchKey` | 404 | Object not found |
| `NoSuchUpload` | 404 | Upload not found |
| `SignatureDoesNotMatch` | 403 | Invalid signature |

---

## Rate Limits

Default rate limits (configurable):

| Operation | Limit |
|-----------|-------|
| Requests per second | 1000 |
| PUT object size | 5GB (single), 5TB (multipart) |
| Part size | 5MB - 5GB |
| Parts per upload | 10,000 |
| Tags per object | 10 |
| Metadata per object | 2KB |

---

## SDK Examples

### Python (boto3)

```python
import boto3

s3 = boto3.client(
    's3',
    endpoint_url='http://localhost:9000',
    aws_access_key_id='minioadmin',
    aws_secret_access_key='minioadmin'
)

# Upload
s3.put_object(Bucket='mybucket', Key='test.txt', Body=b'Hello')

# Download
response = s3.get_object(Bucket='mybucket', Key='test.txt')
content = response['Body'].read()

# Presigned URL
url = s3.generate_presigned_url(
    'get_object',
    Params={'Bucket': 'mybucket', 'Key': 'test.txt'},
    ExpiresIn=3600
)
```

### JavaScript (AWS SDK v3)

```javascript
import { S3Client, PutObjectCommand } from '@aws-sdk/client-s3';
import { getSignedUrl } from '@aws-sdk/s3-request-presigner';

const client = new S3Client({
  endpoint: 'http://localhost:9000',
  region: 'us-east-1',
  credentials: {
    accessKeyId: 'minioadmin',
    secretAccessKey: 'minioadmin'
  },
  forcePathStyle: true
});

// Upload
await client.send(new PutObjectCommand({
  Bucket: 'mybucket',
  Key: 'test.txt',
  Body: 'Hello'
}));

// Presigned URL
const url = await getSignedUrl(client, new GetObjectCommand({
  Bucket: 'mybucket',
  Key: 'test.txt'
}), { expiresIn: 3600 });
```

### Go

```go
import (
    "github.com/aws/aws-sdk-go-v2/aws"
    "github.com/aws/aws-sdk-go-v2/service/s3"
)

client := s3.New(s3.Options{
    BaseEndpoint: aws.String("http://localhost:9000"),
    Region:       "us-east-1",
    Credentials: aws.CredentialsProviderFunc(func(ctx context.Context) (aws.Credentials, error) {
        return aws.Credentials{
            AccessKeyID:     "minioadmin",
            SecretAccessKey: "minioadmin",
        }, nil
    }),
    UsePathStyle: true,
})

// Upload
_, err := client.PutObject(context.TODO(), &s3.PutObjectInput{
    Bucket: aws.String("mybucket"),
    Key:    aws.String("test.txt"),
    Body:   strings.NewReader("Hello"),
})
```
