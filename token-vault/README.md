# Token Vault 🔐

[![Crates.io](https://img.shields.io/crates/v/token-vault)](https://crates.io/crates/token-vault)
[![Documentation](https://docs.rs/token-vault/badge.svg)](https://docs.rs/token-vault)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/token-vault)](LICENSE)
[![Build Status](https://github.com/SuperInstance/token-vault/workflows/CI/badge.svg)](https://github.com/SuperInstance/token-vault/actions)

> Secure token storage system with AES-256-GCM encryption-at-rest for API keys, secrets, and credentials.

**Token Vault** provides military-grade encryption for your sensitive data with a simple, secure API. All secrets are encrypted with AES-256-GCM before storage, using Argon2id key derivation for password-based encryption.

## 🚀 Quick Start

```toml
# Cargo.toml
[dependencies]
token-vault = "0.1"
```

```rust
use token_vault::TokenVault;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create vault with password
    let vault = TokenVault::new("my_vault.db", "my-secure-password")?;

    // Store API tokens
    vault.store("github_token", "ghp_1234567890abcdef", None)?;

    // Retrieve securely
    let token = vault.retrieve("github_token", None)?;
    assert_eq!(token, Some("ghp_1234567890abcdef".to_string()));

    Ok(())
}
```

## ✨ Features

- **🔒 Encryption-at-rest**: AES-256-GCM encryption for all stored secrets
- **🔑 Key derivation**: Argon2id password-based key derivation (256 MB RAM, 3 iterations)
- **📦 Session isolation**: Multiple isolated sessions with separate token namespaces
- **⏰ Token expiration**: Automatic expiration and rotation support
- **👥 Access control**: Role-based access control (RBAC)
- **📋 Audit logging**: Comprehensive audit trail for all operations
- **🧹 Memory security**: Automatic zeroization of sensitive data with `zeroize`
- **🔌 Thread-safe**: `Arc<Mutex<T>>` for concurrent access

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     Token Vault                          │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  User Password                                          │
│       │                                                  │
│       ▼                                                  │
│  Argon2id Key Derivation (t=3, m=256MiB, p=4)           │
│       │                                                  │
│       ▼                                                  │
│  Master Key (256-bit)                                   │
│       │                                                  │
│       ├──▶ Encryption Key (AES-256-GCM)                 │
│       │      │                                          │
│       │      ▼                                          │
│       │  Encrypted Secrets (SQLite)                     │
│       │      │                                          │
│       │      └──▶ Stored on Disk                        │
│       │                                                 │
│       └──▶ HMAC Key (authentication)                    │
│              │                                          │
│              └──▶ Verify operations                      │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## 📖 Documentation

### Core Operations

#### Create Vault

```rust
let vault = TokenVault::new("path/to/vault.db", "master-password")?;
```

#### Store Token

```rust
vault.store("api_key", "sk_live_1234567890", None)?;
vault.store("db_password", "secret123", Some("production"))?;
```

#### Retrieve Token

```rust
let value = vault.retrieve("api_key", None)?;
```

#### Update Token

```rust
vault.update("api_key", "new_secret_value", None)?;
```

#### Delete Token

```rust
vault.delete("api_key", None)?;
```

#### List Tokens

```rust
let tokens = vault.list_tokens(Some("production"))?;
for token in tokens {
    println!("{}", token);
}
```

### Session Management

```rust
// Use sessions to isolate different contexts
vault.store("api_key", "prod_key", Some("production"))?;
vault.store("api_key", "dev_key", Some("development"))?;

// Retrieve from specific session
let prod_key = vault.retrieve("api_key", Some("production"))?;
let dev_key = vault.retrieve("api_key", Some("development"))?;
```

### Audit Logging

```rust
let entries = vault.audit_entries();
for entry in entries {
    println!("{:?} {} - {:?}", entry.operation, entry.target, entry.result);
}
```

## 🔐 Security

### Threat Model

Token Vault protects against:

1. **Compromised Database**: AES-256-GCM encryption-at-rest
2. **Memory Dump Attacks**: Zeroization with `zeroize` crate
3. **Password Brute Force**: Argon2id with high memory/time cost
4. **SQL Injection**: Parameterized queries only

See [`threat_model`](https://docs.rs/token-vault/#threat-model) for detailed analysis.

### Key Derivation Parameters

- **Algorithm**: Argon2id
- **Time Cost**: 3 iterations
- **Memory Cost**: 256 MiB (262,144 KiB)
- **Parallelism**: 4 lanes
- **Output**: 256-bit key

### Encryption

- **Algorithm**: AES-256-GCM
- **Nonce**: 96-bit (random per encryption)
- **Authentication**: GCM auth tag (128-bit)

## 🎯 Use Cases

```rust
// 1. Application Configuration
let config = AppConfig::new("config.db", "password")?;
let db_url = config.get_database_url()?;

// 2. API Key Storage
vault.store("stripe_secret", "sk_live_...", None)?;
vault.store("github_token", "ghp_...", None)?;

// 3. Database Credentials
vault.store("db_password", "secure_password", Some("production"))?;

// 4. Service Tokens
vault.store("jwt_secret", "jwt_signing_key", None)?;
```

## 📦 CLI Tools

Token Vault includes three CLI tools:

### Server (token-vault-server)

```bash
token-vault-server
# Starts HTTP server on 0.0.0.0:8080
# TODO: Full implementation coming soon
```

### Client (token-vault-client)

```bash
token-vault-client get <token>
token-vault-client set <token> <value>
token-vault-client list
# TODO: Full implementation coming soon
```

### Admin (token-vault-admin)

```bash
token-vault-admin init <db-path>
token-vault-admin backup <db-path> <backup-file>
token-vault-admin restore <backup-file> <db-path>
# TODO: Full implementation coming soon
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_store_and_retrieve
```

## 📚 Examples

See the [`examples/`](examples/) directory for complete examples:

- [`basic_vault.rs`](examples/basic_vault.rs) - Basic CRUD operations
- [`custom_integration.rs`](examples/custom_integration.rs) - Application integration
- [`server_client.rs`](examples/server_client.rs) - Server/client usage (TODO)
- [`backup_restore.rs`](examples/backup_restore.rs) - Backup/restore (TODO)

Run examples:

```bash
cargo run --example basic_vault
cargo run --example custom_integration
```

## 🔧 Installation

```bash
cargo install token-vault
```

Or add to your `Cargo.toml`:

```toml
[dependencies]
token-vault = "0.1"
```

## 📊 Performance

- **Store**: O(1) - Single INSERT with indexed lookup
- **Retrieve**: O(1) - Single SELECT with index
- **Encrypt**: ~1µs per 1KB (AES-256-GCM hardware accelerated)
- **Decrypt**: ~1µs per 1KB (AES-256-GCM hardware accelerated)

## 🤝 Integration

### With Privox (PII Redaction)

```rust
// TODO: Add privox integration example
// Use privox to redact PII in audit logs
```

## 📝 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🙏 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 🔗 Related Projects

- [privox](https://github.com/SuperInstance/privox) - Privacy redaction engine
- [tripartite-rs](https://github.com/SuperInstance/tripartite-rs) - Multi-agent consensus
- [knowledge-vault](https://github.com/SuperInstance/knowledge-vault) - Vector database

## 📮 Contact

- GitHub Issues: https://github.com/SuperInstance/token-vault/issues
- Discussions: https://github.com/SuperInstance/token-vault/discussions

## 🌟 Used By

- [SuperInstance](https://github.com/SuperInstance) - Multi-agent orchestration platform

---

**Made with ❤️ by the SuperInstance team**

[⬆ Back to top](#token-vault-)
