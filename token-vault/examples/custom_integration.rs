//! Custom integration example
//!
//! Demonstrates how to integrate token-vault into your application.

use token_vault::TokenVault;

/// Application configuration that uses token vault
struct AppConfig {
    vault: TokenVault,
    session: String,
}

impl AppConfig {
    fn new(db_path: &str, password: &str, session: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let vault = TokenVault::new(db_path, password)?;
        Ok(Self {
            vault,
            session: session.to_string(),
        })
    }

    /// Get API key for service
    fn get_api_key(&self, service: &str) -> Result<String, Box<dyn std::error::Error>> {
        let token_name = format!("{}_api_key", service);
        self.vault
            .retrieve(&token_name, Some(&self.session))?
            .ok_or_else(|| format!("API key not found: {}", token_name).into())
    }

    /// Set API key for service
    fn set_api_key(&self, service: &str, key: &str) -> Result<(), Box<dyn std::error::Error>> {
        let token_name = format!("{}_api_key", service);
        self.vault.store(&token_name, key, Some(&self.session))?;
        Ok(())
    }

    /// Get database URL
    fn get_database_url(&self) -> Result<String, Box<dyn std::error::Error>> {
        self.vault
            .retrieve("database_url", Some(&self.session))?
            .ok_or_else(|| "Database URL not found".into())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Token Vault - Custom Integration Example\n");

    // Initialize app configuration
    let config = AppConfig::new("integration_example.db", "app-password", "production")?;

    // Store configuration
    println!("Storing application configuration...");
    config.set_api_key("stripe", "sk_live_1234567890abcdef")?;
    config.set_api_key("github", "ghp_1234567890abcdef")?;
    config.vault.store(
        "database_url",
        "postgresql://app:password@db.example.com/production",
        Some("production"),
    )?;

    println!("✓ Configuration stored\n");

    // Use configuration in application
    println!("Using configuration in application:");
    println!("  Stripe API Key: {}", config.get_api_key("stripe")?);
    println!("  GitHub API Key: {}", config.get_api_key("github")?);
    println!("  Database URL: {}", config.get_database_url()?);

    println!("\n✅ Integration example completed!");
    println!("\n💡 Tips for integration:");
    println!("  1. Use session IDs to separate environments (dev/staging/prod)");
    println!("  2. Load vault at application startup");
    println!("  3. Retrieve secrets on-demand (avoid caching in memory)");
    println!("  4. Use audit log to track access patterns");
    println!("  5. Store the vault database file securely");

    Ok(())
}
