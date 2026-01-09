//! Multiple versions example
//!
//! This example demonstrates managing multiple versions of the same model.

use model_registry::{ModelMetadata, ModelSource, Quantization, RegistryBuilder, StorageConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let registry = RegistryBuilder::new()
        .storage(StorageConfig::local_default())
        .build()
        .await?;

    println!("✓ Registry created\n");

    // Register multiple versions of the same model
    let versions = vec![
        ModelMetadata {
            id: "llama-3.2-8b".to_string(),
            version: "1.0.0".to_string(),
            name: "Llama 3.2 8B v1.0.0".to_string(),
            family: "llama".to_string(),
            parameters: "8B".to_string(),
            quantization: Quantization::Q4,
            size_bytes: 4_700_000_000,
            sha256: None,
            source: ModelSource::huggingface(
                "meta-llama/Llama-3.2-8B-Instruct-GGUF",
                "llama-3.2-8b-instruct-q4_k_m.gguf",
            ),
            license: "Apache-2.0".to_string(),
            description: "Initial release".to_string(),
            tags: vec!["stable".to_string()],
            context_length: 8192,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            custom: Default::default(),
        },
        ModelMetadata {
            id: "llama-3.2-8b".to_string(),
            version: "1.1.0".to_string(),
            name: "Llama 3.2 8B v1.1.0".to_string(),
            family: "llama".to_string(),
            parameters: "8B".to_string(),
            quantization: Quantization::Q4,
            size_bytes: 4_700_000_000,
            sha256: None,
            source: ModelSource::huggingface(
                "meta-llama/Llama-3.2-8B-Instruct-GGUF",
                "llama-3.2-8b-instruct-q4_k_m.gguf",
            ),
            license: "Apache-2.0".to_string(),
            description: "Improved alignment".to_string(),
            tags: vec!["stable".to_string()],
            context_length: 8192,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            custom: Default::default(),
        },
        ModelMetadata {
            id: "llama-3.2-8b".to_string(),
            version: "2.0.0-beta".to_string(),
            name: "Llama 3.2 8B v2.0.0-beta".to_string(),
            family: "llama".to_string(),
            parameters: "8B".to_string(),
            quantization: Quantization::Q4,
            size_bytes: 4_700_000_000,
            sha256: None,
            source: ModelSource::huggingface(
                "meta-llama/Llama-3.2-8B-Instruct-GGUF",
                "llama-3.2-8b-instruct-q4_k_m.gguf",
            ),
            license: "Apache-2.0".to_string(),
            description: "Beta release with extended context".to_string(),
            tags: vec!["beta".to_string()],
            context_length: 16384,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            custom: Default::default(),
        },
    ];

    for metadata in &versions {
        registry.register(metadata.clone()).await?;
        println!("✓ Registered: {}@{}", metadata.id, metadata.version);
    }

    println!("\nAll versions of llama-3.2-8b:");
    let all_versions = registry.get("llama-3.2-8b").await?;

    for version in &all_versions {
        println!(
            "  • {} - {} ({} context)",
            version.version, version.description, version.context_length
        );
    }

    println!("\nLatest version:");
    let latest = registry.get_latest("llama-3.2-8b").await?;
    println!(
        "  {}@{} - {}",
        latest.id, latest.version, latest.description
    );

    Ok(())
}
