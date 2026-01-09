//! Basic registry usage example
//!
//! This example demonstrates how to create a registry, register a model,
//! and query model information.

use model_registry::{ModelMetadata, ModelSource, Quantization, RegistryBuilder, StorageConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create registry with default local storage
    let registry = RegistryBuilder::new()
        .storage(StorageConfig::local_default())
        .build()
        .await?;

    println!("✓ Registry created\n");

    // Create model metadata
    let metadata = ModelMetadata {
        id: "llama-3.2-8b".to_string(),
        version: "1.0.0".to_string(),
        name: "Llama 3.2 8B".to_string(),
        family: "llama".to_string(),
        parameters: "8B".to_string(),
        quantization: Quantization::Q4,
        size_bytes: 4_700_000_000, // 4.7GB
        sha256: None,
        source: ModelSource::huggingface(
            "meta-llama/Llama-3.2-8B-Instruct-GGUF",
            "llama-3.2-8b-instruct-q4_k_m.gguf",
        ),
        license: "Apache-2.0".to_string(),
        description: "Llama 3.2 8B instruct model with 8K context".to_string(),
        tags: vec![
            "llm".to_string(),
            "text-generation".to_string(),
            "reasoning".to_string(),
        ],
        context_length: 8192,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        custom: Default::default(),
    };

    // Register the model
    registry.register(metadata.clone()).await?;
    println!("✓ Registered model: {}@{}", metadata.id, metadata.version);

    // List all models
    let models = registry.list().await?;
    println!("\n✓ Total models registered: {}", models.len());

    // Get specific version
    let retrieved = registry.get_version("llama-3.2-8b", "1.0.0").await?;
    println!("\nModel Details:");
    println!("  Name: {}", retrieved.name);
    println!("  Family: {}", retrieved.family);
    println!("  Parameters: {}", retrieved.parameters);
    println!("  Quantization: {}", retrieved.quantization);
    println!("  Size: {}", retrieved.formatted_size());
    println!("  Context: {} tokens", retrieved.context_length);
    println!("  Tags: {}", retrieved.tags.join(", "));

    Ok(())
}
