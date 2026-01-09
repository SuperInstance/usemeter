# Contributing to Model Registry

Thank you for your interest in contributing to Model Registry!

## Development Setup

1. **Clone the repository**

```bash
git clone https://github.com/SuperInstance/model-registry.git
cd model-registry
```

2. **Install dependencies**

```bash
cargo fetch
```

3. **Run tests**

```bash
cargo test --all-features
```

4. **Run examples**

```bash
cargo run --example basic_registry
cargo run --example download_model
cargo run --example multiple_versions
cargo run --example search_and_query
cargo run --example s3_storage --features s3-storage
```

## Code Style

We use standard Rust formatting:

```bash
cargo fmt --all
```

And linting:

```bash
cargo clippy --all-targets --all-features
```

## Testing

Run the full test suite:

```bash
cargo test --all-features
```

Run tests with coverage:

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Documentation

Build documentation:

```bash
cargo doc --all-features --no-deps --open
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linting
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## Commit Messages

Use clear, descriptive commit messages:

- `feat: Add S3 storage backend`
- `fix: Correct progress reporting for large files`
- `docs: Update README with new examples`
- `test: Add integration tests for download`

## License

By contributing, you agree that your contributions will be licensed under the MIT OR Apache-2.0 license.
