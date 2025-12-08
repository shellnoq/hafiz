#!/bin/bash
# Docker Image Build and Push Script
# Usage: ./scripts/docker-publish.sh [registry] [version]

set -e

REGISTRY=${1:-"docker.io"}
VERSION=${2:-"latest"}
IMAGE_NAME="hafiz/hafiz"

echo "🐳 Building Docker image..."
echo "   Registry: $REGISTRY"
echo "   Image: $IMAGE_NAME"
echo "   Version: $VERSION"
echo ""

# Build image
docker build \
    -t "$IMAGE_NAME:$VERSION" \
    -t "$IMAGE_NAME:latest" \
    -f deployments/docker/Dockerfile \
    .

echo ""
echo "✅ Build complete!"
echo ""

# Ask for confirmation before pushing
read -p "Push to $REGISTRY? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "📤 Pushing to registry..."
    
    # Tag for registry
    docker tag "$IMAGE_NAME:$VERSION" "$REGISTRY/$IMAGE_NAME:$VERSION"
    docker tag "$IMAGE_NAME:latest" "$REGISTRY/$IMAGE_NAME:latest"
    
    # Push
    docker push "$REGISTRY/$IMAGE_NAME:$VERSION"
    docker push "$REGISTRY/$IMAGE_NAME:latest"
    
    echo ""
    echo "✅ Push complete!"
    echo ""
    echo "Pull command:"
    echo "  docker pull $REGISTRY/$IMAGE_NAME:$VERSION"
else
    echo "Push cancelled."
fi
