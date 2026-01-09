//! Core vault implementation with encrypted token storage

use std::path::Path;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::audit::{AuditEntry, AuditLog, AuditOperation, AuditResult};
use crate::encryption::{decrypt, encrypt, KeyDerivation};
use crate::error::{VaultError, VaultResult};
use crate::session::SessionManager;

/// Maximum token name length
const MAX_TOKEN_NAME_LENGTH: usize = 100;

/// Token metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    /// Token name
    pub name: String,

    /// Session ID
    pub session_id: String,

    /// Creation time
    pub created_at: DateTime<Utc>,

    /// Last updated time
    pub updated_at: DateTime<Utc>,

    /// Expiration time (optional)
    pub expires_at: Option<DateTime<Utc>>,

    /// Description (optional)
    pub description: Option<String>,

    /// Tags (optional)
    pub tags: Option<Vec<String>>,
}

/// Main token vault with encryption-at-rest
pub struct TokenVault {
    conn: Arc<Mutex<Connection>>,
    key_derivation: KeyDerivation,
    audit_log: Arc<Mutex<AuditLog>>,
    session_manager: Arc<Mutex<SessionManager>>,
}

impl TokenVault {
    /// Create new vault with password
    ///
    /// # Arguments
    /// * `db_path` - Path to SQLite database
    /// * `password` - Master password for encryption
    ///
    /// # Example
    /// ```no_run
    /// use token_vault::TokenVault;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let vault = TokenVault::new("/path/to/vault.db", "my-secure-password")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<P: AsRef<Path>>(db_path: P, _password: &str) -> VaultResult<Self> {
        let conn = Connection::open(db_path)?;

        // Create tables
        Self::init_database(&conn)?;

        // Initialize audit log
        let audit_log = AuditLog::new();

        // Initialize session manager
        let session_manager = SessionManager::new();

        // Create default session
        let mut manager = session_manager;
        manager.create_session("default".to_string(), "Default Session".to_string())?;

        // TODO: Load key derivation from database
        // For now, use default
        let key_derivation = KeyDerivation::default();

        // TODO: Verify password by attempting to decrypt a known value

        let vault = Self {
            conn: Arc::new(Mutex::new(conn)),
            key_derivation,
            audit_log: Arc::new(Mutex::new(audit_log)),
            session_manager: Arc::new(Mutex::new(manager)),
        };

