//! Model Registry - Version management and downloading system for ML models
//!
//! # Overview
//!
//! `model-registry` is a Rust library and CLI tool for managing ML model versions,
//! downloading models from various sources (HuggingFace, URLs, local files), and
//! storing them in configurable storage backends (local filesystem, S3).
//!
//! # Features
//!
//! - **Version tracking**: Manage multiple versions of models
//! - **Multiple storage backends**: Local filesystem, S3 (with extensibility)
//! - **Progress tracking**: Real-time download progress with speed and ETA
//! - **Checksum verification**: SHA256 validation for downloaded models
//! - **Metadata management**: Rich metadata for each model version
//! - **Search**: Find models by tags, family, or other criteria
//!
//! # Quick Start
//!
//! ## Library Usage
//!
//! ```no_run
//! use model_registry::{Registry, RegistryBuilder, StorageConfig};
//! use model_registry::types::{ModelMetadata, ModelSource, Quantization};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create registry with default local storage
//! let registry = RegistryBuilder::new()
//!     .storage(StorageConfig::local_default())
//!     .build()
//!     .await?;
//!
//! // Register a model
//! let metadata = ModelMetadata {
//!     id: "llama-3.2-8b".to_string(),
//!     version: "1.0.0".to_string(),
//!     name: "Llama 3.2 8B".to_string(),
//!     family: "llama".to_string(),
//!     parameters: "8B".to_string(),
//!     quantization: Quantization::Q4,
//!     size_bytes: 4_700_000_000,
//!     sha256: None,
//!     source: ModelSource::huggingface(
//!         "meta-llama/Llama-3.2-8B-Instruct-GGUF",
//!         "llama-3.2-8b-instruct-q4_k_m.gguf"
//!     ),
//!     license: "Apache-2.0".to_string(),
//!     description: "Llama 3.2 8B model".to_string(),
//!     tags: vec!["llm".to_string(), "text-generation".to_string()],
//!     context_length: 8192,
//!     created_at: chrono::Utc::now(),
//!     updated_at: chrono::Utc::now(),
//!     custom: Default::default(),
//! };
//!
//! registry.register(metadata).await?;
//!
//! // Download the model
//! let progress_callback = std::sync::Arc::new(|p| {
//!     println!("Downloaded: {} / {}", p.downloaded, p.total.unwrap_or(0));
//! });
//!
//! let path = registry
//!     .download("llama-3.2-8b", "1.0.0", Some(progress_callback))
//!     .await?;
//!
//! println!("Model downloaded to: {}", path.display());
//! # Ok(())
//! # }
//! ```
//!
//! ## CLI Usage
//!
//! ```bash
//! # Install
//! cargo install model-registry
//!
//! # List all models
//! model-registry list
//!
//! # Download a model
//! model-registry download llama-3.2-8b --version 1.0.0
//!
//! # Show model info
//! model-registry info llama-3.2-8b
//!
//! # Delete a model
//! model-registry delete llama-3.2-8b --version 1.0.0
//! ```

pub mod download;
pub mod error;
pub mod registry;
pub mod storage;
pub mod types;

// Re-exports
pub use error::{RegistryError, Result};
pub use registry::{Registry, RegistryBuilder};
pub use storage::StorageConfig;
pub use types::{ModelMetadata, ModelSource, ModelStatus, Quantization};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_display() {
        let q4 = Quantization::Q4;
        assert_eq!(q4.to_string(), "q4_k_m");
    }
}
