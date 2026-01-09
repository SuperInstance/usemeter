//! Report generation example
//!
//! Demonstrates creating and exporting usage reports.

use chrono::{Duration, Utc};
use usemeter::storage::SqliteBackend;
use usemeter::{Event, Meter, Report, StorageBackend};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = SqliteBackend::new_in_memory()?;
    let meter = Meter::new(storage);
    meter.initialize().await?;

    println!("📊 Report Generation Example\n");

    // Generate sample data
    let now = Utc::now();
    for user in 1..=5 {
        for i in 0..20 {
            let event = Event::builder()
                .event_type("api_call")
                .user_id(&format!("user-{}", user))
                .timestamp(now - Duration::hours(24 * 7))
                .metric("tokens", 1000 + i * 100)
                .metric("duration_ms", 200 + i * 10)
                .tag(
                    "model",
                    if user % 2 == 0 {
                        "claude-sonnet"
                    } else {
                        "claude-opus"
                    },
                )
                .build()?;

            meter.record(event).await?;
        }
    }

    println!("✓ Generated sample data for 5 users over 7 days\n");

    // Generate usage report
    let report = meter
        .generate_report(
            None, // All users
            now - Duration::days(7),
            now,
            &[], // No pricing rules for this example
        )
        .await?;

    println!("📈 Usage Report:");
    println!("  Report ID: {}", report.id);
    println!(
        "  Period: {} to {}",
        report.data.period_start.format("%Y-%m-%d"),
        report.data.period_end.format("%Y-%m-%d")
    );
    println!("  Total events: {}", report.data.total_events);
    println!("  Total users: {}", 5);

    // Per-user breakdown
    println!("\n👥 Per-User Breakdown:");
    for user in 1..=5 {
        let user_id = format!("user-{}", user);
        let stats = meter
            .query()
            .user_id(&user_id)
            .start_time(now - Duration::days(7))
            .execute_stats()
            .await?;

        let tokens = stats.get_metric_sum("tokens").unwrap_or(0.0);
        let avg_duration = stats.get_metric_average("duration_ms").unwrap_or(0.0);

        println!(
            "  {}: {:.0} tokens, {:.0}ms avg duration",
            user_id, tokens, avg_duration
        );
    }

    // Export to JSON
    let json = report.to_json()?;
    println!("\n📄 Report exported as JSON ({} bytes)", json.len());

    // Export to CSV (if feature enabled)
    #[cfg(feature = "csv-reports")]
    {
        let csv = report.to_csv()?;
        println!("📄 Report exported as CSV ({} bytes)", csv.len());
    }

    Ok(())
}
