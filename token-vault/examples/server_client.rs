//! Server/Client usage example
//!
//! Demonstrates using the vault server and client (placeholder for future implementation).

use token_vault::TokenVault;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Token Vault - Server/Client Example\n");

    // For now, this example shows the local vault
    // In the future, it will demonstrate server/client usage

    let vault = TokenVault::new("server_client_example.db", "password123")?;

    println!("Storing configuration tokens...");
    vault.store("database_url", "postgresql://user:pass@localhost/db", None)?;
    vault.store("redis_url", "redis://localhost:6379", None)?;
    vault.store("api_endpoint", "https://api.example.com", None)?;

    println!("✓ Configuration stored\n");

    println!("Retrieving configuration:");
    let db_url = vault.retrieve("database_url", None)?;
    println!("  Database: {}", db_url.unwrap());

    let redis_url = vault.retrieve("redis_url", None)?;
    println!("  Redis: {}", redis_url.unwrap());

    let api_endpoint = vault.retrieve("api_endpoint", None)?;
    println!("  API: {}", api_endpoint.unwrap());

    println!("\n📝 Note: Server/client mode coming soon!");
    println!("   This will allow remote vault access over HTTP.");

    Ok(())
}
