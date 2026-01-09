//! Core types for model registry

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::Result;

/// Model version string (e.g., "1.0.0", "v2", "latest")
pub type Version = String;

/// Model identifier (e.g., "llama-3.2-8b")
pub type ModelId = String;

/// Quantization level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Quantization {
    /// 4-bit quantization
    Q4,
    /// 5-bit quantization
    Q5,
    /// 8-bit quantization
    Q8,
    /// 16-bit float
    F16,
    /// 32-bit float
    F32,
}

impl Quantization {
    /// Get file suffix for this quantization
    pub fn suffix(&self) -> &'static str {
        match self {
            Quantization::Q4 => "q4_k_m",
            Quantization::Q5 => "q5_k_m",
            Quantization::Q8 => "q8_0",
            Quantization::F16 => "f16",
            Quantization::F32 => "f32",
        }
    }

    /// Approximate size multiplier relative to F32
    pub fn size_multiplier(&self) -> f32 {
        match self {
            Quantization::Q4 => 0.25,
            Quantization::Q5 => 0.31,
            Quantization::Q8 => 0.5,
            Quantization::F16 => 0.5,
            Quantization::F32 => 1.0,
        }
    }
}

impl std::fmt::Display for Quantization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.suffix())
    }
}

impl std::str::FromStr for Quantization {
    type Err = crate::error::RegistryError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "q4" | "q4_k_m" => Ok(Quantization::Q4),
            "q5" | "q5_k_m" => Ok(Quantization::Q5),
            "q8" | "q8_0" => Ok(Quantization::Q8),
            "f16" => Ok(Quantization::F16),
            "f32" => Ok(Quantization::F32),
            _ => Err(crate::error::RegistryError::InvalidVersion(format!(
                "Unknown quantization: {}",
                s
            ))),
        }
    }
}

/// Model source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ModelSource {
    /// HuggingFace Hub
    HuggingFace {
        repo_id: String,
        filename: String,
        revision: Option<String>,
    },
    /// Direct URL
    Url { url: String, filename: String },
    /// Local file
    Local { path: PathBuf },
}

impl std::fmt::Display for ModelSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelSource::HuggingFace {
                repo_id, filename, ..
            } => write!(f, "{}/{}", repo_id, filename),
            ModelSource::Url { url, .. } => write!(f, "{}", url),
            ModelSource::Local { path } => write!(f, "{}", path.display()),
        }
    }
}

impl ModelSource {
    /// Create HuggingFace source
    pub fn hugging_face(repo_id: impl Into<String>, filename: impl Into<String>) -> Self {
        ModelSource::HuggingFace {
            repo_id: repo_id.into(),
            filename: filename.into(),
            revision: None,
        }
    }

    /// Create HuggingFace source with revision
    pub fn huggingface(repo_id: impl Into<String>, filename: impl Into<String>) -> Self {
        Self::hugging_face(repo_id, filename)
    }

    /// Create HuggingFace source with revision
    pub fn huggingface_with_revision(
        repo_id: impl Into<String>,
        filename: impl Into<String>,
        revision: impl Into<String>,
    ) -> Self {
        ModelSource::HuggingFace {
            repo_id: repo_id.into(),
            filename: filename.into(),
            revision: Some(revision.into()),
        }
    }

    /// Create URL source
    pub fn url(url: impl Into<String>, filename: impl Into<String>) -> Self {
        ModelSource::Url {
            url: url.into(),
            filename: filename.into(),
        }
    }

    /// Create local source
    pub fn local(path: PathBuf) -> Self {
        ModelSource::Local { path }
    }
}

impl StorageConfig {
    /// Create local storage config with default path
    pub fn local_default() -> Self {
        StorageConfig::Local(crate::storage::LocalStorageConfig::default())
    }

    /// Create local storage config with custom path
    pub fn local(path: PathBuf) -> Self {
        StorageConfig::Local(crate::storage::LocalStorageConfig {
            base_dir: path,
            create_if_missing: true,
        })
    }

    /// Create S3 storage config
    #[cfg(feature = "s3-storage")]
    pub fn s3(bucket: impl Into<String>, region: impl Into<String>) -> Self {
        StorageConfig::S3(crate::storage::S3StorageConfig {
            region: region.into(),
            bucket: bucket.into(),
            prefix: None,
            access_key_id: None,
            secret_access_key: None,
            endpoint_url: None,
        })
    }
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Model ID
    pub id: ModelId,
    /// Version
    pub version: Version,
    /// Display name
    pub name: String,
    /// Model family (llama, phi, mistral, etc.)
    pub family: String,
    /// Parameter count (e.g., "7B", "8B")
    pub parameters: String,
    /// Quantization
    pub quantization: Quantization,
    /// File size in bytes
    pub size_bytes: u64,
    /// SHA256 checksum
    pub sha256: Option<String>,
    /// Source
    pub source: ModelSource,
    /// License
    pub license: String,
    /// Description
    pub description: String,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
    /// Context window size
    #[serde(default = "default_context_length")]
    pub context_length: u32,
    /// Created at
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    /// Last updated
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
    /// Custom metadata
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

fn default_context_length() -> u32 {
    4096
}

impl ModelMetadata {
    /// Validate metadata
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(crate::error::RegistryError::InvalidMetadata(
                "Model ID cannot be empty".to_string(),
            ));
        }

        if self.version.is_empty() {
            return Err(crate::error::RegistryError::InvalidMetadata(
                "Version cannot be empty".to_string(),
            ));
        }

        if self.size_bytes == 0 {
            return Err(crate::error::RegistryError::InvalidMetadata(
                "Size must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Get a formatted size string
    pub fn formatted_size(&self) -> String {
        format_bytes(self.size_bytes)
    }
}

/// Model version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    /// Version
    pub version: Version,
    /// Metadata
    pub metadata: ModelMetadata,
    /// Download status
    pub status: ModelStatus,
    /// Local path (if downloaded)
    pub local_path: Option<PathBuf>,
    /// Download progress (0-100)
    pub download_progress: u8,
}

/// Model status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelStatus {
    /// Available but not downloaded
    Available,
    /// Downloading
    Downloading,
    /// Downloaded and ready
    Ready,
    /// Failed
    Failed,
}

/// Download progress information
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// Current bytes downloaded
    pub downloaded: u64,
    /// Total bytes
    pub total: Option<u64>,
    /// Current speed in bytes/sec
    pub speed: u64,
    /// Estimated time remaining in seconds
    pub eta: Option<u64>,
    /// Current phase
    pub phase: DownloadPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadPhase {
    Checking,
    Downloading,
    Verifying,
    Complete,
}

/// Format bytes as human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_quantization_from_str() {
        assert_eq!(Quantization::from_str("q4").unwrap(), Quantization::Q4);
        assert_eq!(Quantization::from_str("Q4_K_M").unwrap(), Quantization::Q4);
        assert_eq!(Quantization::from_str("f16").unwrap(), Quantization::F16);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }
}
