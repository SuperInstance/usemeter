//! Basic metering example
//!
//! Demonstrates basic usage tracking with usemeter.

use chrono::Utc;
use usemeter::storage::SqliteBackend;
use usemeter::{Event, Meter, StorageBackend};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize storage and meter
    let storage = SqliteBackend::new_in_memory()?;
    let meter = Meter::new(storage);
    meter.initialize().await?;

    println!("🔢 Basic Metering Example\n");

    // Record some API calls
    for i in 1..=5 {
        let event = Event::builder()
            .event_type("api_call")
            .user_id("user-123")
            .timestamp(Utc::now())
            .metric("tokens", 1000 * i)
            .metric("duration_ms", 200 + i * 50)
            .tag("model", "claude-sonnet")
            .tag("endpoint", "/v1/chat")
            .build()?;

        meter.record(event).await?;
        println!("✓ Recorded API call #{}", i);
    }

    println!("\n📊 Usage Statistics:");

    // Query usage
    let stats = meter.query().user_id("user-123").execute_stats().await?;

    println!("  Total events: {}", stats.total_events);
    println!(
        "  Total tokens: {:.0}",
        stats.get_metric_sum("tokens").unwrap_or(0.0)
    );
    println!(
        "  Avg duration: {:.0}ms",
        stats.get_metric_average("duration_ms").unwrap_or(0.0)
    );

    Ok(())
}
