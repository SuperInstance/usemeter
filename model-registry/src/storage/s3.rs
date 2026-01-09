//! S3 storage backend

use async_trait::async_trait;
use std::path::PathBuf;
use tracing::{debug, info};

use crate::error::{RegistryError, Result};
use crate::storage::Storage;
use crate::types::ModelMetadata;

/// S3 storage configuration
#[derive(Debug, Clone)]
pub struct S3StorageConfig {
    /// AWS region
    pub region: String,
    /// Bucket name
    pub bucket: String,
    /// Key prefix (e.g., "models/")
    pub prefix: Option<String>,
    /// AWS credentials (optional, can use environment)
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    /// Endpoint URL (for S3-compatible services like MinIO)
    pub endpoint_url: Option<String>,
}

/// S3 storage backend
pub struct S3Storage {
    config: S3StorageConfig,
    #[allow(dead_code)]
    client: aws_sdk_s3::Client,
}

impl S3Storage {
    /// Create new S3 storage
    pub async fn new(config: S3StorageConfig) -> Result<Self> {
        // Load AWS config
        let mut loader = aws_config::defaults(aws_config::Behavior::latest());

        if let Some(endpoint) = &config.endpoint_url {
            loader = loader.endpoint_url(endpoint);
        }

        let aws_config = loader.load().await;

        // Configure S3 client
        let mut s3_config = aws_sdk_s3::config::Builder::from(&aws_config);

        if let Some(key_id) = &config.access_key_id {
            if let Some(secret) = &config.secret_access_key {
                s3_config = s3_config.credentials_provider(aws_sdk_s3::config::Credentials::new(
                    key_id,
                    secret,
                    None,
                    None,
                    "model-registry",
                ));
            }
        }

        let client = aws_sdk_s3::Client::from_conf(s3_config.build());

        info!(
            "S3 storage initialized: bucket={}, region={}",
            config.bucket, config.region
        );

        Ok(Self { config, client })
    }

    /// Get S3 key for a model
    fn s3_key(&self, metadata: &ModelMetadata) -> String {
        let base = match &self.config.prefix {
            Some(prefix) => format!("{}/", prefix.trim_end_matches('/')),
            None => String::new(),
        };

        format!(
            "{}{}/{}/{}/model.bin",
            base, metadata.id, metadata.version, metadata.quantization
        )
    }
}

#[async_trait]
impl Storage for S3Storage {
    async fn store(&self, metadata: &ModelMetadata, source: PathBuf) -> Result<PathBuf> {
        let key = self.s3_key(metadata);

        // Read file
        let body = tokio::fs::read(&source)
            .await
            .map_err(|e| RegistryError::StorageError(format!("Failed to read file: {}", e)))?;

        // Upload to S3
        self.client
            .put_object()
            .bucket(&self.config.bucket)
            .key(&key)
            .body(body.into())
            .send()
            .await
            .map_err(|e| RegistryError::StorageError(format!("Failed to upload to S3: {}", e)))?;

        debug!(
            "Stored model {}@{} in S3: {}",
            metadata.id, metadata.version, key
        );

        Ok(PathBuf::from(format!(
            "s3://{}/{}",
            self.config.bucket, key
        )))
    }

    async fn retrieve(&self, metadata: &ModelMetadata) -> Result<PathBuf> {
        let key = self.s3_key(metadata);

        // Check if object exists
        self.client
            .head_object()
            .bucket(&self.config.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| {
                if e.to_string().contains("NotFound") {
                    RegistryError::ModelNotFound(format!("{}@{}", metadata.id, metadata.version))
                } else {
                    RegistryError::StorageError(format!("Failed to check S3 object: {}", e))
                }
            })?;

        Ok(PathBuf::from(format!(
            "s3://{}/{}",
            self.config.bucket, key
        )))
    }

    async fn exists(&self, metadata: &ModelMetadata) -> bool {
        let key = self.s3_key(metadata);

        match self
            .client
            .head_object()
            .bucket(&self.config.bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(_) => true,
            Err(e) => {
                debug!("S3 object check failed: {}", e);
                false
            },
        }
    }

    async fn delete(&self, metadata: &ModelMetadata) -> Result<()> {
        let key = self.s3_key(metadata);

        self.client
            .delete_object()
            .bucket(&self.config.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| {
                RegistryError::StorageError(format!("Failed to delete S3 object: {}", e))
            })?;

        info!(
            "Deleted model {}@{} from S3: {}",
            metadata.id, metadata.version, key
        );

        Ok(())
    }

    async fn size(&self, metadata: &ModelMetadata) -> Result<u64> {
        let key = self.s3_key(metadata);

        let response = self
            .client
            .head_object()
            .bucket(&self.config.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| {
                RegistryError::StorageError(format!("Failed to get S3 object size: {}", e))
            })?;

        Ok(response.content_length().unwrap_or(0) as u64)
    }

    async fn list(&self) -> Result<Vec<String>> {
        let prefix = self.config.prefix.clone().unwrap_or_default();

        let response = self
            .client
            .list_objects_v2()
            .bucket(&self.config.bucket)
            .prefix(&prefix)
            .send()
            .await
            .map_err(|e| {
                RegistryError::StorageError(format!("Failed to list S3 objects: {}", e))
            })?;

        let mut models = Vec::new();
        if let Some(objects) = response.contents() {
            for obj in objects {
                if let Some(key) = obj.key() {
                    // Extract model ID from key
                    // Format: {prefix}{model_id}/{version}/{quant}/model.bin
                    let relative = key.strip_prefix(&prefix).unwrap_or(key);
                    if let Some(model_id) = relative.split('/').next() {
                        if !model_id.is_empty() && !models.contains(&model_id.to_string()) {
                            models.push(model_id.to_string());
                        }
                    }
                }
            }
        }

        Ok(models)
    }

    async fn available_space(&self) -> Result<u64> {
        // S3 has unlimited storage from the client's perspective
        Ok(u64::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s3_key_generation() {
        let config = S3StorageConfig {
            region: "us-east-1".to_string(),
            bucket: "test-bucket".to_string(),
            prefix: Some("models/".to_string()),
            access_key_id: None,
            secret_access_key: None,
            endpoint_url: None,
        };

        let storage = S3Storage {
            config,
            client: aws_sdk_s3::Client::from_conf(aws_sdk_s3::config::Builder::new().build()),
        };

        let metadata = crate::types::ModelMetadata {
            id: "test-model".to_string(),
            version: "1.0.0".to_string(),
            name: "Test".to_string(),
            family: "test".to_string(),
            parameters: "7B".to_string(),
            quantization: crate::types::Quantization::Q4,
            size_bytes: 1024,
            sha256: None,
            source: crate::types::ModelSource::Url {
                url: "https://example.com/model.gguf".to_string(),
                filename: "model.gguf".to_string(),
            },
            license: "MIT".to_string(),
            description: "Test".to_string(),
            tags: vec![],
            context_length: 4096,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            custom: Default::default(),
        };

        let key = storage.s3_key(&metadata);
        assert_eq!(key, "models/test-model/1.0.0/q4_k_m/model.bin");
    }
}
