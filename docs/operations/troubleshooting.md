---
title: Troubleshooting
description: Common issues and solutions
---

# Troubleshooting

## Common Issues

### Connection Refused

```bash
# Check if service is running
systemctl status hafiz

# Check port binding
ss -tlnp | grep 9000
```

### Authentication Failed

```bash
# Verify credentials
echo $HAFIZ_ROOT_ACCESS_KEY
echo $HAFIZ_ROOT_SECRET_KEY

# Check signature
aws --endpoint-url http://localhost:9000 s3 ls --debug
```

### Database Connection

```bash
# Test PostgreSQL
psql -h localhost -U hafiz -d hafiz -c "SELECT 1"

# Check connection string
echo $HAFIZ_DATABASE_URL
```

### Disk Full

```bash
# Check disk space
df -h /data

# Find large objects
hafiz du -H s3://
```

## Logs

### Docker

```bash
docker logs hafiz
docker logs -f hafiz  # Follow
```

### Kubernetes

```bash
kubectl logs -f deployment/hafiz -n hafiz
```

### Systemd

```bash
journalctl -u hafiz -f
```

## Debug Mode

```bash
HAFIZ_LOG_LEVEL=debug hafiz-server
```

## Getting Help

- [GitHub Issues](https://github.com/shellnoq/hafiz/issues)
- [Discussions](https://github.com/shellnoq/hafiz/discussions)
