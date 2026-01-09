//! Search and query example
//!
//! This example demonstrates searching and querying models.

use model_registry::{ModelMetadata, ModelSource, Quantization, RegistryBuilder, StorageConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let registry = RegistryBuilder::new()
        .storage(StorageConfig::local_default())
        .build()
        .await?;

    println!("✓ Registry created\n");

    // Register various models
    let models = vec![
        ModelMetadata {
            id: "llama-3.2-8b".to_string(),
            version: "1.0.0".to_string(),
            name: "Llama 3.2 8B".to_string(),
            family: "llama".to_string(),
            parameters: "8B".to_string(),
            quantization: Quantization::Q4,
            size_bytes: 4_700_000_000,
            sha256: None,
            source: ModelSource::huggingface("meta-llama/Llama-3.2-8B-Instruct-GGUF", "model.gguf"),
            license: "Apache-2.0".to_string(),
            description: "General purpose LLM".to_string(),
            tags: vec![
                "llm".to_string(),
                "text-generation".to_string(),
                "reasoning".to_string(),
            ],
            context_length: 8192,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            custom: Default::default(),
        },
        ModelMetadata {
            id: "bge-micro".to_string(),
            version: "1.0.0".to_string(),
            name: "BGE Micro".to_string(),
            family: "bge".to_string(),
            parameters: "22M".to_string(),
            quantization: Quantization::F16,
            size_bytes: 48_000_000,
            sha256: None,
            source: ModelSource::huggingface("BAAI/bge-micro-v1.5", "model.gguf"),
            license: "MIT".to_string(),
            description: "Embedding model for RAG".to_string(),
            tags: vec!["embeddings".to_string(), "rag".to_string()],
            context_length: 512,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            custom: Default::default(),
        },
        ModelMetadata {
            id: "mistral-7b".to_string(),
            version: "1.0.0".to_string(),
            name: "Mistral 7B".to_string(),
            family: "mistral".to_string(),
            parameters: "7B".to_string(),
            quantization: Quantization::Q4,
            size_bytes: 4_100_000_000,
            sha256: None,
            source: ModelSource::huggingface(
                "mistralai/Mistral-7B-Instruct-v0.3-GGUF",
                "model.gguf",
            ),
            license: "Apache-2.0".to_string(),
            description: "Safety-focused LLM".to_string(),
            tags: vec!["llm".to_string(), "safety".to_string()],
            context_length: 32768,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            custom: Default::default(),
        },
    ];

    for metadata in &models {
        registry.register(metadata.clone()).await?;
    }

    println!("✓ Registered {} models\n", models.len());

    // Search by tag
    println!("Models tagged with 'llm':");
    let llm_models = registry.search_by_tag("llm").await?;
    for model in &llm_models {
        println!("  • {} - {}", model.id, model.name);
    }

    println!("\nModels tagged with 'rag':");
    let rag_models = registry.search_by_tag("rag").await?;
    for model in &rag_models {
        println!("  • {} - {}", model.id, model.name);
    }

    // Search by family
    println!("\nModels in 'llama' family:");
    let llama_models = registry.search_by_family("llama").await?;
    for model in &llama_models {
        println!("  • {}@{}", model.id, model.version);
    }

    // Export to JSON
    let json = registry.export_json().await?;
    println!("\n✓ Exported {} models to JSON", models.len());

    Ok(())
}
