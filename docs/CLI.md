# Hafiz CLI Reference

The `hafiz` command-line tool provides an AWS CLI-compatible interface for interacting with Hafiz.

## Installation

### From Binary

```bash
# Linux
curl -LO https://github.com/shellnoq/hafiz/releases/latest/download/hafiz-cli-linux-amd64.tar.gz
tar xzf hafiz-cli-linux-amd64.tar.gz
sudo mv hafiz /usr/local/bin/

# macOS
brew install shellnoq/tap/hafiz
```

### From Source

```bash
cargo install hafiz-cli
```

## Configuration

### Interactive Setup

```bash
hafiz configure
# Endpoint URL []: http://localhost:9000
# Access Key []: minioadmin
# Secret Key []: minioadmin
# Region [us-east-1]: 
```

### Environment Variables

```bash
export HAFIZ_ENDPOINT=http://localhost:9000
export HAFIZ_ACCESS_KEY=minioadmin
export HAFIZ_SECRET_KEY=minioadmin
export HAFIZ_REGION=us-east-1
```

### Config File

Location: `~/.hafiz/config.toml`

```toml
[default]
endpoint = "http://localhost:9000"
access_key = "minioadmin"
secret_key = "minioadmin"
region = "us-east-1"

[production]
endpoint = "https://s3.example.com"
access_key = "prod-access-key"
secret_key = "prod-secret-key"
```

Use profile: `hafiz --profile production ls s3://`

## Global Options

```bash
hafiz [OPTIONS] <COMMAND>

Options:
  --endpoint <URL>       Endpoint URL
  --access-key <KEY>     Access key ID
  --secret-key <KEY>     Secret access key
  --region <REGION>      AWS region [default: us-east-1]
  --profile <NAME>       Configuration profile
  --output <FORMAT>      Output format: text, json [default: text]
  -v, --verbose          Verbose output
  -q, --quiet            Quiet mode
  -h, --help             Print help
  -V, --version          Print version
```

---

## Commands

### ls - List Buckets/Objects

```bash
# List all buckets
hafiz ls s3://

# List objects in bucket
hafiz ls s3://my-bucket/

# List with prefix
hafiz ls s3://my-bucket/folder/

# Long format with details
hafiz ls -l s3://my-bucket/

# Human-readable sizes
hafiz ls -lH s3://my-bucket/

# Recursive listing
hafiz ls -r s3://my-bucket/

# Summary only
hafiz ls --summarize s3://my-bucket/
```

### cp - Copy Files

```bash
# Upload file
hafiz cp file.txt s3://my-bucket/

# Upload with different name
hafiz cp file.txt s3://my-bucket/renamed.txt

# Download file
hafiz cp s3://my-bucket/file.txt ./

# Download to specific path
hafiz cp s3://my-bucket/file.txt ./downloads/

# Copy between S3 locations
hafiz cp s3://bucket1/file.txt s3://bucket2/

# Recursive copy (directory)
hafiz cp -r ./folder/ s3://my-bucket/backup/

# With include/exclude patterns
hafiz cp -r ./logs/ s3://my-bucket/logs/ \
  --include "*.log" \
  --exclude "*.tmp"

# Dry run
hafiz cp -r ./data/ s3://my-bucket/ --dryrun

# Set content type
hafiz cp image.png s3://my-bucket/ --content-type image/png

# Set storage class
hafiz cp file.txt s3://my-bucket/ --storage-class REDUCED_REDUNDANCY
```

### mv - Move Files

```bash
# Move local to S3
hafiz mv file.txt s3://my-bucket/

# Move S3 to S3
hafiz mv s3://bucket1/file.txt s3://bucket2/

# Move directory
hafiz mv -r ./folder/ s3://my-bucket/
```

### sync - Synchronize Directories

```bash
# Sync local to S3
hafiz sync ./local/ s3://my-bucket/prefix/

# Sync S3 to local
hafiz sync s3://my-bucket/prefix/ ./local/

# Delete files not in source
hafiz sync ./local/ s3://my-bucket/ --delete

# Exclude patterns
hafiz sync ./local/ s3://my-bucket/ --exclude "*.tmp"

# Only sync if size differs
hafiz sync ./local/ s3://my-bucket/ --size-only

# Dry run
hafiz sync ./local/ s3://my-bucket/ --dryrun
```

### rm - Remove Objects

