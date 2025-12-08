---
title: Bucket Operations
description: S3 API bucket operations
---

# Bucket Operations

## ListBuckets

Lists all buckets owned by the authenticated user.

**Request:**
```http
GET / HTTP/1.1
Host: s3.example.com
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<ListAllMyBucketsResult>
  <Owner>
    <ID>user-id</ID>
    <DisplayName>username</DisplayName>
  </Owner>
  <Buckets>
    <Bucket>
      <n>my-bucket</n>
      <CreationDate>2024-01-01T00:00:00.000Z</CreationDate>
    </Bucket>
  </Buckets>
</ListAllMyBucketsResult>
```

---

## CreateBucket

Creates a new bucket.

**Request:**
```http
PUT /my-bucket HTTP/1.1
Host: s3.example.com
```

**With region:**
```xml
<CreateBucketConfiguration>
  <LocationConstraint>eu-west-1</LocationConstraint>
</CreateBucketConfiguration>
```

**Response:** `200 OK`

---

## DeleteBucket

Deletes an empty bucket.

**Request:**
```http
DELETE /my-bucket HTTP/1.1
Host: s3.example.com
```

**Response:** `204 No Content`

!!! warning
    Bucket must be empty. Delete all objects first.

---

## HeadBucket

Checks if a bucket exists and you have access.

**Request:**
```http
HEAD /my-bucket HTTP/1.1
Host: s3.example.com
```

**Response:** `200 OK` or `404 Not Found`

---

## GetBucketLocation

Returns the region of a bucket.

**Request:**
```http
GET /my-bucket?location HTTP/1.1
Host: s3.example.com
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<LocationConstraint>us-east-1</LocationConstraint>
```

---

## GetBucketVersioning

Gets versioning state.

**Request:**
```http
GET /my-bucket?versioning HTTP/1.1
```

**Response:**
```xml
<VersioningConfiguration>
  <Status>Enabled</Status>
</VersioningConfiguration>
```

---

## PutBucketVersioning

Enables or suspends versioning.

**Request:**
```http
PUT /my-bucket?versioning HTTP/1.1

<VersioningConfiguration>
  <Status>Enabled</Status>
</VersioningConfiguration>
```

---

## GetBucketPolicy

Gets bucket policy.

**Request:**
```http
GET /my-bucket?policy HTTP/1.1
```

**Response:** JSON policy document

---

## PutBucketPolicy

Sets bucket policy.

**Request:**
```http
PUT /my-bucket?policy HTTP/1.1
Content-Type: application/json

{"Version": "2012-10-17", "Statement": [...]}
```