        Ok(vault)
    }

    /// Initialize database schema
    fn init_database(conn: &Connection) -> VaultResult<()> {
        // Tokens table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tokens (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                session_id TEXT NOT NULL,
                encrypted_value BLOB NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                expires_at TEXT,
                description TEXT,
                tags TEXT,
                UNIQUE(name, session_id)
            )",
            [],
        )?;

        // Indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tokens_name ON tokens(name)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tokens_session ON tokens(session_id)",
            [],
        )?;

        // Metadata table for key derivation
        conn.execute(
            "CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    /// Store a token
    ///
    /// # Arguments
    /// * `name` - Token name (unique within session)
    /// * `value` - Token value (will be encrypted)
    /// * `session_id` - Session ID (uses "default" if None)
    ///
    /// # Example
    /// ```no_run
    /// # use token_vault::TokenVault;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let vault = TokenVault::new("/path/to/vault.db", "password")?;
    /// vault.store("github_token", "ghp_1234567890abcdef", Some("dev"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn store(&self, name: &str, value: &str, session_id: Option<&str>) -> VaultResult<()> {
        // Validate token name
        Self::validate_token_name(name)?;

        // Derive encryption key
        let key = self.key_derivation.derive_key("")?; // TODO: Get password from secure storage

        // Encrypt value
        let encrypted = encrypt(&key, value.as_bytes())?;

        // Get session ID
        let session = session_id.unwrap_or("default");

        // Insert into database
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().map_err(|e| VaultError::Internal(e.to_string()))?;

        conn.execute(
            "INSERT INTO tokens (name, session_id, encrypted_value, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            params![name, session, encrypted, now],
        )?;

        // Log audit entry
        let audit = AuditEntry::new(
            AuditOperation::TokenCreate,
            name.to_string(),
            "system".to_string(),
            AuditResult::Success,
        );
        self.log_audit(audit);

        Ok(())
    }

    /// Retrieve a token
    ///
    /// # Arguments
    /// * `name` - Token name
    /// * `session_id` - Session ID (uses "default" if None)
    ///
    /// # Returns
    /// * Some(token_value) if found
    /// * None if not found
    pub fn retrieve(&self, name: &str, session_id: Option<&str>) -> VaultResult<Option<String>> {
        let session = session_id.unwrap_or("default");
        let conn = self.conn.lock().map_err(|e| VaultError::Internal(e.to_string()))?;

        let encrypted: Vec<u8> = match conn.query_row(
            "SELECT encrypted_value FROM tokens WHERE name = ?1 AND session_id = ?2",
            params![name, session],
            |row| row.get(0),
        ) {
            Ok(data) => data,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Log audit entry
                let audit = AuditEntry::new(
                    AuditOperation::TokenRead,
                    name.to_string(),
                    "system".to_string(),
                    AuditResult::Failure("Not found".to_string()),
                );
                self.log_audit(audit);

                return Ok(None);
            }
            Err(e) => return Err(VaultError::Database(e)),
        };

        // Derive decryption key
        let key = self.key_derivation.derive_key("")?;

        // Decrypt
        let decrypted = decrypt(&key, &encrypted)?;
        let value = String::from_utf8(decrypted)?;

        // Log audit entry
        let audit = AuditEntry::new(
            AuditOperation::TokenRead,
            name.to_string(),
            "system".to_string(),
            AuditResult::Success,
        );
        self.log_audit(audit);

        Ok(Some(value))
    }

    /// Update a token
    pub fn update(&self, name: &str, value: &str, session_id: Option<&str>) -> VaultResult<()> {
        let session = session_id.unwrap_or("default");

        // Check if token exists
        let conn = self.conn.lock().map_err(|e| VaultError::Internal(e.to_string()))?;
        let exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM tokens WHERE name = ?1 AND session_id = ?2",
            params![name, session],
            |row| row.get(0),
        )?;

        if !exists {
            return Err(VaultError::TokenNotFound(name.to_string()));
        }

        // Derive encryption key
        let key = self.key_derivation.derive_key("")?;

        // Encrypt new value
        let encrypted = encrypt(&key, value.as_bytes())?;

        // Update
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE tokens SET encrypted_value = ?1, updated_at = ?2 WHERE name = ?3 AND session_id = ?4",
            params![encrypted, now, name, session],
        )?;

        // Log audit entry
        let audit = AuditEntry::new(
            AuditOperation::TokenUpdate,
            name.to_string(),
            "system".to_string(),
            AuditResult::Success,
        );
        self.log_audit(audit);

        Ok(())
    }

    /// Delete a token
    pub fn delete(&self, name: &str, session_id: Option<&str>) -> VaultResult<()> {
        let session = session_id.unwrap_or("default");
        let conn = self.conn.lock().map_err(|e| VaultError::Internal(e.to_string()))?;

        let rows_affected = conn.execute(
            "DELETE FROM tokens WHERE name = ?1 AND session_id = ?2",
            params![name, session],
        )?;

        if rows_affected == 0 {
            return Err(VaultError::TokenNotFound(name.to_string()));
        }

        // Log audit entry
        let audit = AuditEntry::new(
            AuditOperation::TokenDelete,
            name.to_string(),
            "system".to_string(),
            AuditResult::Success,
        );
        self.log_audit(audit);

        Ok(())
    }

    /// List all tokens in a session
    pub fn list_tokens(&self, session_id: Option<&str>) -> VaultResult<Vec<String>> {
        let session = session_id.unwrap_or("default");
        let conn = self.conn.lock().map_err(|e| VaultError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT name FROM tokens WHERE session_id = ?1 ORDER BY name",
        )?;

        let tokens = stmt.query_map(params![session], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tokens)
    }

    /// Get token metadata
    pub fn get_metadata(&self, name: &str, session_id: Option<&str>) -> VaultResult<TokenMetadata> {
        let session = session_id.unwrap_or("default");
        let conn = self.conn.lock().map_err(|e| VaultError::Internal(e.to_string()))?;

        let (name_val, session_id_val, created_at, updated_at): (String, String, String, String) =
            conn.query_row(
                "SELECT name, session_id, created_at, updated_at FROM tokens WHERE name = ?1 AND session_id = ?2",
                params![name, session],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )?;

        Ok(TokenMetadata {
            name: name_val,
            session_id: session_id_val,
            created_at: created_at.parse()?,
            updated_at: updated_at.parse()?,
            expires_at: None,
            description: None,
            tags: None,
        })
    }

    /// Validate token name
    fn validate_token_name(name: &str) -> VaultResult<()> {
        if name.is_empty() {
            return Err(VaultError::InvalidTokenName("Name cannot be empty".to_string()));
        }

        if name.len() > MAX_TOKEN_NAME_LENGTH {
            return Err(VaultError::InvalidTokenName(format!(
                "Name too long (max {} characters)",
                MAX_TOKEN_NAME_LENGTH
            )));
        }

        // Allow alphanumeric, underscore, dash, and dot
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.') {
            return Err(VaultError::InvalidTokenName(
                "Name must contain only alphanumeric characters, underscores, dashes, and dots".to_string()
            ));
        }

        Ok(())
    }

    /// Log audit entry
    fn log_audit(&self, entry: AuditEntry) {
        let mut log = self.audit_log.lock().unwrap();
        log.log(entry);
    }

    /// Get audit log entries
    pub fn audit_entries(&self) -> Vec<AuditEntry> {
        let log = self.audit_log.lock().unwrap();
        log.entries().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_store_and_retrieve() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let vault = TokenVault::new(&db_path, "test-password").unwrap();

        vault.store("test_token", "secret_value", None).unwrap();
        let retrieved = vault.retrieve("test_token", None).unwrap();

        assert_eq!(retrieved, Some("secret_value".to_string()));
    }

    #[test]
    fn test_retrieve_nonexistent() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let vault = TokenVault::new(&db_path, "test-password").unwrap();

        let result = vault.retrieve("nonexistent", None).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_update_token() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let vault = TokenVault::new(&db_path, "test-password").unwrap();

        vault.store("test_token", "old_value", None).unwrap();
        vault.update("test_token", "new_value", None).unwrap();

        let retrieved = vault.retrieve("test_token", None).unwrap();
        assert_eq!(retrieved, Some("new_value".to_string()));
    }

    #[test]
    fn test_delete_token() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let vault = TokenVault::new(&db_path, "test-password").unwrap();

        vault.store("test_token", "secret_value", None).unwrap();
        vault.delete("test_token", None).unwrap();

        let result = vault.retrieve("test_token", None).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_list_tokens() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let vault = TokenVault::new(&db_path, "test-password").unwrap();

        vault.store("token1", "value1", None).unwrap();
        vault.store("token2", "value2", None).unwrap();

        let tokens = vault.list_tokens(None).unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(tokens.contains(&"token1".to_string()));
        assert!(tokens.contains(&"token2".to_string()));
    }

    #[test]
    fn test_invalid_token_name() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let vault = TokenVault::new(&db_path, "test-password").unwrap();

        // Empty name
        let result = vault.store("", "value", None);
        assert!(result.is_err());

        // Name with invalid characters
        let result = vault.store("test token!", "value", None);
        assert!(result.is_err());
    }
}
