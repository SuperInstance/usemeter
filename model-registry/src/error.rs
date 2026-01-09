//! Error types for model-registry

use std::path::PathBuf;
use thiserror::Error;

/// Result type for model registry operations
pub type Result<T> = std::result::Result<T, RegistryError>;

/// Model registry errors
#[derive(Debug, Error)]
pub enum RegistryError {
    /// Model not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Model version not found
    #[error("Version {version} not found for model {model}")]
    VersionNotFound { model: String, version: String },

    /// Invalid model version
    #[error("Invalid version: {0}")]
    InvalidVersion(String),

    /// Download failed
    #[error("Download failed: {0}")]
    DownloadFailed(String),

    /// Checksum mismatch
    #[error("Checksum mismatch for {model}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        model: String,
        expected: String,
        actual: String,
    },

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// HTTP error
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Invalid path
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Invalid metadata
    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),

    /// Already exists
    #[error("Model {model} version {version} already exists")]
    AlreadyExists { model: String, version: String },

    /// Insufficient disk space
    #[error("Insufficient disk space: needed {needed} bytes, available {available} bytes")]
    InsufficientDiskSpace { needed: u64, available: u64 },

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl RegistryError {
    /// Create a storage error
    pub fn storage<S: Into<String>>(msg: S) -> Self {
        Self::StorageError(msg.into())
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }
}
