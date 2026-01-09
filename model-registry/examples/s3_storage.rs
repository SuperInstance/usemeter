//! S3 storage backend example
//!
//! This example demonstrates using S3 as a storage backend.
//!
//! Note: This requires the `s3-storage` feature to be enabled:
//! cargo run --example s3_storage --features s3-storage

#[cfg(feature = "s3-storage")]
use model_registry::{
    registry::StorageConfig,
    types::{ModelMetadata, ModelSource, Quantization},
    RegistryBuilder,
};

#[cfg(feature = "s3-storage")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Configure S3 storage
    // Note: You can set AWS credentials via environment variables or AWS config files
    let storage_config = StorageConfig::s3("my-model-bucket", "us-east-1");

    // Create registry with S3 storage
    let registry = RegistryBuilder::new()
        .storage(storage_config)
        .build()
        .await?;

    println!("✓ Registry created with S3 storage\n");

    // Register a model
    let metadata = ModelMetadata {
        id: "my-model".to_string(),
        version: "1.0.0".to_string(),
        name: "My Model".to_string(),
        family: "custom".to_string(),
        parameters: "7B".to_string(),
        quantization: Quantization::Q4,
        size_bytes: 4_100_000_000,
        sha256: None,
        source: ModelSource::url("https://example.com/model.gguf", "model.gguf"),
        license: "MIT".to_string(),
        description: "A custom model stored in S3".to_string(),
        tags: vec!["custom".to_string()],
        context_length: 4096,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        custom: Default::default(),
    };

    registry.register(metadata.clone()).await?;
    println!("✓ Registered model\n");

    // The model will be stored in S3 when downloaded
    println!(
        "S3 path: s3://my-model-bucket/models/{}/{}/{}/model.bin",
        metadata.id, metadata.version, metadata.quantization
    );

    Ok(())
}

#[cfg(not(feature = "s3-storage"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("This example requires the 's3-storage' feature.");
    println!("Run with: cargo run --example s3_storage --features s3-storage");
    Ok(())
}
