//! Model Registry CLI

use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use model_registry::{RegistryBuilder, StorageConfig};
use std::path::PathBuf;
use tracing::{error, info, Level};
use tracing_subscriber::fmt;

#[derive(Parser)]
#[command(name = "model-registry")]
#[command(about = "Version management and downloading system for ML models", long_about = None)]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Storage backend configuration (JSON)
    #[arg(short, long)]
    storage: Option<String>,

    /// Verbose mode
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List all registered models
    List {
        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,

        /// Filter by family
        #[arg(short = 'f', long)]
        family: Option<String>,
    },

    /// Download a model
    Download {
        /// Model ID
        model: String,

        /// Version (default: latest)
        #[arg(short, long)]
        version: Option<String>,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Upload a model
    Upload {
        /// Model file path
        path: PathBuf,

        /// Model ID
        model: String,

        /// Version
        #[arg(short, long)]
        version: String,
    },

    /// Show model information
    Info {
        /// Model ID
        model: String,

        /// Version
        #[arg(short, long)]
        version: Option<String>,
    },

    /// Delete a model
    Delete {
        /// Model ID
        model: String,

        /// Version
        #[arg(short, long)]
        version: String,

        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Search models
    Search {
        /// Search query (tag or family)
        query: String,
    },

    /// Import models from JSON
    Import {
        /// JSON file path
        path: PathBuf,
    },

    /// Export models to JSON
    Export {
        /// Output file path
        path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    fmt().with_max_level(level).with_target(false).init();

    // Parse storage config or use default
    let storage_config = if let Some(config_str) = cli.storage {
        serde_json::from_str(&config_str)?
    } else {
        StorageConfig::Local(model_registry::storage::LocalStorageConfig::default())
    };

    // Create registry
    let registry = RegistryBuilder::new()
        .storage(storage_config)
        .build()
        .await?;

    // Execute command
    match cli.command {
        Commands::List { tag, family } => {
            cmd_list(&registry, tag, family).await?;
        },
        Commands::Download {
            model,
            version,
            output,
        } => {
            cmd_download(&registry, model, version, output).await?;
        },
        Commands::Upload {
            path,
            model,
            version,
        } => {
            cmd_upload(&registry, path, model, version).await?;
        },
        Commands::Info { model, version } => {
            cmd_info(&registry, model, version).await?;
        },
        Commands::Delete {
            model,
            version,
            force,
        } => {
            cmd_delete(&registry, model, version, force).await?;
        },
        Commands::Search { query } => {
            cmd_search(&registry, query).await?;
        },
        Commands::Import { path } => {
            cmd_import(&registry, path).await?;
        },
        Commands::Export { path } => {
            cmd_export(&registry, path).await?;
        },
    }

    Ok(())
}

async fn cmd_list(
    registry: &model_registry::Registry,
    tag: Option<String>,
    family: Option<String>,
) -> anyhow::Result<()> {
    let models = if let Some(tag) = tag {
        registry.search_by_tag(&tag).await?
    } else if let Some(family) = family {
        registry.search_by_family(&family).await?
    } else {
        registry.list().await?
    };

    if models.is_empty() {
        println!("No models found.");
        return Ok(());
    }

    println!("Registered models:");
    println!();

    for model in models {
        println!("  {}@{}", model.id, model.version);
        println!("    Name: {}", model.name);
        println!("    Family: {}", model.family);
        println!("    Parameters: {}", model.parameters);
        println!("    Quantization: {}", model.quantization);
        println!("    Size: {}", model.formatted_size());
        println!("    License: {}", model.license);

        let downloaded = registry.is_downloaded(&model.id, &model.version).await;
        println!(
            "    Status: {}",
            if downloaded {
                "✓ Downloaded"
            } else {
                "○ Available"
            }
        );

        if !model.tags.is_empty() {
            println!("    Tags: {}", model.tags.join(", "));
        }
        println!();
    }

    Ok(())
}

async fn cmd_download(
    registry: &model_registry::Registry,
    model_id: String,
    version: Option<String>,
    output: Option<PathBuf>,
) -> anyhow::Result<()> {
    let version = match version {
        Some(v) => v,
        None => {
            let latest = registry.get_latest(&model_id).await?;
            latest.version.clone()
        },
    };

    info!("Downloading {}@{}", model_id, version);

    // Check if already downloaded
    if registry.is_downloaded(&model_id, &version).await {
        println!("Model {}@{} is already downloaded.", model_id, version);
        return Ok(());
    }

    // Create progress bar
    let progress = ProgressBar::new_spinner();
    progress.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner} {msg}")
            .unwrap(),
    );
    progress.set_message("Starting download...");

    let progress_clone = progress.clone();
    let callback = std::sync::Arc::new(move |p: model_registry::types::DownloadProgress| {
        match p.phase {
            model_registry::types::DownloadPhase::Downloading => {
                if let Some(total) = p.total {
                    progress_clone.set_length(total);
                    progress_clone.set_position(p.downloaded);
                    progress_clone.set_style(
                        ProgressStyle::default_bar()
                            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                            .unwrap()
                            .progress_chars("##-"),
                    );
                }
            },
            model_registry::types::DownloadPhase::Verifying => {
                progress_clone.set_message("Verifying checksum...");
            },
            model_registry::types::DownloadPhase::Complete => {
                progress_clone.finish_with_message("Download complete!");
            },
            _ => {},
        }
    });

    let path = registry
        .download(&model_id, &version, Some(callback))
        .await?;

    println!("Model downloaded to: {}", path.display());

    Ok(())
}

