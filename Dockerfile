# ============================================================================
# Hafiz Production Dockerfile
# ============================================================================
# Multi-stage build for minimal, secure production image
#
# Build:
#   docker build -t ghcr.io/shellnoq/hafiz:latest .
#
# Run:
#   docker run -d -p 9000:9000 -p 9001:9001 \
#     -v hafiz-data:/data \
#     -e HAFIZ_ROOT_ACCESS_KEY=your-key \
#     -e HAFIZ_ROOT_SECRET_KEY=your-secret \
#     ghcr.io/shellnoq/hafiz:latest
# ============================================================================

# ----------------------------------------------------------------------------
# Stage 1: Chef - Cargo dependency caching
# ----------------------------------------------------------------------------
FROM rust:1.75-bookworm AS chef

RUN cargo install cargo-chef
WORKDIR /build

# ----------------------------------------------------------------------------
# Stage 2: Planner - Generate dependency recipe
# ----------------------------------------------------------------------------
FROM chef AS planner

COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

RUN cargo chef prepare --recipe-path recipe.json

# ----------------------------------------------------------------------------
# Stage 3: Builder - Compile the application
# ----------------------------------------------------------------------------
FROM chef AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency recipe and build dependencies (cached)
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

# Build release binaries
RUN cargo build --release --bin hafiz-server --bin hafiz

# Strip debug symbols for smaller binaries
RUN strip target/release/hafiz-server target/release/hafiz

# ----------------------------------------------------------------------------
# Stage 4: Runtime - Minimal production image
# ----------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# OCI Labels
LABEL org.opencontainers.image.title="Hafiz" \
      org.opencontainers.image.description="Enterprise-grade S3-compatible object storage written in Rust" \
      org.opencontainers.image.vendor="Shellnoq" \
      org.opencontainers.image.source="https://github.com/shellnoq/hafiz" \
      org.opencontainers.image.documentation="https://docs.hafiz.e2esolutions.tech" \
      org.opencontainers.image.licenses="Apache-2.0"

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    tini \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN groupadd -r -g 1000 hafiz \
    && useradd -r -u 1000 -g hafiz -d /home/hafiz -s /sbin/nologin hafiz

# Create directories with proper permissions
RUN mkdir -p \
    /data \
    /data/objects \
    /data/metadata \
    /etc/hafiz \
    /var/log/hafiz \
    && chown -R hafiz:hafiz /data /etc/hafiz /var/log/hafiz

# Copy binaries from builder
COPY --from=builder /build/target/release/hafiz-server /usr/local/bin/
COPY --from=builder /build/target/release/hafiz /usr/local/bin/

# Set executable permissions
RUN chmod +x /usr/local/bin/hafiz-server /usr/local/bin/hafiz

# Environment variables with sensible defaults
ENV HAFIZ_S3_BIND=0.0.0.0 \
    HAFIZ_S3_PORT=9000 \
    HAFIZ_ADMIN_BIND=0.0.0.0 \
    HAFIZ_ADMIN_PORT=9001 \
    HAFIZ_METRICS_PORT=9090 \
    HAFIZ_STORAGE_BASE_PATH=/data/objects \
    HAFIZ_METADATA_PATH=/data/metadata \
    HAFIZ_LOG_LEVEL=info \
    HAFIZ_LOG_FORMAT=json \
    RUST_BACKTRACE=0

# Ports
# 9000 - S3 API
# 9001 - Admin API/UI
# 9090 - Prometheus metrics
# 7946 - Cluster gossip (TCP/UDP)
EXPOSE 9000 9001 9090 7946

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -sf http://localhost:9000/health || exit 1

# Volume for persistent data
VOLUME ["/data"]

# Switch to non-root user
USER hafiz
WORKDIR /home/hafiz

# Use tini as init system for proper signal handling
ENTRYPOINT ["/usr/bin/tini", "--"]

# Default command
CMD ["hafiz-server"]
