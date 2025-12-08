---
title: Backup & Restore
description: Backup and restore procedures
---

# Backup & Restore

## What to Backup

1. **Metadata** - PostgreSQL database
2. **Data** - Object storage files
3. **Configuration** - Settings and secrets

## Metadata Backup

### PostgreSQL

```bash
# Backup
pg_dump -h localhost -U hafiz hafiz > hafiz-backup-$(date +%Y%m%d).sql

# Restore
psql -h localhost -U hafiz hafiz < hafiz-backup-20240101.sql
```

### Kubernetes CronJob

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: hafiz-backup
spec:
  schedule: "0 2 * * *"
  jobTemplate:
    spec:
      template:
        spec:
          containers:
            - name: backup
              image: postgres:15
              command:
                - pg_dump
                - -h
                - postgres
                - -U
                - hafiz
              volumeMounts:
                - name: backup
                  mountPath: /backup
```

## Data Backup

### Filesystem

```bash
# Backup
tar czf hafiz-data-$(date +%Y%m%d).tar.gz /data

# Restore
tar xzf hafiz-data-20240101.tar.gz -C /
```

### Volume Snapshots

Use cloud provider volume snapshots for faster recovery.

## Disaster Recovery

1. Restore PostgreSQL from backup
2. Restore data files
3. Start Hafiz
4. Verify with health check
