//! Model registry

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

use crate::download::{DownloadConfig, Downloader, ProgressCallback};
use crate::error::Result;
use crate::storage::{Storage, StorageConfig};
use crate::types::{DownloadProgress, ModelMetadata, ModelSource, ModelStatus, Version};

/// Model registry
///
/// Manages model versions, downloads, and storage.
pub struct Registry {
    storage: Arc<dyn Storage>,
    downloader: Downloader,
    models: Arc<RwLock<HashMap<String, Vec<ModelMetadata>>>>,
}

impl Registry {
    /// Create new registry
    pub async fn new(storage_config: StorageConfig) -> Result<Self> {
        let storage = crate::storage::create_storage(storage_config).await?;

        let downloader = Downloader::new(DownloadConfig::default())?;

        Ok(Self {
            storage,
            downloader,
            models: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create with custom download config
    pub async fn with_config(
        storage_config: StorageConfig,
        download_config: DownloadConfig,
    ) -> Result<Self> {
        let storage = crate::storage::create_storage(storage_config).await?;
        let downloader = Downloader::new(download_config)?;

        Ok(Self {
            storage,
            downloader,
            models: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Register a model
    ///
    /// Adds model metadata to the registry.
    #[instrument(skip(self))]
    pub async fn register(&self, metadata: ModelMetadata) -> Result<()> {
        metadata.validate()?;

        let mut models = self.models.write().await;

        let versions = models.entry(metadata.id.clone()).or_default();

        // Check if version already exists
        if versions.iter().any(|v| v.version == metadata.version) {
            return Err(crate::error::RegistryError::AlreadyExists {
                model: metadata.id.clone(),
                version: metadata.version.clone(),
            });
        }

        versions.push(metadata);

        info!("Registered model {}@{}", metadata.id, metadata.version);

        Ok(())
    }

    /// Get all models
    pub async fn list(&self) -> Result<Vec<ModelMetadata>> {
        let models = self.models.read().await;
        let mut all_models = Vec::new();

        for versions in models.values() {
            all_models.extend(versions.clone());
        }

        Ok(all_models)
    }

    /// Get all versions of a model
    pub async fn get(&self, id: &str) -> Result<Vec<ModelMetadata>> {
        let models = self.models.read().await;

        match models.get(id) {
            Some(versions) => Ok(versions.clone()),
            None => Err(crate::error::RegistryError::ModelNotFound(id.to_string())),
        }
    }

    /// Get specific version of a model
    pub async fn get_version(&self, id: &str, version: &str) -> Result<ModelMetadata> {
        let models = self.models.read().await;

        match models.get(id) {
            Some(versions) => versions
                .iter()
                .find(|v| v.version == version)
                .cloned()
                .ok_or_else(|| crate::error::RegistryError::VersionNotFound {
                    model: id.to_string(),
                    version: version.to_string(),
                }),
            None => Err(crate::error::RegistryError::ModelNotFound(id.to_string())),
        }
    }

    /// Get latest version of a model
    pub async fn get_latest(&self, id: &str) -> Result<ModelMetadata> {
        let versions = self.get(id).await?;

        versions
            .into_iter()
            .max_by_key(|v| v.version.clone())
            .ok_or_else(|| crate::error::RegistryError::ModelNotFound(id.to_string()))
    }

    /// Download a model
    ///
    /// Downloads model to local storage.
    #[instrument(skip(self, progress_callback))]
    pub async fn download(
        &self,
        id: &str,
        version: &str,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<PathBuf> {
        let metadata = self.get_version(id, version).await?;

        // Check if already downloaded
        if self.storage.exists(&metadata).await {
            info!("Model {}@{} already downloaded", id, version);
            return self.storage.retrieve(&metadata).await;
        }

        // Check available space
        let available = self.storage.available_space().await?;
        if available < metadata.size_bytes {
            return Err(crate::error::RegistryError::InsufficientDiskSpace {
                needed: metadata.size_bytes,
                available,
            });
        }

        // Download to temp file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("{}-{}-download", id, version));

        self.downloader
            .download(&metadata, &temp_file, progress_callback)
            .await?;

        // Store in backend
        let path = self.storage.store(&metadata, temp_file).await?;

        // Clean up temp file
        let _ = tokio::fs::remove_file(temp_file).await;

        Ok(path)
    }

    /// Delete a model
    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str, version: &str) -> Result<()> {
        let metadata = self.get_version(id, version).await?;
        self.storage.delete(&metadata).await?;

        info!("Deleted model {}@{}", id, version);
        Ok(())
    }

    /// Check if model is downloaded
    pub async fn is_downloaded(&self, id: &str, version: &str) -> bool {
        match self.get_version(id, version).await {
            Ok(metadata) => self.storage.exists(&metadata).await,
            Err(_) => false,
        }
    }

    /// Get model size
    pub async fn get_size(&self, id: &str, version: &str) -> Result<u64> {
        let metadata = self.get_version(id, version).await?;
        self.storage.size(&metadata).await
    }

    /// Search models by tag
    pub async fn search_by_tag(&self, tag: &str) -> Result<Vec<ModelMetadata>> {
        let models = self.models.read().await;
        let mut results = Vec::new();

        for versions in models.values() {
            for metadata in versions {
                if metadata.tags.iter().any(|t| t == tag) {
                    results.push(metadata.clone());
                }
            }
        }

        Ok(results)
    }

    /// Search models by family
    pub async fn search_by_family(&self, family: &str) -> Result<Vec<ModelMetadata>> {
        let models = self.models.read().await;
        let mut results = Vec::new();

        for versions in models.values() {
            for metadata in versions {
                if metadata.family == family {
                    results.push(metadata.clone());
                }
            }
        }

        Ok(results)
    }

    /// Import models from JSON
    pub async fn import_json(&self, json: &str) -> Result<()> {
        let metadatas: Vec<ModelMetadata> = serde_json::from_str(json)?;

        for metadata in metadatas {
            self.register(metadata).await?;
        }

        Ok(())
    }

    /// Export models to JSON
    pub async fn export_json(&self) -> Result<String> {
        let models = self.list().await?;
        let json = serde_json::to_string_pretty(&models)?;

        Ok(json)
    }
}

/// Registry builder
pub struct RegistryBuilder {
    storage_config: Option<StorageConfig>,
    download_config: Option<DownloadConfig>,
}

impl Default for RegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistryBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            storage_config: None,
            download_config: None,
        }
    }

    /// Set storage configuration
    pub fn storage(mut self, config: StorageConfig) -> Self {
        self.storage_config = Some(config);
        self
    }

    /// Set download configuration
    pub fn download_config(mut self, config: DownloadConfig) -> Self {
        self.download_config = Some(config);
        self
    }

    /// Build the registry
    pub async fn build(self) -> Result<Registry> {
        let storage_config = self.storage_config.unwrap_or(StorageConfig::Local(
            crate::storage::LocalStorageConfig::default(),
        ));

        match self.download_config {
            Some(config) => Registry::with_config(storage_config, config).await,
            None => Registry::new(storage_config).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ModelMetadata, ModelSource, Quantization};
    use tempfile::TempDir;

    fn create_test_metadata(id: &str, version: &str) -> ModelMetadata {
        ModelMetadata {
            id: id.to_string(),
            version: version.to_string(),
            name: format!("Test Model {}", id),
            family: "test".to_string(),
            parameters: "7B".to_string(),
            quantization: Quantization::Q4,
            size_bytes: 1024 * 1024,
            sha256: None,
            source: ModelSource::Url {
                url: format!("https://example.com/{}.gguf", id),
                filename: format!("{}.gguf", id),
            },
            license: "MIT".to_string(),
            description: "Test model".to_string(),
            tags: vec!["test".to_string()],
            context_length: 4096,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            custom: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_registry_builder() {
        let temp_dir = TempDir::new().unwrap();
        let storage_config = StorageConfig::Local(crate::storage::LocalStorageConfig {
            base_dir: temp_dir.path().to_path_buf(),
            create_if_missing: true,
        });

        let registry = RegistryBuilder::new()
            .storage(storage_config)
            .build()
            .await
            .unwrap();

        assert!(registry.list().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_register_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let storage_config = StorageConfig::Local(crate::storage::LocalStorageConfig {
            base_dir: temp_dir.path().to_path_buf(),
            create_if_missing: true,
        });

        let registry = RegistryBuilder::new()
            .storage(storage_config)
            .build()
            .await
            .unwrap();

        let metadata = create_test_metadata("test-model", "1.0.0");

        registry.register(metadata.clone()).await.unwrap();

        let models = registry.list().await.unwrap();
        assert_eq!(models.len(), 1);

        let retrieved = registry.get_version("test-model", "1.0.0").await.unwrap();
        assert_eq!(retrieved.id, "test-model");
    }

    #[tokio::test]
    async fn test_multiple_versions() {
        let temp_dir = TempDir::new().unwrap();
        let storage_config = StorageConfig::Local(crate::storage::LocalStorageConfig {
            base_dir: temp_dir.path().to_path_buf(),
            create_if_missing: true,
        });

        let registry = RegistryBuilder::new()
            .storage(storage_config)
            .build()
            .await
            .unwrap();

        registry
            .register(create_test_metadata("test-model", "1.0.0"))
            .await
            .unwrap();

        registry
            .register(create_test_metadata("test-model", "2.0.0"))
            .await
            .unwrap();

        let versions = registry.get("test-model").await.unwrap();
        assert_eq!(versions.len(), 2);
    }

    #[tokio::test]
    async fn test_search_by_tag() {
        let temp_dir = TempDir::new().unwrap();
        let storage_config = StorageConfig::Local(crate::storage::LocalStorageConfig {
            base_dir: temp_dir.path().to_path_buf(),
            create_if_missing: true,
        });

        let registry = RegistryBuilder::new()
            .storage(storage_config)
            .build()
            .await
            .unwrap();

        let mut metadata = create_test_metadata("test-model", "1.0.0");
        metadata.tags = vec!["llm".to_string(), "text-generation".to_string()];

        registry.register(metadata).await.unwrap();

        let results = registry.search_by_tag("llm").await.unwrap();
        assert_eq!(results.len(), 1);
    }
}
