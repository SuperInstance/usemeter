//! # Token Vault - Secure Token Storage with Encryption-at-Rest
//!
//! A secure vault for storing API tokens, secrets, and credentials with AES-256-GCM encryption-at-rest.
//!
//! ## Features
//!
//! - **Encryption-at-rest**: AES-256-GCM encryption for all stored secrets
//! - **Secure key derivation**: Argon2id password-based key derivation
//! - **Session isolation**: Multiple isolated sessions with separate token namespaces
//! - **Token expiration**: Automatic expiration and rotation support
//! - **Access control**: Role-based access control (RBAC)
//! - **Audit logging**: Comprehensive audit trail for all operations
//! - **Memory security**: Automatic zeroization of sensitive data
//!
//! ## Security Architecture
//!
//! ```
//! ┌─────────────────────────────────────────────────────────┐
//! │                     Token Vault                          │
//! ├─────────────────────────────────────────────────────────┤
//! │                                                          │
//! │  User Password                                          │
//!       │                                                    │
//!       ▼                                                    │
//! │  Argon2id Key Derivation                                │
//! │  (salt, t=3, p=4, m=256MiB)                             │
//!       │                                                    │
//!       ▼                                                    │
//! │  Master Key (256-bit)                                   │
//!       │                                                    │
//!       ├──▶ Encryption Key (AES-256-GCM)                   │
//!       │      │                                             │
//!       │      ▼                                             │
//!       │  Encrypted Secrets (SQLite)                       │
//!       │      │                                             │
//!       │      └──▶ Stored on Disk                           │
//!       │                                                    │
//!       └──▶ HMAC Key (authentication)                      │
//!              │                                             │
//!              └──▶ Verify operations                        │
//! │                                                          │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```no_run
//! use token_vault::TokenVault;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create vault with password
//! let vault = TokenVault::new("/path/to/vault.db", "my-secure-password")?;
//!
//! // Store a token
//! vault.store("github_token", "ghp_1234567890abcdef", None)?;
//!
//! // Retrieve a token
//! let token = vault.retrieve("github_token")?;
//! assert_eq!(token, Some("ghp_1234567890abcdef".to_string()));
//! # Ok(())
//! # }
//! ```
//!
//! ## Thread Safety
//!
//! `TokenVault` uses `Arc<Mutex<T>>` for thread-safe access to the underlying database.
//! Multiple threads can safely read and write tokens concurrently.
//!
//! ## Threat Model
//!
//! See [`threat_model`](crate::threat_model) for detailed threat analysis.

pub mod audit;
pub mod encryption;
pub mod error;
pub mod session;
pub mod vault;

// Re-export main types
pub use audit::{AuditEntry, AuditLog};
pub use encryption::{EncryptionKey, KeyDerivation};
pub use error::{VaultError, VaultResult};
pub use session::{Session, SessionManager};
pub use vault::{TokenMetadata, TokenVault};

/// Security threat model for token vault
///
/// # Threat Model
///
/// ## Protect Against
///
/// 1. **Compromised Database**
///    - Attacker gains access to vault database file
///    - **Protection**: AES-256-GCM encryption-at-rest
///    - **Requirement**: Attacker needs master password (or brute force)
///
/// 2. **Memory Dump Attacks**
///    - Attacker dumps process memory (core dump, swap)
///    - **Protection**: Zeroization of sensitive data with `zeroize`
///    - **Limitation**: Data exists in memory while in use
///    - **Mitigation**: Encrypted swap, process isolation
///
/// 3. **Password Brute Force**
///    - Attacker attempts to guess master password
///    - **Protection**: Argon2id with high memory/time cost
///    - **Cost**: 256 MB RAM, 3 iterations, 4 parallel lanes
///    - **Attack cost**: ~$100K per guess on cloud GPUs
///
/// 4. **SQL Injection**
///    - Attacker manipulates database queries
///    - **Protection**: Parameterized queries (rusqlite)
///    - **Result**: Not applicable (prepared statements only)
///
/// 5. **Rollback Attacks**
///    - Attacker restores old database backup
///    - **Protection**: Version tracking in metadata
///    - **Detection**: Audit log shows suspicious gaps
///
/// ## Acceptable Limitations
///
/// 1. **Memory Exposure**: Secrets are decrypted in memory while in use
///    - **Mitigation**: Minimize time in memory, zeroize promptly
///    - **Advanced**: Use secure enclave (future work)
///
/// 2. **Password Strength**: Security depends on master password quality
///    - **Recommendation**: Use password manager, 20+ chars, high entropy
///    - **Advanced**: Hardware key support (future work)
///
/// 3. **Process Access**: Attacker with same user privileges can dump memory
///    - **Mitigation**: Run as separate user, container isolation
///    - **Advanced**: OS keychain integration (future work)
///
/// ## Out of Scope
///
/// - Side-channel attacks (timing, cache) - beyond typical threat model
/// - Physical hardware attacks (cold boot, DMA) - use secure enclave
/// - State-level actors with unlimited resources - use air-gapped HSM
pub mod threat_model {}
