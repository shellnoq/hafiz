---
title: CLI Commands
description: Complete CLI command reference
---

# CLI Commands

## ls - List

```bash
# List buckets
hafiz ls s3://

# List objects
hafiz ls s3://my-bucket/

# Long format
hafiz ls -l s3://my-bucket/

# Human-readable sizes
hafiz ls -lH s3://my-bucket/

# Recursive
hafiz ls -r s3://my-bucket/
```

## cp - Copy

```bash
# Upload
hafiz cp file.txt s3://my-bucket/

# Download
hafiz cp s3://my-bucket/file.txt ./

# Recursive
hafiz cp -r ./data/ s3://my-bucket/data/

# With patterns
hafiz cp -r ./logs/ s3://my-bucket/ --include "*.log" --exclude "*.tmp"

# Dry run
hafiz cp -r ./data/ s3://my-bucket/ --dryrun
```

## sync - Synchronize

```bash
# Upload sync
hafiz sync ./local/ s3://my-bucket/

# Download sync
hafiz sync s3://my-bucket/ ./local/

# Delete extra files
hafiz sync ./local/ s3://my-bucket/ --delete

# Dry run
hafiz sync ./local/ s3://my-bucket/ --dryrun
```

## rm - Remove

```bash
# Single file
hafiz rm s3://my-bucket/file.txt

# Recursive
hafiz rm -r s3://my-bucket/old-data/

# Force (no confirmation)
hafiz rm -f s3://my-bucket/file.txt
```

## mb/rb - Bucket Operations

```bash
# Create bucket
hafiz mb s3://new-bucket

# Delete bucket
hafiz rb s3://empty-bucket

# Force delete
hafiz rb s3://bucket --force
```

## head - Metadata

```bash
hafiz head s3://my-bucket/file.txt

# Output:
#   Content-Type: text/plain
#   Content-Length: 1024
#   ETag: "abc123"
```

## cat - Stream Content

```bash
# Print to stdout
hafiz cat s3://my-bucket/file.txt

# Pipe to command
hafiz cat s3://my-bucket/data.json | jq .
```

## du - Disk Usage

```bash
# Calculate usage
hafiz du s3://my-bucket/

# Human-readable
hafiz du -H s3://my-bucket/
```

## presign - Presigned URL

```bash
# Generate URL (1 hour)
hafiz presign s3://my-bucket/file.txt

# Custom expiration
hafiz presign s3://my-bucket/file.txt --expires 86400
```

## configure - Configuration

```bash
# Interactive setup
hafiz configure

# Set value
hafiz configure set endpoint http://localhost:9000

# Get value
hafiz configure get endpoint

# List all
hafiz configure list
```
