# Contributing to usemeter

Thank you for your interest in contributing to usemeter! This document provides guidelines and instructions for contributing.

## 🚀 Getting Started

### Prerequisites

- Rust 1.70 or later
- Git
- Cargo

### Development Setup

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/usemeter.git
   cd usemeter
   ```

3. Install development dependencies:
   ```bash
   cargo build
   cargo test
   ```

## 📋 Development Workflow

### 1. Branch Strategy

Create a branch for your contribution:
```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 2. Make Changes

- Write code following the existing style
- Add tests for new functionality
- Update documentation as needed

### 3. Test Your Changes

```bash
# Run all tests
cargo test --all

# Run with coverage (optional)
cargo tarpaulin --out Html

# Run clippy
cargo clippy --all-targets --all-features

# Check formatting
cargo fmt --all -- --check
```

### 4. Commit Your Changes

Follow conventional commit format:
```
feat: add PostgreSQL storage backend
fix: resolve race condition in event insertion
docs: update billing calculation examples
test: add integration tests for alerting
```

### 5. Submit Pull Request

- Push to your fork
- Create pull request to SuperInstance/usemeter
- Fill in the PR template
- Wait for review

## 📝 Code Style

### Formatting

Use `rustfmt`:
```bash
cargo fmt --all
```

### Linting

Fix all clippy warnings:
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Documentation

- Document all public APIs
- Include examples in documentation
- Run `cargo doc` to check:
  ```bash
  cargo doc --all-features --no-deps
  ```

## 🧪 Testing

### Unit Tests

Write tests alongside code in the same module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Test implementation
    }
}
```

### Integration Tests

Add integration tests in `tests/` directory:
```bash
tests/
├── integration_tests.rs
└── billing_tests.rs
```

### Benchmarks

Add benchmarks in `benches/` directory:
```bash
cargo bench
```

## 📚 Documentation Guidelines

### API Documentation

- Document all public items
- Include examples for complex APIs
- Explain error conditions

```rust
/// Records a usage event.
///
/// # Errors
///
/// Returns an error if:
/// - The event is invalid
/// - Storage backend is unavailable
///
/// # Example
///
/// ```rust
/// use usemeter::{Event, Meter};
/// # async fn example(meter: Meter) -> Result<(), Box<dyn std::error::Error>> {
/// meter.record(event).await?;
/// # Ok(())
/// # }
/// ```
pub async fn record(&self, event: Event) -> Result<(), Error>;
```

### README Documentation

Update README for:
- New features
- API changes
- New examples

## 🐛 Bug Reports

Use the GitHub issue tracker and include:

- Rust version (`rustc --version`)
- usemeter version
- Minimal reproduction code
- Expected vs actual behavior
- Stack traces if available

## 💡 Feature Requests

We welcome feature requests! Please:

- Check existing issues first
- Describe the use case clearly
- Provide examples if possible
- Explain why it's important

## 📦 Release Process

Releases are managed by maintainers:

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create git tag
4. Publish to crates.io
5. Create GitHub release

## 🤝 Code Review

### Reviewer Guidelines

- Check code quality and style
- Ensure tests are adequate
- Verify documentation is complete
- Test on multiple platforms if needed

### Author Response

- Address review comments promptly
- Update tests as requested
- Revise documentation as needed

## 🎯 Priority Areas

We especially welcome contributions to:

- Additional storage backends (PostgreSQL, MySQL)
- Performance optimizations
- Integration examples
- Documentation improvements
- Test coverage

## 📜 License

By contributing, you agree that your contributions will be licensed under the MIT OR Apache-2.0 license, same as the project.

## 💬 Communication

- GitHub Issues: Bug reports, feature requests
- GitHub Discussions: General questions
- PRs: Code changes and reviews

## 🙏 Thank You

Thank you for contributing to usemeter! Your contributions help make it better for everyone.

---

For questions or clarifications, please open an issue or discussion on GitHub.
