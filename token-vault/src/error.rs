//! Error types for token vault


/// Result type for vault operations
pub type VaultResult<T> = Result<T, VaultError>;

/// Errors that can occur in vault operations
#[derive(thiserror::Error, Debug)]
pub enum VaultError {
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// Encryption error
    #[error("Encryption error: {0}")]
    Encryption(String),

    /// Key derivation error
    #[error("Key derivation error: {0}")]
    KeyDerivation(String),

    /// Token not found
    #[error("Token not found: {0}")]
    TokenNotFound(String),

    /// Token already exists
    #[error("Token already exists: {0}")]
    TokenAlreadyExists(String),

    /// Invalid token name
    #[error("Invalid token name: {0}")]
    InvalidTokenName(String),

    /// Session not found
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    /// Authentication failed
    #[error("Authentication failed")]
    AuthenticationFailed,

    /// Access denied
    #[error("Access denied: {0}")]
    AccessDenied(String),

    /// Invalid password
    #[error("Invalid password")]
    InvalidPassword,

    /// Vault is locked
    #[error("Vault is locked")]
    VaultLocked,

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// UTF8 conversion error
    #[error("UTF8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// Date/time parsing error
    #[error("Date/time parsing error: {0}")]
    DateParse(#[from] chrono::ParseError),
}

impl serde::Serialize for VaultError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = VaultError::TokenNotFound("test_token".to_string());
        assert_eq!(err.to_string(), "Token not found: test_token");
    }

    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let vault_err: VaultError = io_err.into();
        assert!(matches!(vault_err, VaultError::Io(_)));
    }
}
