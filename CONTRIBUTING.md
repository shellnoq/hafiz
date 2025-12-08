# Contributing to Hafiz

Thank you for your interest in contributing to Hafiz! This guide will help you get started.

## Code of Conduct

Please be respectful and constructive in all interactions. We're building something great together.

## Getting Started

### Prerequisites

- Rust 1.75 or later
- PostgreSQL 13+ (for integration tests)
- Docker (optional, for testing)
- Git

### Setup Development Environment

```bash
# Clone the repository
git clone https://github.com/shellnoq/hafiz.git
cd hafiz

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build

# Run tests
cargo test

# Run with hot reload (requires cargo-watch)
cargo install cargo-watch
cargo watch -x run
```

### Project Structure

```
hafiz/
├── crates/
│   ├── hafiz-core/       # Core types and utilities
│   ├── hafiz-s3-api/     # S3 API routes
│   ├── hafiz-storage/    # Storage backends
│   ├── hafiz-metadata/   # Database layer
│   ├── hafiz-auth/       # Authentication
│   ├── hafiz-crypto/     # Encryption
│   ├── hafiz-cluster/    # Clustering
│   ├── hafiz-admin/      # Admin API
│   └── hafiz-cli/        # CLI tool
├── deploy/               # Deployment configs
├── docs/                 # Documentation
└── tests/                # Integration tests
```

## How to Contribute

### Reporting Bugs

1. Check [existing issues](https://github.com/shellnoq/hafiz/issues)
2. Create a new issue with:
   - Clear title
   - Steps to reproduce
   - Expected vs actual behavior
   - Version and environment info

### Suggesting Features

1. Check [existing discussions](https://github.com/shellnoq/hafiz/discussions)
2. Open a new discussion with:
   - Use case description
   - Proposed solution
   - Alternatives considered

### Pull Requests

1. **Fork** the repository
2. **Create a branch**: `git checkout -b feature/my-feature`
3. **Make changes** with tests
4. **Run checks**: `cargo fmt && cargo clippy && cargo test`
5. **Commit**: Use [conventional commits](https://www.conventionalcommits.org/)
6. **Push**: `git push origin feature/my-feature`
7. **Open PR**: Against `main` branch

## Development Guidelines

### Code Style

```bash
# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings
```

### Commit Messages

Follow conventional commits:

```
feat: add bucket tagging support
fix: handle empty prefix in list objects
docs: update API reference
test: add versioning tests
refactor: simplify auth middleware
```

### Testing

```bash
# Unit tests
cargo test

# Integration tests (requires PostgreSQL)
cargo test --features integration

# Test specific crate
cargo test -p hafiz-s3-api

# Test with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Documentation

```bash
# Generate docs
cargo doc --no-deps --open

# Check doc comments
cargo doc --no-deps 2>&1 | grep warning
```

## Pull Request Checklist

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] New code has tests
- [ ] Documentation updated
- [ ] Changelog entry added
- [ ] Commit messages follow convention

## Architecture Decisions

When proposing significant changes:

1. Open a discussion first
2. Write an ADR (Architecture Decision Record) if needed
3. Get feedback before implementation

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create release PR
4. After merge, tag release: `git tag v0.1.0`
5. GitHub Actions builds and publishes

## Getting Help

- 💬 [GitHub Discussions](https://github.com/shellnoq/hafiz/discussions)
- 🐛 [Issue Tracker](https://github.com/shellnoq/hafiz/issues)

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.

---

Thank you for contributing! 🚀
