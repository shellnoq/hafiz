# Contributing to Hafiz

Thank you for your interest in contributing to Hafiz! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and constructive in all interactions. We aim to maintain a welcoming environment for all contributors.

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Git
- Docker (optional, for testing)

### Development Setup

```bash
# Clone the repository
git clone https://github.com/hafiz/hafiz.git
cd hafiz

# Build the project
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- server
```

### Project Structure

```
hafiz/
├── crates/
│   ├── hafiz-core/        # Core types and configuration
│   ├── hafiz-storage/     # Storage backend
│   ├── hafiz-metadata/    # Database operations
│   ├── hafiz-crypto/      # Encryption
│   ├── hafiz-auth/        # Authentication and authorization
│   ├── hafiz-s3-api/      # S3 REST API
│   ├── hafiz-cluster/     # Distributed clustering
│   ├── hafiz-admin/       # Admin UI (WebAssembly)
│   └── hafiz-cli/         # Command-line tool
├── deploy/                # Deployment configurations
├── docs/                  # Documentation
└── tests/                 # Integration tests
```

## How to Contribute

### Reporting Bugs

1. Check existing issues to avoid duplicates
2. Create a new issue with:
   - Clear title and description
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, Rust version, etc.)

### Suggesting Features

1. Check existing issues and discussions
2. Create a feature request with:
   - Use case description
   - Proposed solution
   - Alternatives considered

### Submitting Code

1. **Fork** the repository
2. **Create a branch** for your feature/fix:
   ```bash
   git checkout -b feature/my-feature
   ```
3. **Make your changes** following our coding standards
4. **Write tests** for your changes
5. **Run the test suite**:
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```
6. **Commit** with clear messages:
   ```bash
   git commit -m "feat: add support for X"
   ```
7. **Push** to your fork and create a Pull Request

### Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Test additions/changes
- `chore:` - Maintenance tasks

Examples:
```
feat: add support for S3 Select queries
fix: handle empty multipart uploads correctly
docs: update API documentation for versioning
refactor: simplify bucket policy evaluation
test: add integration tests for lifecycle rules
```

## Coding Standards

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Document public APIs with doc comments

### Code Organization

- Keep functions focused and small
- Use meaningful variable and function names
- Add comments for complex logic
- Prefer composition over inheritance

### Error Handling

- Use `anyhow::Result` for application errors
- Use `thiserror` for library errors
- Provide meaningful error messages
- Log errors at appropriate levels

### Testing

- Write unit tests for core logic
- Write integration tests for API endpoints
- Use descriptive test names
- Test edge cases and error conditions

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bucket_name_validation_accepts_valid_names() {
        assert!(is_valid_bucket_name("my-bucket"));
        assert!(is_valid_bucket_name("bucket.name.with.dots"));
    }

    #[test]
    fn test_bucket_name_validation_rejects_invalid_names() {
        assert!(!is_valid_bucket_name(""));
        assert!(!is_valid_bucket_name("ab")); // too short
        assert!(!is_valid_bucket_name("Uppercase")); // no uppercase
    }
}
```

## Pull Request Process

1. **Description**: Clearly describe what the PR does
2. **Tests**: Ensure all tests pass
3. **Documentation**: Update docs if needed
4. **Review**: Address reviewer feedback
5. **Merge**: Maintainers will merge approved PRs

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
How were these changes tested?

## Checklist
- [ ] Tests pass locally
- [ ] Code follows style guidelines
- [ ] Documentation updated
- [ ] No new warnings
```

## Development Tips

### Running Specific Tests

```bash
# Run tests for a specific crate
cargo test -p hafiz-s3-api

# Run a specific test
cargo test test_bucket_creation

# Run with output
cargo test -- --nocapture
```

### Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run -- server

# Enable trace logging for specific module
RUST_LOG=hafiz_s3_api=trace cargo run -- server
```

### Testing with AWS CLI

```bash
# Start the server
cargo run -- server

# In another terminal, configure AWS CLI
aws configure set aws_access_key_id minioadmin
aws configure set aws_secret_access_key minioadmin

# Test operations
aws --endpoint-url http://localhost:9000 s3 mb s3://test
aws --endpoint-url http://localhost:9000 s3 cp README.md s3://test/
aws --endpoint-url http://localhost:9000 s3 ls s3://test/
```

## Getting Help

- **Documentation**: Check the [docs](docs/) folder
- **Issues**: Search existing issues
- **Discussions**: Use GitHub Discussions for questions

## Recognition

Contributors are recognized in:
- Git commit history
- Release notes
- Contributors list

Thank you for contributing to Hafiz! 🎉
