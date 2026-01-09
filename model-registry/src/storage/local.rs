//! Local filesystem storage backend

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info};

use crate::error::{RegistryError, Result};
use crate::storage::{Storage, StorageConfig};
use crate::types::ModelMetadata;

/// Local filesystem storage
pub struct LocalStorage {
    base_dir: PathBuf,
}

impl LocalStorage {
    /// Create new local storage
    pub async fn new(config: super::LocalStorageConfig) -> Result<Self> {
        let base_dir = config.base_dir.clone();

        if config.create_if_missing {
            fs::create_dir_all(&base_dir).await.map_err(|e| {
                RegistryError::StorageError(format!("Failed to create base directory: {}", e))
            })?;
        }

        info!("Local storage initialized at: {}", base_dir.display());

        Ok(Self { base_dir })
    }

    /// Get the path for a model
    fn model_path(&self, metadata: &ModelMetadata) -> PathBuf {
        self.base_dir
            .join(&metadata.id)
            .join(&metadata.version)
            .join(&metadata.quantization.to_string())
    }

    /// Get the file path for a model
    fn file_path(&self, metadata: &ModelMetadata) -> PathBuf {
        let filename = match &metadata.source {
            crate::types::ModelSource::HuggingFace { filename, .. } => filename.clone(),
            crate::types::ModelSource::Url { filename, .. } => filename.clone(),
            crate::types::ModelSource::Local { path } => path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "model.bin".to_string()),
        };

        self.model_path(metadata).join(filename)
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn store(&self, metadata: &ModelMetadata, source: PathBuf) -> Result<PathBuf> {
        let dest_path = self.file_path(metadata);

        // Create parent directories
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                RegistryError::StorageError(format!("Failed to create directory: {}", e))
            })?;
        }

        // Copy file
        fs::copy(&source, &dest_path)
            .await
            .map_err(|e| RegistryError::StorageError(format!("Failed to copy file: {}", e)))?;

        debug!(
            "Stored model {}@{} at {}",
            metadata.id,
            metadata.version,
            dest_path.display()
        );

        Ok(dest_path)
    }

    async fn retrieve(&self, metadata: &ModelMetadata) -> Result<PathBuf> {
        let path = self.file_path(metadata);

        if !path.exists() {
            return Err(RegistryError::ModelNotFound(format!(
                "{}@{}",
                metadata.id, metadata.version
            )));
        }

        Ok(path)
    }

    async fn exists(&self, metadata: &ModelMetadata) -> bool {
        self.file_path(metadata).exists()
    }

    async fn delete(&self, metadata: &ModelMetadata) -> Result<()> {
        let path = self.file_path(metadata);

        if !path.exists() {
            return Ok(()); // Already deleted
        }

        fs::remove_file(&path)
            .await
            .map_err(|e| RegistryError::StorageError(format!("Failed to delete file: {}", e)))?;

        info!(
            "Deleted model {}@{} at {}",
            metadata.id,
            metadata.version,
            path.display()
        );

        Ok(())
    }

    async fn size(&self, metadata: &ModelMetadata) -> Result<u64> {
        let path = self.file_path(metadata);

        let metadata = fs::metadata(&path)
            .await
            .map_err(|e| RegistryError::StorageError(format!("Failed to get file size: {}", e)))?;

        Ok(metadata.len())
    }

    async fn list(&self) -> Result<Vec<String>> {
        let mut models = Vec::new();

        if !self.base_dir.exists() {
            return Ok(models);
        }

        let mut entries = fs::read_dir(&self.base_dir).await.map_err(|e| {
            RegistryError::StorageError(format!("Failed to read base directory: {}", e))
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            RegistryError::StorageError(format!("Failed to read directory entry: {}", e))
        })? {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    models.push(name.to_string());
                }
            }
        }

        Ok(models)
    }

    async fn available_space(&self) -> Result<u64> {
        // Use sysinfo to get available disk space
        // For now, return a dummy value
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = fs::metadata(&self.base_dir).await.map_err(|e| {
                RegistryError::StorageError(format!("Failed to get disk space: {}", e))
            })?;

            // This is a simplified version
            // In production, you'd use sysinfo or similar
            Ok(u64::MAX)
        }

        #[cfg(not(unix))]
        {
            Ok(u64::MAX)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ModelMetadata, ModelSource, Quantization};
    use tempfile::TempDir;

    fn create_test_metadata() -> ModelMetadata {
        ModelMetadata {
            id: "test-model".to_string(),
            version: "1.0.0".to_string(),
            name: "Test Model".to_string(),
            family: "test".to_string(),
            parameters: "7B".to_string(),
            quantization: Quantization::Q4,
            size_bytes: 1024 * 1024 * 1024,
            sha256: None,
            source: ModelSource::Url {
                url: "https://example.com/model.gguf".to_string(),
                filename: "model.gguf".to_string(),
            },
            license: "MIT".to_string(),
            description: "Test model".to_string(),
            tags: vec![],
            context_length: 4096,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            custom: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_local_storage_init() {
        let temp_dir = TempDir::new().unwrap();
        let config = LocalStorageConfig {
            base_dir: temp_dir.path().to_path_buf(),
            create_if_missing: true,
        };

        let storage = LocalStorage::new(config).await.unwrap();
        assert!(storage.base_dir.exists());
    }

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let temp_dir = TempDir::new().unwrap();
        let config = LocalStorageConfig {
            base_dir: temp_dir.path().to_path_buf(),
            create_if_missing: true,
        };

        let storage = LocalStorage::new(config).await.unwrap();

        // Create a test file
        let test_file = temp_dir.path().join("test.bin");
        fs::write(&test_file, b"test data").await.unwrap();

        let metadata = create_test_metadata();

        // Store
        let stored_path = storage.store(&metadata, test_file).await.unwrap();
        assert!(stored_path.exists());

        // Retrieve
        let retrieved_path = storage.retrieve(&metadata).await.unwrap();
        assert_eq!(retrieved_path, stored_path);
    }

    #[tokio::test]
    async fn test_exists_and_delete() {
        let temp_dir = TempDir::new().unwrap();
        let config = LocalStorageConfig {
            base_dir: temp_dir.path().to_path_buf(),
            create_if_missing: true,
        };

        let storage = LocalStorage::new(config).await.unwrap();

        let test_file = temp_dir.path().join("test.bin");
        fs::write(&test_file, b"test data").await.unwrap();

        let metadata = create_test_metadata();

        // Store
        storage.store(&metadata, test_file).await.unwrap();

        // Check exists
        assert!(storage.exists(&metadata).await);

        // Delete
        storage.delete(&metadata).await.unwrap();
        assert!(!storage.exists(&metadata).await);
    }
}
