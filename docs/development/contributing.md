---
title: Contributing
description: Contributing to Hafiz
---

# Contributing

## Getting Started

```bash
# Clone
git clone https://github.com/shellnoq/hafiz.git
cd hafiz

# Build
cargo build

# Test
cargo test

# Format
cargo fmt

# Lint
cargo clippy
```

## Pull Request Process

1. Fork the repository
2. Create a branch: `git checkout -b feature/my-feature`
3. Make changes with tests
4. Run `cargo fmt && cargo clippy && cargo test`
5. Commit using conventional commits
6. Push and create PR

## Commit Messages

```
feat: add bucket tagging
fix: handle empty prefix
docs: update API reference
test: add versioning tests
```

## Code Style

- Run `cargo fmt` before committing
- No clippy warnings
- Add tests for new features
- Document public APIs

## Getting Help

- [GitHub Issues](https://github.com/shellnoq/hafiz/issues)
- [Discussions](https://github.com/shellnoq/hafiz/discussions)