async fn cmd_upload(
    registry: &model_registry::Registry,
    path: PathBuf,
    model_id: String,
    version: String,
) -> anyhow::Result<()> {
    println!("Uploading {} as {}@{}", path.display(), model_id, version);

    // TODO: Implement upload functionality
    error!("Upload not yet implemented");
    Ok(())
}

async fn cmd_info(
    registry: &model_registry::Registry,
    model_id: String,
    version: Option<String>,
) -> anyhow::Result<()> {
    let metadata = match version {
        Some(v) => registry.get_version(&model_id, &v).await?,
        None => registry.get_latest(&model_id).await?,
    };

    println!("Model Information:");
    println!("  ID: {}", metadata.id);
    println!("  Version: {}", metadata.version);
    println!("  Name: {}", metadata.name);
    println!("  Family: {}", metadata.family);
    println!("  Parameters: {}", metadata.parameters);
    println!("  Quantization: {}", metadata.quantization);
    println!("  Size: {}", metadata.formatted_size());
    println!("  License: {}", metadata.license);
    println!("  Context Length: {}", metadata.context_length);
    println!();
    println!("  Description: {}", metadata.description);

    if !metadata.tags.is_empty() {
        println!("  Tags: {}", metadata.tags.join(", "));
    }

    let downloaded = registry
        .is_downloaded(&metadata.id, &metadata.version)
        .await;
    println!(
        "  Status: {}",
        if downloaded {
            "✓ Downloaded"
        } else {
            "○ Available"
        }
    );

    if downloaded {
        let size = registry.get_size(&metadata.id, &metadata.version).await?;
        println!(
            "  Local Size: {}",
            model_registry::types::format_bytes(size)
        );
    }

    Ok(())
}

async fn cmd_delete(
    registry: &model_registry::Registry,
    model_id: String,
    version: String,
    force: bool,
) -> anyhow::Result<()> {
    if !force {
        println!(
            "Are you sure you want to delete {}@{}? (y/N)",
            model_id, version
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    registry.delete(&model_id, &version).await?;
    println!("Deleted {}@{}", model_id, version);

    Ok(())
}

async fn cmd_search(registry: &model_registry::Registry, query: String) -> anyhow::Result<()> {
    // Try tag search first
    let mut results = registry.search_by_tag(&query).await?;

    // If no results, try family search
    if results.is_empty() {
        results = registry.search_by_family(&query).await?;
    }

    if results.is_empty() {
        println!("No models found matching '{}'", query);
        return Ok(());
    }

    println!("Found {} models matching '{}':", results.len(), query);
    println!();

    for model in results {
        println!("  {}@{} - {}", model.id, model.version, model.name);
    }

    Ok(())
}

async fn cmd_import(registry: &model_registry::Registry, path: PathBuf) -> anyhow::Result<()> {
    let json = tokio::fs::read_to_string(path).await?;
    registry.import_json(&json).await?;
    println!("Models imported successfully.");

    Ok(())
}

async fn cmd_export(registry: &model_registry::Registry, path: PathBuf) -> anyhow::Result<()> {
    let json = registry.export_json().await?;
    tokio::fs::write(path, json).await?;
    println!("Models exported successfully.");

    Ok(())
}
