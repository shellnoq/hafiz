#!/bin/bash
# Hafiz Development Setup Script

set -e

echo "🚀 Setting up Hafiz development environment..."

# Check Rust installation
if ! command -v cargo &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Check Rust version
RUST_VERSION=$(rustc --version | cut -d' ' -f2)
echo "✅ Rust version: $RUST_VERSION"

# Install required tools
echo "📦 Installing development tools..."
rustup component add clippy rustfmt

# Create data directory
echo "📁 Creating data directories..."
mkdir -p data/hafiz

# Build project
echo "🔨 Building project..."
cargo build

echo ""
echo "✅ Setup complete!"
echo ""
echo "To start the server, run:"
echo "  cargo run --package hafiz-cli -- server"
echo ""
echo "Or with Docker:"
echo "  make docker"
echo "  make docker-run"
echo ""
echo "Default credentials:"
echo "  Access Key: minioadmin"
echo "  Secret Key: minioadmin"
echo ""
