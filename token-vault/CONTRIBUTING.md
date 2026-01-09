# Contributing to Token Vault

Thank you for your interest in contributing to Token Vault! This document provides guidelines and instructions for contributing.

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to [CONTRIBUTING.md contact].

## How to Contribute

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates. When creating a bug report, include:

- **Title**: Clear and descriptive
- **Description**: Detailed explanation of the problem
- **Reproduction Steps**: Steps to reproduce the issue
- **Expected Behavior**: What you expected to happen
- **Actual Behavior**: What actually happened
- **Environment**: OS, Rust version, token-vault version
- **Logs**: Relevant error messages or logs

### Suggesting Enhancements

Enhancement suggestions are welcome! Please include:

- **Use Case**: What problem would this solve?
- **Proposed Solution**: Detailed description of your idea
- **Alternatives**: Other approaches you considered
- **Impact**: How would this benefit users?

### Pull Requests

1. **Fork** the repository
2. **Create a branch** for your change (`git checkout -b feature/amazing-feature`)
3. **Make your changes** with clear, descriptive commit messages
4. **Add tests** for new functionality
5. **Ensure all tests pass** (`cargo test`)
6. **Format code** (`cargo fmt`)
7. **Run linter** (`cargo clippy`)
8. **Commit** your changes (`git commit -m 'Add amazing feature'`)
9. **Push** to the branch (`git push origin feature/amazing-feature`)
10. **Open a Pull Request**

### Development Setup

```bash
# Clone repository
git clone https://github.com/SuperInstance/token-vault.git
cd token-vault

# Install dependencies
cargo build

# Run tests
cargo test

# Run with examples
cargo run --example basic_vault
```

### Coding Standards

- **Formatting**: Use `cargo fmt` for consistent formatting
- **Linting**: Use `cargo clippy` and address warnings
- **Testing**: Maintain 100% test pass rate
- **Documentation**: Document public APIs with rustdoc comments
- **Security**: Follow security best practices (see below)

### Security Guidelines

When working with security-sensitive code:

1. **Never hardcode credentials** in code or tests
2. **Use secure random generation** (`getrandom`, not `rand::thread_rng()`)
3. **Zeroize sensitive data** after use
4. **Follow constant-time practices** for comparisons
5. **Validate all inputs** before processing
6. **Document security considerations** in code comments

### Commit Messages

Follow conventional commit format:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Build process or auxiliary tool changes

Example:
```
feat(encryption): add ChaCha20-Poly1305 cipher option

Add support for ChaCha20-Poly1305 as an alternative to AES-GCM.
This provides better performance on mobile devices without AES
hardware acceleration.

Closes #123
```

### Testing

Write tests for all new functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_functionality() {
        // Arrange
        let input = "test";

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, "expected");
    }
}
```

Run tests before committing:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_new_functionality
```

### Documentation

Document all public APIs:

```rust
/// Does something amazing
///
/// # Arguments
///
/// * `arg1` - Description of argument
/// * `arg2` - Description of argument
///
/// # Returns
///
/// Description of return value
///
/// # Examples
///
/// ```
/// use token_vault::function;
///
/// let result = function("test");
/// assert_eq!(result, "expected");
/// ```
///
/// # Errors
///
/// This function will return an error if...
pub fn amazing_function(arg1: &str, arg2: &str) -> Result<String> {
    // ...
}
```

## Review Process

All submissions go through review:

1. **Automated Checks**: CI runs tests, formatting, and linting
2. **Manual Review**: Maintainers review code quality and design
3. **Feedback**: You may receive feedback for improvements
4. **Approval**: At least one maintainer must approve
5. **Merge**: Changes are merged after approval

## Getting Help

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and ideas
- **Discord/Slack**: For real-time discussion (if available)

## License

By contributing, you agree that your contributions will be licensed under the [Apache-2.0 OR MIT](LICENSE) license.

Thank you for contributing to Token Vault! 🎉
