//! Custom aggregations example
//!
//! Demonstrates advanced aggregation features.

use usemeter::{Event, Meter, StorageBackend, Aggregation, TimeWindow, AggregationFn};
use usemeter::storage::SqliteBackend;
use chrono::{Utc, Duration};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = SqliteBackend::new_in_memory()?;
    let meter = Meter::new(storage);
    meter.initialize().await?;

    println!("📈 Custom Aggregations Example\n");

    // Record events over time
    let now = Utc::now();
    for hour in 0..24 {
        for i in 0..10 {
            let event = Event::builder()
                .event_type("api_call")
                .user_id("user-123")
                .timestamp(now - Duration::hours(24 - hour))
                .metric("tokens", 500 + i * 100)
                .metric("latency_ms", 100 + (hour % 12) * 50)
                .build()?;

            meter.record(event).await?;
        }
    }

    println!("✓ Recorded 240 events over 24 hours\n");

    // Aggregate by hour
    println!("📊 Hourly Token Usage:");
    let mut hourly_tokens: HashMap<String, f64> = HashMap::new();

    let events = meter.query()
        .user_id("user-123")
        .start_time(now - Duration::hours(24))
        .execute_events()
        .await?;

    for event in events {
        let hour_key = event.timestamp.format("%Y-%m-%d %H:00").to_string();
        *hourly_tokens.entry(hour_key).or_insert(0.0) += event.get_metric("tokens")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
    }

    // Show last 6 hours
    let mut sorted_hours: Vec<_> = hourly_tokens.into_iter().collect();
    sorted_hours.sort_by(|a, b| a.0.cmp(&b.0));

    for (hour, tokens) in sorted_hours.iter().take(6) {
        println!("  {}: {:.0} tokens", hour, tokens);
    }

    Ok(())
}
