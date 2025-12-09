---
title: CLI Reference
description: Hafiz command-line interface
---

# CLI Reference

The `hafiz` CLI provides an AWS CLI-compatible interface for Hafiz.

## Installation

=== "Binary"

    ```bash
    curl -LO https://github.com/shellnoq/hafiz/releases/latest/download/hafiz-cli-linux-amd64.tar.gz
    tar xzf hafiz-cli-linux-amd64.tar.gz
    sudo mv hafiz /usr/local/bin/
    ```

=== "Cargo"

    ```bash
    cargo install hafiz-cli
    ```

=== "Homebrew"

    ```bash
    brew install shellnoq/tap/hafiz
    ```

## Configuration

```bash
hafiz configure
# Endpoint URL: http://localhost:9000
# Access Key: hafizadmin
# Secret Key: hafizadmin
```

Or use environment variables:

```bash
export HAFIZ_ENDPOINT=http://localhost:9000
export HAFIZ_ACCESS_KEY=hafizadmin
export HAFIZ_SECRET_KEY=hafizadmin
```

## Commands

| Command | Description |
|---------|-------------|
| `ls` | List buckets or objects |
| `cp` | Copy files |
| `mv` | Move files |
| `sync` | Synchronize directories |
| `rm` | Remove objects |
| `mb` | Make bucket |
| `rb` | Remove bucket |
| `head` | Get object metadata |
| `cat` | Stream object content |
| `du` | Disk usage |
| `presign` | Generate presigned URL |
| `configure` | Manage configuration |

[:octicons-arrow-right-24: Full Command Reference](commands.md)

## Quick Examples

```bash
# List buckets
hafiz ls s3://

# Upload file
hafiz cp file.txt s3://my-bucket/

# Download file
hafiz cp s3://my-bucket/file.txt ./

# Sync directory
hafiz sync ./local/ s3://my-bucket/backup/

# Delete object
hafiz rm s3://my-bucket/old-file.txt
```

## Global Options

```bash
hafiz [OPTIONS] <COMMAND>

Options:
  --endpoint <URL>       Endpoint URL
  --access-key <KEY>     Access key
  --secret-key <KEY>     Secret key
  --profile <n>       Config profile
  --output <FORMAT>      text or json
  -v, --verbose          Verbose output
  -q, --quiet            Quiet mode
  -h, --help             Help
```
