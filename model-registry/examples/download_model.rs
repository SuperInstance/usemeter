//! Download a model example
//!
//! This example demonstrates how to download a model with progress tracking.

use model_registry::{ModelMetadata, ModelSource, Quantization, RegistryBuilder, StorageConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Create registry
    let registry = RegistryBuilder::new()
        .storage(StorageConfig::local_default())
        .build()
        .await?;

    println!("✓ Registry created\n");

    // Create metadata for a small model (BGE Micro for embeddings)
    let metadata = ModelMetadata {
        id: "bge-micro".to_string(),
        version: "1.0.0".to_string(),
        name: "BGE Micro".to_string(),
        family: "bge".to_string(),
        parameters: "22M".to_string(),
        quantization: Quantization::F16,
        size_bytes: 48_000_000, // 48MB
        sha256: None,
        source: ModelSource::huggingface("BAAI/bge-micro-v1.5", "bge-micro-v1.5.gguf"),
        license: "MIT".to_string(),
        description: "BGE Micro embedding model for RAG".to_string(),
        tags: vec!["embeddings".to_string(), "rag".to_string()],
        context_length: 512,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        custom: Default::default(),
    };

    // Register
    registry.register(metadata.clone()).await?;
    println!("✓ Registered model\n");

    // Create progress callback
    let progress_callback = Arc::new(|progress| {
        use model_registry::types::DownloadPhase;

        match progress.phase {
            DownloadPhase::Checking => {
                println!("Checking if download is needed...");
            },
            DownloadPhase::Downloading => {
                if let Some(total) = progress.total {
                    let percent = (progress.downloaded as f64 / total as f64) * 100.0;
                    let speed_mb = progress.speed as f64 / (1024.0 * 1024.0);
                    println!(
                        "Downloading: {:.1}% ({}/{} - {:.2} MB/s)",
                        percent,
                        model_registry::types::format_bytes(progress.downloaded),
                        model_registry::types::format_bytes(total),
                        speed_mb
                    );
                }
            },
            DownloadPhase::Verifying => {
                println!("Verifying checksum...");
            },
            DownloadPhase::Complete => {
                println!("✓ Download complete!");
            },
        }
    });

    // Download the model
    println!("Starting download...\n");
    let path = registry
        .download("bge-micro", "1.0.0", Some(progress_callback))
        .await?;

    println!("\n✓ Model downloaded to: {}", path.display());

    // Verify it's marked as downloaded
    let is_downloaded = registry.is_downloaded("bge-micro", "1.0.0").await;
    println!("✓ Downloaded status: {}", is_downloaded);

    Ok(())
}
