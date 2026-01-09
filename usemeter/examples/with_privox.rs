//! Integration example with privox
//!
//! Demonstrates metering privox redaction operations.

use usemeter::{Event, Meter, StorageBackend};
use usemeter::storage::SqliteBackend;
use chrono::Utc;

// This example shows how to integrate usemeter with privox
// In a real application, you would use: use privox::{Redactor, Pattern};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = SqliteBackend::new_in_memory()?;
    let meter = Meter::new(storage);
    meter.initialize().await?;

    println!("🔐 Privox Integration Example\n");
    println!("Metering privox redaction operations...\n");

    // Simulate privox redaction operations
    let test_texts = vec![
        ("My email is user@example.com", "email"),
        ("Call me at 555-123-4567", "phone"),
        ("SSN: 123-45-6789", "ssn"),
        ("Credit card: 4111-1111-1111-1111", "credit_card"),
    ];

    for (text, pattern) in test_texts {
        // In real integration:
        // let redactor = Redactor::with_pattern(&Pattern::from_name(pattern)?);
        // let redacted = redactor.redact(text)?;
        // let tokens = text.len() as i64; // Approximate

        let tokens = text.len() as i64;

        // Record the redaction event
        let event = Event::builder()
            .event_type("privox_redaction")
            .user_id("user-123")
            .resource_id("privox-engine")
            .metric("characters_processed", tokens)
            .metric("patterns_matched", 1)
            .tag("pattern", pattern)
            .tag("model", "standard")
            .build()?;

        meter.record(event).await?;

        println!("✓ Redacted {} ({} chars) - pattern: {}",
            text.chars().take(20).collect::<String>(),
            tokens,
            pattern
        );
    }

    println!("\n📊 Privox Usage Statistics:");

    let stats = meter.query()
        .event_type("privox_redaction")
        .execute_stats()
        .await?;

    println!("  Total redactions: {}", stats.total_events);
    println!("  Total characters: {:.0}", stats.get_metric_sum("characters_processed").unwrap_or(0.0));
    println!("  Total patterns matched: {:.0}", stats.get_metric_sum("patterns_matched").unwrap_or(0.0));

    println!("\n💡 Integration Benefits:");
    println!("  • Track redaction volume by user");
    println!("  • Monitor which patterns are most used");
    println!("  • Calculate costs based on usage");
    println!("  • Generate billing reports");

    Ok(())
}
