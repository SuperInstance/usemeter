//! Basic token vault example
//!
//! Demonstrates creating a vault, storing tokens, and retrieving them.

use token_vault::TokenVault;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create vault with password
    let vault = TokenVault::new("example_vault.db", "my-secure-password-123")?;

    println!("🔐 Token Vault Example\n");

    // Store API tokens
    println!("Storing API tokens...");
    vault.store("github_token", "ghp_1234567890abcdef", None)?;
    vault.store("aws_access_key", "AKIAIOSFODNN7EXAMPLE", None)?;
    vault.store("slack_token", "xoxb-1234567890-1234567890123", None)?;

    println!("✓ Stored 3 tokens\n");

    // Retrieve tokens
    println!("Retrieving tokens:");
    let github_token = vault.retrieve("github_token", None)?;
    println!("  github_token: {}", github_token.unwrap());

    let aws_key = vault.retrieve("aws_access_key", None)?;
    println!("  aws_access_key: {}", aws_key.unwrap());

    let slack_token = vault.retrieve("slack_token", None)?;
    println!("  slack_token: {}", slack_token.unwrap());

    // List all tokens
    println!("\nAll tokens in vault:");
    let tokens = vault.list_tokens(None)?;
    for token in tokens {
        println!("  - {}", token);
    }

    // Update a token
    println!("\nUpdating github_token...");
    vault.update("github_token", "ghp_newtoken123456", None)?;
    let updated = vault.retrieve("github_token", None)?;
    println!("  New value: {}", updated.unwrap());

    // Delete a token
    println!("\nDeleting slack_token...");
    vault.delete("slack_token", None)?;
    println!("✓ Deleted");

    // Show audit log
    println!("\n📋 Audit Log:");
    let entries = vault.audit_entries();
    for entry in entries.iter().take(10) {
        println!("  [{:?}] {} - {:?}", entry.operation, entry.target, entry.result);
    }

    println!("\n✅ Example completed successfully!");
    println!("🗑️  Database file: example_vault.db");

    Ok(())
}
