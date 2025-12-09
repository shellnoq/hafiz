---
title: System Requirements
description: Hardware, software, and OS requirements for Hafiz
---

# System Requirements

## Supported Operating Systems

| OS | Version | Support Level | Notes |
|----|---------|---------------|-------|
| **Ubuntu** | 22.04 LTS, 24.04 LTS | ✅ Recommended | Best tested, production ready |
| **Debian** | 11, 12 | ✅ Full | Stable, enterprise ready |
| **RHEL/Rocky/Alma** | 8, 9 | ✅ Full | Enterprise Linux |
| **Amazon Linux** | 2, 2023 | ✅ Full | AWS optimized |
| **macOS** | 13+ (Ventura) | ⚠️ Development | Not for production |
| **Windows** | Server 2019+ | ⚠️ Experimental | Docker recommended |

!!! tip "Recommended OS"
    **Ubuntu 24.04 LTS** is the recommended operating system for production deployments. It offers the best balance of stability, security updates, and community support.

## Hardware Requirements

### Minimum (Development/Testing)

| Resource | Requirement |
|----------|-------------|
| CPU | 1 core (x86_64 or ARM64) |
| Memory | 512 MB RAM |
| Disk | 1 GB free space |
| Network | Any |

### Recommended (Production)

| Resource | Small | Medium | Large |
|----------|-------|--------|-------|
| CPU | 4 cores | 8 cores | 16+ cores |
| Memory | 4 GB | 16 GB | 64+ GB |
| Disk | 100 GB SSD | 1 TB NVMe | 10+ TB NVMe |
| Network | 1 Gbps | 10 Gbps | 25+ Gbps |
| Nodes | 1 | 3 | 5+ |

### Storage Recommendations

| Workload | Disk Type | RAID | Notes |
|----------|-----------|------|-------|
| Development | Any | - | Local disk fine |
| Production | NVMe SSD | RAID 10 | Best performance |
| Archive | HDD | RAID 6 | Cost effective |
| Cloud | Cloud SSD | - | EBS gp3, Persistent Disk |

!!! warning "Avoid"
    - Network filesystems (NFS, CIFS) for data storage
    - Shared storage without proper locking
    - Spinning disks for metadata (use SSD)

## Software Dependencies

### Runtime

| Dependency | Version | Required | Notes |
|------------|---------|----------|-------|
| Linux Kernel | 5.4+ | Yes | For io_uring support |
| glibc | 2.31+ | Yes | Standard C library |
| OpenSSL | 1.1.1+ / 3.x | Yes | TLS support |

### Optional

| Dependency | Version | For |
|------------|---------|-----|
| PostgreSQL | 13+ | Production metadata |
| Docker | 20.10+ | Container deployment |
| Kubernetes | 1.23+ | Orchestration |

### Build from Source

| Dependency | Version | Notes |
|------------|---------|-------|
| Rust | 1.75+ | `rustup` recommended |
| Cargo | 1.75+ | Comes with Rust |
| GCC/Clang | 11+ | C compiler |
| pkg-config | Any | Build tool |
| OpenSSL dev | 1.1.1+ | `libssl-dev` |

## Network Requirements

### Ports

| Port | Protocol | Service | Required |
|------|----------|---------|----------|
| 9000 | TCP | S3 API | Yes |
| 9001 | TCP | Admin API/UI | Optional |
| 9090 | TCP | Metrics | Optional |
| 7946 | TCP/UDP | Cluster gossip | Cluster only |

### Firewall Rules

```bash
# S3 API (required)
ufw allow 9000/tcp

# Admin UI (optional)
ufw allow 9001/tcp

# Metrics (optional)
ufw allow 9090/tcp

# Cluster communication (if clustering)
ufw allow 7946/tcp
ufw allow 7946/udp
```

### DNS

- Forward and reverse DNS recommended
- Static IP or stable DHCP lease for production

## Filesystem Requirements

| Path | Purpose | Recommended Size | Permissions |
|------|---------|------------------|-------------|
| `/data` | Object storage | 90% of total | `hafiz:hafiz` (750) |
| `/var/lib/hafiz` | Metadata (SQLite) | 10 GB | `hafiz:hafiz` (750) |
| `/var/log/hafiz` | Logs | 10 GB | `hafiz:hafiz` (750) |
| `/etc/hafiz` | Configuration | 1 MB | `root:hafiz` (640) |

### Filesystem Recommendations

| Filesystem | Support | Notes |
|------------|---------|-------|
| **XFS** | ✅ Recommended | Best for large files |
| **ext4** | ✅ Full | Good general purpose |
| **ZFS** | ✅ Full | Advanced features |
| **Btrfs** | ⚠️ Experimental | Use with caution |

## Memory Considerations

Hafiz memory usage scales with:

- **Concurrent connections** - ~1 MB per connection
- **Multipart uploads** - ~5 MB per active upload
- **Metadata cache** - Configurable, default 256 MB
- **Encryption buffers** - ~64 KB per encrypted operation

### Tuning

```bash
# Recommended sysctl settings
vm.swappiness=10
vm.dirty_ratio=40
vm.dirty_background_ratio=10
net.core.somaxconn=65535
net.ipv4.tcp_max_syn_backlog=65535
```

## Cloud Provider Recommendations

### AWS

| Component | Recommended |
|-----------|-------------|
| Instance | m6i.xlarge (4 vCPU, 16 GB) |
| Storage | gp3 (3000 IOPS, 125 MB/s) |
| Network | Enhanced networking |

### GCP

| Component | Recommended |
|-----------|-------------|
| Instance | e2-standard-4 |
| Storage | pd-ssd |
| Network | Premium tier |

### Azure

| Component | Recommended |
|-----------|-------------|
| Instance | Standard_D4s_v5 |
| Storage | Premium SSD |
| Network | Accelerated networking |

## Quick Compatibility Check

Run this script to verify your system:

```bash
#!/bin/bash
echo "=== Hafiz System Check ==="

# OS
echo -n "OS: "
cat /etc/os-release | grep PRETTY_NAME | cut -d'"' -f2

# Kernel
echo -n "Kernel: "
uname -r

# CPU
echo -n "CPU Cores: "
nproc

# Memory
echo -n "Memory: "
free -h | awk '/^Mem:/ {print $2}'

# Disk
echo -n "Disk Free: "
df -h / | awk 'NR==2 {print $4}'

# Rust (if building from source)
if command -v rustc &> /dev/null; then
    echo -n "Rust: "
    rustc --version
fi

# Docker (if using containers)
if command -v docker &> /dev/null; then
    echo -n "Docker: "
    docker --version
fi

echo "========================="
```

## Next Steps

Once your system meets the requirements:

- [Quick Start](quickstart.md) - Get Hafiz running in 5 minutes
- [Installation](installation.md) - Detailed installation guide
- [Configuration](configuration.md) - Configure for your environment
