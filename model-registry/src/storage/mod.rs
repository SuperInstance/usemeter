//! Storage backends for model registry

use async_trait::async_trait;
use std::path::PathBuf;

use crate::error::Result;
use crate::types::ModelMetadata;

pub mod local;

#[cfg(feature = "s3-storage")]
pub mod s3;

/// Storage backend abstraction
#[async_trait]
pub trait Storage: Send + Sync {
    /// Store a model file
    async fn store(&self, metadata: &ModelMetadata, source: PathBuf) -> Result<PathBuf>;

    /// Retrieve a model file
    async fn retrieve(&self, metadata: &ModelMetadata) -> Result<PathBuf>;

    /// Check if a model exists
    async fn exists(&self, metadata: &ModelMetadata) -> bool;

    /// Delete a model file
    async fn delete(&self, metadata: &ModelMetadata) -> Result<()>;

    /// Get file size
    async fn size(&self, metadata: &ModelMetadata) -> Result<u64>;

    /// List all models
    async fn list(&self) -> Result<Vec<String>>;

    /// Get available disk space
    async fn available_space(&self) -> Result<u64>;
}

/// Storage configuration
#[derive(Debug, Clone)]
pub enum StorageConfig {
    Local(LocalStorageConfig),
    #[cfg(feature = "s3-storage")]
    S3(S3StorageConfig),
}

/// Local storage configuration
#[derive(Debug, Clone)]
pub struct LocalStorageConfig {
    /// Base directory for models
    pub base_dir: PathBuf,
    /// Create directory if it doesn't exist
    pub create_if_missing: bool,
}

impl Default for LocalStorageConfig {
    fn default() -> Self {
        Self {
            base_dir: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".model-registry")
                .join("models"),
            create_if_missing: true,
        }
    }
}

#[cfg(feature = "s3-storage")]
pub use s3::S3StorageConfig;

/// Create storage backend from configuration
pub async fn create_storage(config: StorageConfig) -> Result<Box<dyn Storage>> {
    match config {
        StorageConfig::Local(config) => Ok(Box::new(local::LocalStorage::new(config).await?)),
        #[cfg(feature = "s3-storage")]
        StorageConfig::S3(config) => Ok(Box::new(s3::S3Storage::new(config).await?)),
    }
}
