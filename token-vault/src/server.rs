//! Token Vault Server
//!
//! HTTP server for remote vault access

use tokio::net::TcpListener;
use tracing::{info, Level};
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    fmt().with_max_level(Level::INFO).init();

    info!("🔐 Token Vault Server v0.1.0");
    info!("Starting server on 0.0.0.0:8080...");

    // TODO: Implement HTTP server with axum
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    info!("✓ Server listening on http://0.0.0.0:8080");
    info!("📚 API documentation: http://0.0.0.0:8080/docs");
    info!("\nPress Ctrl+C to stop");

    // For now, just keep the server running
    axum::serve(
        listener,
        axum::Router::new().route("/health", axum::routing::get(|| async { "OK" })),
    )
    .await?;

    Ok(())
}
