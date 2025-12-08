# ============================================================================
# Hafiz Production Dockerfile
# ============================================================================
# Multi-stage build for optimized production image
#
# Build: docker build -t hafiz:latest .
# Run:   docker run -p 9000:9000 -v hafiz-data:/data/hafiz hafiz:latest
# ============================================================================

# ----------------------------------------------------------------------------
# Stage 1: Build the Rust backend
# ----------------------------------------------------------------------------
FROM rust:1.75-bookworm AS backend-builder

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
COPY crates/hafiz-admin/Cargo.toml crates/hafiz-admin/

# Create dummy source files for dependency caching
RUN mkdir -p crates/hafiz-core/src && echo "pub fn dummy() {}" > crates/hafiz-core/src/lib.rs && \
    mkdir -p crates/hafiz-crypto/src && echo "pub fn dummy() {}" > crates/hafiz-crypto/src/lib.rs && \
    mkdir -p crates/hafiz-storage/src && echo "pub fn dummy() {}" > crates/hafiz-storage/src/lib.rs && \
    mkdir -p crates/hafiz-metadata/src && echo "pub fn dummy() {}" > crates/hafiz-metadata/src/lib.rs && \
    mkdir -p crates/hafiz-auth/src && echo "pub fn dummy() {}" > crates/hafiz-auth/src/lib.rs && \
    mkdir -p crates/hafiz-s3-api/src && echo "pub fn dummy() {}" > crates/hafiz-s3-api/src/lib.rs && \
    mkdir -p crates/hafiz-cli/src && echo "fn main() {}" > crates/hafiz-cli/src/main.rs && \
    mkdir -p crates/hafiz-admin/src && echo "pub fn dummy() {}" > crates/hafiz-admin/src/lib.rs

# Build dependencies only (cached layer)
RUN cargo build --release --package hafiz-cli 2>/dev/null || true

# Copy actual source code
COPY crates/ crates/

# Touch source files to invalidate cache
RUN find crates -name "*.rs" -exec touch {} \;

# Build the final binary
RUN cargo build --release --package hafiz-cli

# Strip debug symbols for smaller binary
RUN strip /build/target/release/hafiz-cli

# ----------------------------------------------------------------------------
# Stage 2: Build the WASM frontend
# ----------------------------------------------------------------------------
FROM rust:1.75-bookworm AS frontend-builder

WORKDIR /build

# Install wasm-pack and trunk
RUN cargo install wasm-pack trunk
RUN rustup target add wasm32-unknown-unknown

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/hafiz-admin/Cargo.toml crates/hafiz-admin/

# Copy frontend source
COPY crates/hafiz-admin/ crates/hafiz-admin/

# Build WASM
WORKDIR /build/crates/hafiz-admin
RUN trunk build --release 2>/dev/null || \
    (wasm-pack build --target web --release && \
     mkdir -p dist && \
     cp -r pkg/* dist/)

# ----------------------------------------------------------------------------
# Stage 3: Production runtime image
# ----------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# Labels
LABEL org.opencontainers.image.title="Hafiz"
LABEL org.opencontainers.image.description="S3-compatible object storage server"
LABEL org.opencontainers.image.vendor="Hafiz"
LABEL org.opencontainers.image.source="https://github.com/hafiz/hafiz"
LABEL org.opencontainers.image.licenses="Apache-2.0"

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -u 1000 -g root novus

# Create directories
RUN mkdir -p /data/hafiz /data/hafiz/certs /etc/hafiz /app/static \
    && chown -R novus:root /data/hafiz /etc/hafiz /app

# Copy binary from builder
COPY --from=backend-builder /build/target/release/hafiz-cli /usr/local/bin/hafiz

# Copy frontend assets (if built successfully)
COPY --from=frontend-builder /build/crates/hafiz-admin/dist/ /app/static/ 2>/dev/null || true

# Copy default config
COPY config/config.example.toml /etc/hafiz/config.toml.example

# Set permissions
RUN chmod +x /usr/local/bin/hafiz

# Environment defaults
ENV HAFIZ_BIND_ADDRESS=0.0.0.0 \
    HAFIZ_PORT=9000 \
    HAFIZ_DATA_DIR=/data/hafiz \
    HAFIZ_DATABASE_URL=sqlite:///data/hafiz/hafiz.db?mode=rwc \
    HAFIZ_LOG_LEVEL=info \
    HAFIZ_ROOT_ACCESS_KEY=minioadmin \
    HAFIZ_ROOT_SECRET_KEY=minioadmin

# Expose ports
EXPOSE 9000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:9000/metrics || exit 1

# Volume for persistent data
VOLUME ["/data/hafiz"]

# Run as non-root user
USER novus

# Entry point
ENTRYPOINT ["/usr/local/bin/hafiz"]
CMD ["serve"]
