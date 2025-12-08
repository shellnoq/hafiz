---
title: Objects
description: Working with objects in Hafiz
---

# Objects

Objects are the fundamental entities stored in Hafiz. Each object consists of data, a key (name), and metadata.

## Uploading Objects

=== "AWS CLI"

    ```bash
    # Single file
    aws --endpoint-url http://localhost:9000 s3 cp file.txt s3://my-bucket/

    # With specific key
    aws --endpoint-url http://localhost:9000 s3 cp file.txt s3://my-bucket/folder/renamed.txt

    # Directory (recursive)
    aws --endpoint-url http://localhost:9000 s3 cp ./data/ s3://my-bucket/data/ --recursive
    ```

=== "Hafiz CLI"

    ```bash
    hafiz cp file.txt s3://my-bucket/
    hafiz cp -r ./data/ s3://my-bucket/data/
    ```

=== "Python"

    ```python
    # From file
    s3.upload_file('file.txt', 'my-bucket', 'file.txt')

    # From bytes
    s3.put_object(Bucket='my-bucket', Key='hello.txt', Body=b'Hello World!')
    ```

## Downloading Objects

```bash
# AWS CLI
aws --endpoint-url http://localhost:9000 s3 cp s3://my-bucket/file.txt ./

# Hafiz CLI
hafiz cp s3://my-bucket/file.txt ./
```

## Listing Objects

```bash
# List all objects
hafiz ls s3://my-bucket/

# With prefix
hafiz ls s3://my-bucket/folder/

# Long format (size, date)
hafiz ls -l s3://my-bucket/

# Human-readable sizes
hafiz ls -lH s3://my-bucket/
```

## Deleting Objects

```bash
# Single object
hafiz rm s3://my-bucket/file.txt

# Multiple objects (by prefix)
hafiz rm -r s3://my-bucket/old-logs/

# With confirmation
hafiz rm s3://my-bucket/important.txt  # Prompts for confirmation

# Force (no confirmation)
hafiz rm -f s3://my-bucket/file.txt
```

## Object Metadata

### Get Metadata

```bash
hafiz head s3://my-bucket/file.txt
```

Output:
```
s3://my-bucket/file.txt

  Content-Type: text/plain
  Content-Length: 1024 (1 KiB)
  Last-Modified: 2024-01-01 00:00:00 UTC
  ETag: "d41d8cd98f00b204e9800998ecf8427e"
  Storage-Class: STANDARD
```

### Custom Metadata

```bash
aws --endpoint-url http://localhost:9000 s3 cp file.txt s3://my-bucket/ \
    --metadata '{"author":"john","version":"1.0"}'
```

## Content Types

Hafiz automatically detects content types, or you can specify them:

```bash
hafiz cp image.png s3://my-bucket/ --content-type image/png
```

## Large Files

For files larger than 5GB, use multipart upload:

```bash
# Automatic with AWS CLI
aws --endpoint-url http://localhost:9000 s3 cp large-file.zip s3://my-bucket/

# Configure thresholds
aws configure set s3.multipart_threshold 64MB
aws configure set s3.multipart_chunksize 16MB
```

## Presigned URLs

Generate temporary URLs for direct access:

```bash
# Download URL (1 hour)
hafiz presign s3://my-bucket/file.txt

# Custom expiration (24 hours)
hafiz presign s3://my-bucket/file.txt --expires 86400

# Upload URL
hafiz presign s3://my-bucket/upload.txt --method PUT
```

## Streaming Content

```bash
# Stream to stdout
hafiz cat s3://my-bucket/log.txt

# Pipe to command
hafiz cat s3://my-bucket/data.json | jq .

# Pipe from command
echo "Hello" | hafiz cp - s3://my-bucket/hello.txt
```
