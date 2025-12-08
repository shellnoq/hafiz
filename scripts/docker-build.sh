#!/bin/bash
# ============================================================================
# Hafiz Docker Build Script
# ============================================================================
#
# Builds and optionally pushes Hafiz Docker images
#
# Usage:
#   ./scripts/docker-build.sh                    # Build only
#   ./scripts/docker-build.sh --push             # Build and push
#   ./scripts/docker-build.sh --platform linux/amd64,linux/arm64  # Multi-arch
#
# ============================================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
IMAGE_NAME="${DOCKER_IMAGE:-hafiz/hafiz}"
VERSION="${VERSION:-$(git describe --tags --always 2>/dev/null || echo "dev")}"
PLATFORMS="${PLATFORMS:-linux/amd64}"
PUSH=false
CACHE=true

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --push)
            PUSH=true
            shift
            ;;
        --no-cache)
            CACHE=false
            shift
            ;;
        --platform)
            PLATFORMS="$2"
            shift 2
            ;;
        --version)
            VERSION="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --push              Push image to registry"
            echo "  --no-cache          Build without cache"
            echo "  --platform PLAT     Target platforms (default: linux/amd64)"
            echo "  --version VER       Image version tag"
            echo ""
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║       Hafiz Docker Build              ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""

echo -e "${GREEN}Image:${NC}     $IMAGE_NAME"
echo -e "${GREEN}Version:${NC}   $VERSION"
echo -e "${GREEN}Platforms:${NC} $PLATFORMS"
echo -e "${GREEN}Push:${NC}      $PUSH"
echo ""

# Check if buildx is available for multi-platform builds
if [[ "$PLATFORMS" == *","* ]]; then
    if ! docker buildx version &>/dev/null; then
        echo -e "${RED}Docker buildx is required for multi-platform builds${NC}"
        exit 1
    fi
    
    # Create builder if not exists
    if ! docker buildx inspect hafiz-builder &>/dev/null; then
        echo -e "${YELLOW}Creating buildx builder...${NC}"
        docker buildx create --name hafiz-builder --use
    else
        docker buildx use hafiz-builder
    fi
fi

# Build arguments
BUILD_ARGS=(
    --file Dockerfile
    --tag "$IMAGE_NAME:$VERSION"
    --tag "$IMAGE_NAME:latest"
    --label "org.opencontainers.image.version=$VERSION"
    --label "org.opencontainers.image.created=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
)

if [[ "$CACHE" == false ]]; then
    BUILD_ARGS+=(--no-cache)
fi

if [[ "$PLATFORMS" == *","* ]]; then
    # Multi-platform build with buildx
    BUILD_ARGS+=(--platform "$PLATFORMS")
    
    if [[ "$PUSH" == true ]]; then
        BUILD_ARGS+=(--push)
    else
        BUILD_ARGS+=(--load)
        echo -e "${YELLOW}Note: Multi-platform builds require --push or only load current platform${NC}"
    fi
    
    echo -e "${BLUE}Building multi-platform image...${NC}"
    docker buildx build "${BUILD_ARGS[@]}" .
else
    # Single platform build
    echo -e "${BLUE}Building image...${NC}"
    docker build "${BUILD_ARGS[@]}" .
    
    if [[ "$PUSH" == true ]]; then
        echo -e "${BLUE}Pushing image...${NC}"
        docker push "$IMAGE_NAME:$VERSION"
        docker push "$IMAGE_NAME:latest"
    fi
fi

echo ""
echo -e "${GREEN}✓ Build complete!${NC}"
echo ""
echo "Run with:"
echo -e "  ${BLUE}docker run -p 9000:9000 $IMAGE_NAME:$VERSION${NC}"
echo ""
echo "Or with docker-compose:"
echo -e "  ${BLUE}docker-compose up -d${NC}"
