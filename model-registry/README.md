# Model Registry

[![crates.io](https://img.shields.io/crates/v/model-registry)](https://crates.io/crates/model-registry)
[![docs.rs](https://img.shields.io/docsrs/model-registry)](https://docs.rs/model-registry)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Build Status](https://github.com/SuperInstance/model-registry/workflows/CI/badge.svg)](https://github.com/SuperInstance/model-registry/actions)

**Version management and downloading system for ML models**

Model Registry is a Rust library and CLI tool for managing ML model versions, downloading models from various sources, and storing them in configurable storage backends.

## Features

- **Version Tracking**: Manage multiple versions of the same model
- **Multiple Storage Backends**:
  - Local filesystem (default)
  - Amazon S3 (with `s3-storage` feature)
  - Extensible storage abstraction
- **Progress Tracking**: Real-time download progress with speed and ETA
- **Checksum Verification**: SHA256 validation for downloaded models
- **Rich Metadata**: Track model family, parameters, quantization, tags, and more
- **Search**: Find models by tags, family, or other criteria
- **CLI Tool**: Command-line interface for common operations

## Installation

### CLI

```bash
cargo install model-registry
```

### Library

```toml
[dependencies]
model-registry = "0.1"
```

For S3 support:

```toml
[dependencies]
model-registry = { version = "0.1", features = ["s3-storage"] }
```

## Quick Start

### Library Usage

```rust
use model_registry::{ModelMetadata, ModelSource, Quantization, RegistryBuilder, StorageConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create registry with default local storage
    let registry = RegistryBuilder::new()
        .storage(StorageConfig::local_default())
        .build()
        .await?;

    // Register a model
    let metadata = ModelMetadata {
        id: "llama-3.2-8b".to_string(),
        version: "1.0.0".to_string(),
        name: "Llama 3.2 8B".to_string(),
        family: "llama".to_string(),
        parameters: "8B".to_string(),
        quantization: Quantization::Q4,
        size_bytes: 4_700_000_000,
        sha256: None,
        source: ModelSource::huggingface(
            "meta-llama/Llama-3.2-8B-Instruct-GGUF",
            "llama-3.2-8b-instruct-q4_k_m.gguf"
        ),
        license: "Apache-2.0".to_string(),
        description: "Llama 3.2 8B model".to_string(),
        tags: vec!["llm".to_string(), "text-generation".to_string()],
        context_length: 8192,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        custom: Default::default(),
    };

    registry.register(metadata).await?;

    // Download with progress tracking
    let progress = std::sync::Arc::new(|p| {
        println!("Downloaded: {} / {}", p.downloaded, p.total.unwrap_or(0));
    });

    let path = registry.download("llama-3.2-8b", "1.0.0", Some(progress)).await?;

    println!("Model downloaded to: {}", path.display());
    Ok(())
}
```

### CLI Usage

```bash
# List all models
model-registry list

# Download a model
model-registry download llama-3.2-8b --version 1.0.0

# Show model info
model-registry info llama-3.2-8b

# Search by tag
model-registry search llm

# Export models to JSON
model-registry export models.json

# Import models from JSON
model-registry import models.json
```

## Storage Backends

### Local Storage (Default)

Models are stored in `~/.model-registry/models/` by default.

```rust
let storage = StorageConfig::local_default();
// Or specify a custom path
let storage = StorageConfig::local(PathBuf::from("/custom/path"));
```

### S3 Storage

```rust
let storage = StorageConfig::s3("my-bucket", "us-east-1");
```

## Model Sources

### HuggingFace

```rust
let source = ModelSource::huggingface(
    "meta-llama/Llama-3.2-8B-Instruct-GGUF",
    "llama-3.2-8b-instruct-q4_k_m.gguf"
);
```

### Direct URL

```rust
let source = ModelSource::url(
    "https://example.com/model.gguf",
    "model.gguf"
);
```

### Local File

```rust
let source = ModelSource::local(PathBuf::from("/path/to/model.gguf"));
```

## Examples

See the [examples](https://github.com/SuperInstance/model-registry/tree/main/examples) directory:

- [basic_registry](examples/basic_registry.rs) - Create a registry and register models
- [download_model](examples/download_model.rs) - Download a model with progress tracking
- [multiple_versions](examples/multiple_versions.rs) - Manage multiple versions
- [search_and_query](examples/search_and_query.rs) - Search and query models
- [s3_storage](examples/s3_storage.rs) - Use S3 as storage backend

## Model Metadata

Each model version tracks:

- **ID**: Unique identifier (e.g., "llama-3.2-8b")
- **Version**: Semantic version (e.g., "1.0.0")
- **Family**: Model family (e.g., "llama", "mistral", "phi")
- **Parameters**: Parameter count (e.g., "7B", "8B")
- **Quantization**: Quantization level (Q4, Q5, Q8, F16, F32)
- **Size**: File size in bytes
- **SHA256**: Optional checksum for verification
- **Source**: Download source (HuggingFace, URL, or local)
- **License**: Model license
- **Description**: Human-readable description
- **Tags**: Arbitrary tags for searching
- **Context Length**: Maximum context window
- **Custom Fields**: Additional metadata

## Progress Tracking

Track download progress with real-time updates:

```rust
let callback = std::sync::Arc::new(|progress| {
    match progress.phase {
        DownloadPhase::Checking => println!("Checking..."),
        DownloadPhase::Downloading => {
            let percent = (progress.downloaded as f64 / progress.total.unwrap() as f64) * 100.0;
            println!("Progress: {:.1}%", percent);
        }
        DownloadPhase::Verifying => println!("Verifying..."),
        DownloadPhase::Complete => println!("Done!"),
    }
});

registry.download("model-id", "version", Some(callback)).await?;
```

## Search and Query

```rust
// Search by tag
let llm_models = registry.search_by_tag("llm").await?;

// Search by family
let llama_models = registry.search_by_family("llama").await?;

// Get specific version
let model = registry.get_version("llama-3.2-8b", "1.0.0").await?;

// Get latest version
let latest = registry.get_latest("llama-3.2-8b").await?;

// Check if downloaded
let is_downloaded = registry.is_downloaded("llama-3.2-8b", "1.0.0").await;
```

## CLI Commands

### List Models

```bash
model-registry list
model-registry list --tag llm
model-registry list --family llama
```

### Download Models

```bash
model-registry download <model-id>
model-registry download <model-id> --version 1.0.0
model-registry download <model-id> --output /path/to/output
```

### Model Information

```bash
model-registry info <model-id>
model-registry info <model-id> --version 1.0.0
```

### Delete Models

```bash
model-registry delete <model-id> --version 1.0.0
model-registry delete <model-id> --version 1.0.0 --force
```

### Import/Export

```bash
model-registry export models.json
model-registry import models.json
```

## Features

| Feature | Default | Feature Flag |
|---------|---------|--------------|
| Local storage | ✓ | `local-storage` |
| S3 storage | | `s3-storage` |
| All storage | | `all-storage` |

## Environment Variables

- `HF_TOKEN` - HuggingFace authentication token (for gated models)
- `RUST_LOG` - Logging level (e.g., `debug`, `info`)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## Used By

- [SuperInstance](https://github.com/SuperInstance/SuperInstance) - AI inference platform

## Related Projects

- [privox](https://github.com/SuperInstance/privox) - Privacy redaction engine
- [tripartite-rs](https://github.com/SuperInstance/tripartite-rs) - Multi-agent consensus system
- [knowledge-vault](https://github.com/SuperInstance/knowledge-vault) - Vector database for RAG

## Acknowledgments

Inspired by the need for better model management in ML applications.