```bash
# Delete single object
hafiz rm s3://my-bucket/file.txt

# Delete with confirmation
hafiz rm s3://my-bucket/file.txt  # Prompts: Delete? [y/N]

# Force delete (no confirmation)
hafiz rm -f s3://my-bucket/file.txt

# Delete recursively
hafiz rm -r s3://my-bucket/folder/

# Delete with pattern
hafiz rm -r s3://my-bucket/ --include "*.log"

# Dry run
hafiz rm -r s3://my-bucket/logs/ --dryrun
```

### mb - Make Bucket

```bash
# Create bucket
hafiz mb s3://new-bucket

# Create with region
hafiz mb s3://new-bucket --region eu-west-1
```

### rb - Remove Bucket

```bash
# Delete empty bucket
hafiz rb s3://my-bucket

# Force delete (delete all objects first)
hafiz rb s3://my-bucket --force
```

### head - Get Object Metadata

```bash
hafiz head s3://my-bucket/file.txt

# Output:
# s3://my-bucket/file.txt
# 
#   Content-Type: text/plain
#   Content-Length: 1024 (1 KiB)
#   Last-Modified: 2024-01-01 00:00:00 UTC
#   ETag: "d41d8cd98f00b204e9800998ecf8427e"
#   Storage-Class: STANDARD
```

### cat - Stream Object Content

```bash
# Print to stdout
hafiz cat s3://my-bucket/file.txt

# Pipe to command
hafiz cat s3://my-bucket/data.json | jq .

# Save to file
hafiz cat s3://my-bucket/file.txt > local.txt
```

### du - Disk Usage

```bash
# Calculate usage
hafiz du s3://my-bucket/

# Human-readable
hafiz du -H s3://my-bucket/

# Summary only
hafiz du -sH s3://my-bucket/

# Output:
#      1.2 GiB     1024 obj  s3://my-bucket/folder1/
#    512 MiB      500 obj  s3://my-bucket/folder2/
#      1.7 GiB     1524 obj  s3://my-bucket/ (total)
```

### presign - Generate Presigned URL

```bash
# GET URL (default 1 hour)
hafiz presign s3://my-bucket/file.txt

# Custom expiration (seconds)
hafiz presign s3://my-bucket/file.txt --expires 86400

# PUT URL for uploads
hafiz presign s3://my-bucket/upload.txt --method PUT
```

### info - Bucket/Object Info

```bash
# Bucket info
hafiz info s3://my-bucket

# Output:
# s3://my-bucket
# 
#   Region: us-east-1
#   Versioning: Enabled
#   Objects: 1524
#   Total Size: 1.7 GiB
```

### configure - Manage Configuration

```bash
# Interactive setup
hafiz configure

# Set single value
hafiz configure set endpoint http://localhost:9000

# Get value
hafiz configure get endpoint

# List all settings
hafiz configure list

# Add profile
hafiz configure add-profile production

# Remove profile
hafiz configure remove-profile old-profile
```

---

## Examples

### Backup Script

```bash
#!/bin/bash
DATE=$(date +%Y%m%d)
SOURCE=/var/www/html
BUCKET=s3://backups

# Sync website files
hafiz sync "$SOURCE/" "$BUCKET/website/$DATE/" --delete

# Keep only last 7 days
for OLD in $(hafiz ls "$BUCKET/website/" | head -n -7 | awk '{print $NF}'); do
  hafiz rm -rf "$BUCKET/website/$OLD"
done
```

### Log Rotation

```bash
#!/bin/bash
# Upload and compress logs
for LOG in /var/log/app/*.log; do
  gzip -c "$LOG" | hafiz cp - "s3://logs/$(basename $LOG).gz"
done
```

### Batch Processing

```bash
# Process all JSON files
hafiz ls s3://data/input/ | while read KEY; do
  hafiz cat "s3://data/$KEY" | process.py | hafiz cp - "s3://data/output/$KEY"
done
```

---

## Tips

### Faster Transfers

```bash
# Increase parallelism
hafiz cp -r ./large-folder/ s3://bucket/ --parallel 8
```

### JSON Output

```bash
# For scripting
hafiz ls s3://bucket/ --output json | jq '.objects[].key'
```

### Debugging

```bash
# Verbose mode
hafiz -v cp file.txt s3://bucket/

# See all debug info
RUST_LOG=debug hafiz cp file.txt s3://bucket/
```
