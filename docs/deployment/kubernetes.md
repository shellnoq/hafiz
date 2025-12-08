---
title: Kubernetes Deployment
description: Running Hafiz on Kubernetes with Helm
---

# Kubernetes Deployment

## Prerequisites

- Kubernetes 1.23+
- Helm 3.8+
- kubectl configured

## Installation

### Add Helm Repository

```bash
helm repo add hafiz https://shellnoq.github.io/hafiz
helm repo update
```

### Create Namespace

```bash
kubectl create namespace hafiz
```

### Create Secrets

```bash
# Generate credentials
kubectl create secret generic hafiz-credentials \
  --namespace hafiz \
  --from-literal=access-key=$(openssl rand -hex 10) \
  --from-literal=secret-key=$(openssl rand -hex 20)

# Encryption key
kubectl create secret generic hafiz-encryption \
  --namespace hafiz \
  --from-literal=master-key=$(openssl rand -base64 32)
```

### Install

```bash
helm install hafiz hafiz/hafiz \
  --namespace hafiz \
  --set replicaCount=3 \
  --set hafiz.auth.existingSecret=hafiz-credentials
```

## Configuration

### values.yaml

```yaml
replicaCount: 3

image:
  repository: ghcr.io/shellnoq/hafiz
  tag: latest

resources:
  requests:
    cpu: 500m
    memory: 1Gi
  limits:
    cpu: 2000m
    memory: 4Gi

persistence:
  enabled: true
  size: 100Gi

ingress:
  enabled: true
  hosts:
    - host: s3.example.com
      paths:
        - path: /
          pathType: Prefix
```

### Install with Values

```bash
helm install hafiz hafiz/hafiz \
  --namespace hafiz \
  -f values.yaml
```

## Verify

```bash
# Check pods
kubectl get pods -n hafiz

# Check services
kubectl get svc -n hafiz

# Port forward
kubectl port-forward svc/hafiz 9000:9000 -n hafiz

# Test
aws --endpoint-url http://localhost:9000 s3 ls
```

## Upgrade

```bash
helm upgrade hafiz hafiz/hafiz \
  --namespace hafiz \
  -f values.yaml
```

## Uninstall

```bash
helm uninstall hafiz --namespace hafiz
kubectl delete pvc -l app.kubernetes.io/name=hafiz -n hafiz
```
