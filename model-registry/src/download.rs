//! Download functionality

use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info, warn};

use crate::error::{RegistryError, Result};
use crate::types::{DownloadPhase, DownloadProgress, ModelMetadata, ModelSource};

/// Download progress callback
pub type ProgressCallback = Arc<dyn Fn(DownloadProgress) + Send + Sync>;

/// Download configuration
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    /// Buffer size for streaming (1 MB)
    pub buffer_size: usize,
    /// Timeout in seconds
    pub timeout_secs: u64,
    /// Retry attempts
    pub retry_attempts: u32,
    /// User agent
    pub user_agent: String,
    /// HuggingFace token (optional)
    pub hf_token: Option<String>,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1024 * 1024, // 1 MB
            timeout_secs: 30,
            retry_attempts: 3,
            user_agent: format!("model-registry/{}", env!("CARGO_PKG_VERSION")),
            hf_token: std::env::var("HF_TOKEN").ok(),
        }
    }
}

/// Model downloader
pub struct Downloader {
    config: DownloadConfig,
    client: reqwest::Client,
}

impl Downloader {
    /// Create new downloader
    pub fn new(config: DownloadConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(&config.user_agent)
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| RegistryError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Create with default config
    pub fn default_config() -> Result<Self> {
        Self::new(DownloadConfig::default())
    }

    /// Download a model
    pub async fn download(
        &self,
        metadata: &ModelMetadata,
        dest: &Path,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<()> {
        info!(
            "Downloading model {}@{} to {}",
            metadata.id,
            metadata.version,
            dest.display()
        );

        // Check if already exists
        if dest.exists() {
            if let Some(expected) = &metadata.sha256 {
                if self.verify_checksum(dest, expected).await? {
                    info!("Model already exists and checksum matches");
                    self.report_complete(progress_callback, metadata.size_bytes);
                    return Ok(());
                }
                warn!("Existing file has invalid checksum, re-downloading");
            } else {
                info!("Model already exists (no checksum to verify)");
                return Ok(());
            }
        }

        // Create parent directory
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Download based on source
        let temp_path = dest.with_extension("part");

        match &metadata.source {
            ModelSource::HuggingFace {
                repo_id,
                filename,
                revision,
            } => {
                self.download_from_huggingface(
                    repo_id,
                    filename,
                    revision.as_deref(),
                    &temp_path,
                    metadata.size_bytes,
                    progress_callback.clone(),
                )
                .await?;
            },
            ModelSource::Url { url, .. } => {
                self.download_from_url(
                    url,
                    &temp_path,
                    metadata.size_bytes,
                    progress_callback.clone(),
                )
                .await?;
            },
            ModelSource::Local { path } => {
                self.copy_local(path, &temp_path).await?;
            },
        }

        // Verify checksum if provided
        if let Some(expected) = &metadata.sha256 {
            self.report_progress(
                progress_callback.clone(),
                DownloadProgress {
                    downloaded: metadata.size_bytes,
                    total: Some(metadata.size_bytes),
                    speed: 0,
                    eta: Some(0),
                    phase: DownloadPhase::Verifying,
                },
            );

            if !self.verify_checksum(&temp_path, expected).await? {
                tokio::fs::remove_file(&temp_path).await?;
                return Err(RegistryError::ChecksumMismatch {
                    model: format!("{}@{}", metadata.id, metadata.version),
                    expected: expected.clone(),
                    actual: "invalid".to_string(),
                });
            }
        }

        // Move to final destination
        tokio::fs::rename(&temp_path, dest).await?;

        self.report_complete(progress_callback, metadata.size_bytes);

        info!("Download complete: {}", dest.display());
        Ok(())
    }

    /// Download from HuggingFace
    async fn download_from_huggingface(
        &self,
        repo_id: &str,
        filename: &str,
        revision: Option<&str>,
        dest: &Path,
        total_size: u64,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<()> {
        let revision = revision.unwrap_or("main");
        let url = format!(
            "https://huggingface.co/{}/resolve/{}/{}",
            repo_id, revision, filename
        );

        debug!("Downloading from HuggingFace: {}", url);
        self.download_from_url(&url, dest, total_size, progress_callback)
            .await
    }

    /// Download from URL with progress tracking
    async fn download_from_url(
        &self,
        url: &str,
        dest: &Path,
        total_size: u64,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<()> {
        let mut request = self.client.get(url);

        // Add HuggingFace token if needed
        if url.contains("huggingface.co") {
            if let Some(token) = &self.config.hf_token {
                request = request.header("Authorization", format!("Bearer {}", token));
            }
        }

        let response = request.send().await.map_err(|e| {
            RegistryError::DownloadFailed(format!("Failed to fetch {}: {}", url, e))
        })?;

        if !response.status().is_success() {
            return Err(RegistryError::DownloadFailed(format!(
                "HTTP {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        let total_size = response.content_length().unwrap_or(total_size);

        // Stream download
        let mut file = tokio::fs::File::create(dest).await?;
        let mut stream = response.bytes_stream();

        let mut downloaded = 0u64;
        let mut last_update = Instant::now();
        let mut last_downloaded = 0u64;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| {
                RegistryError::DownloadFailed(format!("Failed to read chunk: {}", e))
            })?;

            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            // Update progress every 100ms
            let now = Instant::now();
            if now.duration_since(last_update).as_millis() >= 100 {
                let elapsed = now.duration_since(last_update).as_secs_f64();
                let bytes_since = downloaded - last_downloaded;
                let speed = if elapsed > 0.0 {
                    (bytes_since as f64 / elapsed) as u64
                } else {
                    0
                };

                let eta = if speed > 0 {
                    Some((total_size - downloaded) / speed)
                } else {
                    None
                };

                self.report_progress(
                    progress_callback.clone(),
                    DownloadProgress {
                        downloaded,
                        total: Some(total_size),
                        speed,
                        eta,
                        phase: DownloadPhase::Downloading,
                    },
                );

                last_update = now;
                last_downloaded = downloaded;
            }
        }

        file.flush().await?;

        Ok(())
    }

    /// Copy local file
    async fn copy_local(&self, src: &Path, dest: &Path) -> Result<()> {
        tokio::fs::copy(src, dest).await?;
        Ok(())
    }

    /// Verify SHA256 checksum
    async fn verify_checksum(&self, path: &Path, expected: &str) -> Result<bool> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; self.config.buffer_size];

        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let result = hex::encode(hasher.finalize());
        Ok(result.eq_ignore_ascii_case(expected))
    }

    /// Report download progress
    fn report_progress(&self, callback: Option<ProgressCallback>, progress: DownloadProgress) {
        if let Some(cb) = callback {
            cb(progress);
        }
    }

    /// Report download complete
    fn report_complete(&self, callback: Option<ProgressCallback>, total: u64) {
        if let Some(cb) = callback {
            cb(DownloadProgress {
                downloaded: total,
                total: Some(total),
                speed: 0,
                eta: Some(0),
                phase: DownloadPhase::Complete,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ModelMetadata, ModelSource, Quantization};

    #[tokio::test]
    async fn test_downloader_creation() {
        let downloader = Downloader::default_config().unwrap();
        assert_eq!(downloader.config.buffer_size, 1024 * 1024);
    }

    #[tokio::test]
    async fn test_checksum_verification() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.bin");
        tokio::fs::write(&test_file, b"test data").await.unwrap();

        let downloader = Downloader::default_config().unwrap();

        // Correct checksum
        let correct_hash = sha2::Sha256::digest(b"test data");
        let correct_hash = hex::encode(correct_hash);

        assert!(downloader
            .verify_checksum(&test_file, &correct_hash)
            .await
            .unwrap());

        // Incorrect checksum
        assert!(!downloader
            .verify_checksum(&test_file, "invalid")
            .await
            .unwrap());
    }
}
