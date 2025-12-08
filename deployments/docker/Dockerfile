# Hafiz Dockerfile
# Multi-stage build for optimal image size

# ============= Builder Stage =============
FROM rust:1.75-bookworm AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/hafiz-core/Cargo.toml crates/hafiz-core/
COPY crates/hafiz-crypto/Cargo.toml crates/hafiz-crypto/
COPY crates/hafiz-storage/Cargo.toml crates/hafiz-storage/
COPY crates/hafiz-metadata/Cargo.toml crates/hafiz-metadata/
COPY crates/hafiz-auth/Cargo.toml crates/hafiz-auth/
COPY crates/hafiz-s3-api/Cargo.toml crates/hafiz-s3-api/
COPY crates/hafiz-cli/Cargo.toml crates/hafiz-cli/

# Create dummy source files for dependency compilation
RUN mkdir -p crates/hafiz-core/src && echo "pub fn dummy() {}" > crates/hafiz-core/src/lib.rs
RUN mkdir -p crates/hafiz-crypto/src && echo "pub fn dummy() {}" > crates/hafiz-crypto/src/lib.rs
RUN mkdir -p crates/hafiz-storage/src && echo "pub fn dummy() {}" > crates/hafiz-storage/src/lib.rs
RUN mkdir -p crates/hafiz-metadata/src && echo "pub fn dummy() {}" > crates/hafiz-metadata/src/lib.rs
RUN mkdir -p crates/hafiz-auth/src && echo "pub fn dummy() {}" > crates/hafiz-auth/src/lib.rs
RUN mkdir -p crates/hafiz-s3-api/src && echo "pub fn dummy() {}" > crates/hafiz-s3-api/src/lib.rs
RUN mkdir -p crates/hafiz-cli/src && echo "fn main() {}" > crates/hafiz-cli/src/main.rs

# Build dependencies only
RUN cargo build --release --package hafiz-cli 2>/dev/null || true

# Remove dummy source files
RUN rm -rf crates/*/src

# Copy actual source code
COPY crates/ crates/

# Touch source files to ensure rebuild
RUN find crates -name "*.rs" -exec touch {} \;

# Build the actual application
RUN cargo build --release --package hafiz-cli

# ============= Runtime Stage =============
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false -d /data novus

# Create data directories
RUN mkdir -p /data/hafiz /config && chown -R novus:novus /data /config

# Copy binary from builder
COPY --from=builder /build/target/release/hafiz /usr/local/bin/hafiz

# Set permissions
RUN chmod +x /usr/local/bin/hafiz

# Switch to non-root user
USER novus

# Set working directory
WORKDIR /data

# Expose ports
EXPOSE 9000 9001

# Environment variables with defaults
ENV HAFIZ_BIND_ADDRESS=0.0.0.0 \
    HAFIZ_PORT=9000 \
    HAFIZ_DATA_DIR=/data/hafiz \
    HAFIZ_DATABASE_URL=sqlite:///data/hafiz/hafiz.db?mode=rwc \
    HAFIZ_ROOT_ACCESS_KEY=minioadmin \
    HAFIZ_ROOT_SECRET_KEY=minioadmin \
    HAFIZ_LOG_LEVEL=info

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:9000/minio/health/live || exit 1

# Default command
CMD ["hafiz", "server"]
