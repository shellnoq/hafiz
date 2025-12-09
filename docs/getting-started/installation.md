---
title: Installation
description: Install Hafiz on your system
---

# Installation

## Docker

```bash
docker pull ghcr.io/shellnoq/hafiz:latest
```

## Binary Downloads

Download from [GitHub Releases](https://github.com/shellnoq/hafiz/releases):

| Platform | Download |
|----------|----------|
| Linux (amd64) | `hafiz-linux-amd64.tar.gz` |
| Linux (arm64) | `hafiz-linux-arm64.tar.gz` |
| macOS (amd64) | `hafiz-darwin-amd64.tar.gz` |
| macOS (arm64) | `hafiz-darwin-arm64.tar.gz` |
| Windows | `hafiz-windows-amd64.zip` |

```bash
# Linux/macOS
curl -LO https://github.com/shellnoq/hafiz/releases/latest/download/hafiz-linux-amd64.tar.gz
tar xzf hafiz-linux-amd64.tar.gz
sudo mv hafiz-server /usr/local/bin/
```

## From Source

### Prerequisites

- Rust 1.75+
- PostgreSQL 13+ (optional)

### Build

```bash
git clone https://github.com/shellnoq/hafiz.git
cd hafiz
cargo build --release
```

### Install

```bash
sudo cp target/release/hafiz-server /usr/local/bin/
sudo cp target/release/hafiz /usr/local/bin/
```

## Verify

```bash
hafiz-server --version
```
