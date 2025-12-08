---
title: Object Operations
description: S3 API object operations
---

# Object Operations

## PutObject

Uploads an object.

**Request:**
```http
PUT /my-bucket/my-key HTTP/1.1
Host: s3.example.com
Content-Type: application/octet-stream
Content-Length: 1024

[binary data]
```

**Headers:**

| Header | Description |
|--------|-------------|
| `Content-Type` | MIME type |
| `Content-MD5` | Base64 MD5 for integrity |
| `x-amz-storage-class` | Storage class |
| `x-amz-server-side-encryption` | AES256 |
| `x-amz-meta-*` | Custom metadata |

**Response:**
```http
HTTP/1.1 200 OK
ETag: "d41d8cd98f00b204e9800998ecf8427e"
x-amz-version-id: version-id
```

---

## GetObject

Downloads an object.

**Request:**
```http
GET /my-bucket/my-key HTTP/1.1
Host: s3.example.com
```

**Optional Headers:**

| Header | Description |
|--------|-------------|
| `Range` | Byte range `bytes=0-1023` |
| `If-Match` | ETag condition |
| `If-None-Match` | ETag condition |
| `If-Modified-Since` | Date condition |

**Query Parameters:**

| Parameter | Description |
|-----------|-------------|
| `versionId` | Specific version |
| `response-content-type` | Override Content-Type |
| `response-content-disposition` | Override Content-Disposition |

---

## HeadObject

Gets metadata without body.

**Request:**
```http
HEAD /my-bucket/my-key HTTP/1.1
```

**Response:** Headers only, no body.

---

## DeleteObject

Deletes an object.

**Request:**
```http
DELETE /my-bucket/my-key HTTP/1.1
```

**Response:** `204 No Content`

---

## CopyObject

Copies an object.

**Request:**
```http
PUT /dest-bucket/dest-key HTTP/1.1
x-amz-copy-source: /source-bucket/source-key
```

---

## ListObjectsV2

Lists objects in a bucket.

**Request:**
```http
GET /my-bucket?list-type=2&prefix=folder/&max-keys=100 HTTP/1.1
```

**Parameters:**

| Parameter | Description |
|-----------|-------------|
| `list-type` | Must be `2` |
| `prefix` | Filter by prefix |
| `delimiter` | Grouping character |
| `max-keys` | Maximum results |
| `continuation-token` | Pagination |

**Response:**
```xml
<ListBucketResult>
  <n>my-bucket</n>
  <Contents>
    <Key>folder/file.txt</Key>
    <Size>1024</Size>
    <LastModified>2024-01-01T00:00:00.000Z</LastModified>
    <ETag>"etag"</ETag>
  </Contents>
</ListBucketResult>
```

---

## Multipart Upload

### CreateMultipartUpload

```http
POST /my-bucket/large-file?uploads HTTP/1.1
```

### UploadPart

```http
PUT /my-bucket/large-file?uploadId=X&partNumber=1 HTTP/1.1

[5MB+ binary data]
```

### CompleteMultipartUpload

```http
POST /my-bucket/large-file?uploadId=X HTTP/1.1

<CompleteMultipartUpload>
  <Part>
    <PartNumber>1</PartNumber>
    <ETag>"part-etag"</ETag>
  </Part>
</CompleteMultipartUpload>
```
