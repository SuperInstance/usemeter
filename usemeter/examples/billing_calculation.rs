//! Billing calculation example
//!
//! Demonstrates cost calculation and invoicing.

use chrono::{Duration, Utc};
use usemeter::storage::SqliteBackend;
use usemeter::{BillingEngine, Event, Meter, PricingRule, StorageBackend};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = SqliteBackend::new_in_memory()?;
    let meter = Meter::new(storage);
    meter.initialize().await?;

    println!("💰 Billing Calculation Example\n");

    // Setup pricing rules
    let mut billing = BillingEngine::new();
    billing.add_rule(PricingRule::new(
        "tokens_pricing",
        "tokens",
        0.002, // $0.00002 per token (2 cents per 1000 tokens)
        1.0,
    ));
    billing.add_rule(PricingRule::new(
        "api_calls",
        "api_calls",
        0.1, // 0.1 cents per API call
        1.0,
    ));

    println!("✓ Pricing rules configured\n");

    // Simulate usage
    let now = Utc::now();
    for i in 0..100 {
        let event = Event::builder()
            .event_type("api_call")
            .user_id("user-123")
            .timestamp(now - Duration::hours(24))
            .metric("tokens", 1000 + i * 10)
            .metric("api_calls", 1)
            .build()?;

        meter.record(event).await?;
    }

    println!("✓ Recorded 100 API calls\n");

    // Calculate cost for the period
    let start = now - Duration::hours(24);
    let end = now;

    let cost = meter.billing().calculate_cost(
        &meter
            .query()
            .user_id("user-123")
            .start_time(start)
            .end_time(end)
            .execute_stats()
            .await?,
        &["tokens_pricing".to_string(), "api_calls".to_string()],
    )?;

    println!("📊 Cost Breakdown:");
    println!("  Total Cost: ${:.2}", cost.total_cents / 100.0);
    for (metric, metric_cost) in &cost.breakdown {
        println!(
            "    - {}: {:.0} units × ${:.4}/unit = ${:.2}",
            metric,
            metric_cost.value,
            metric_cost.cost_cents / metric_cost.value / 100.0,
            metric_cost.cost_cents / 100.0
        );
    }

    // Generate invoice
    let invoice = meter
        .generate_invoice(
            "user-123",
            start,
            end,
            &["tokens_pricing".to_string(), "api_calls".to_string()],
        )
        .await?;

    println!("\n📄 Invoice Generated:");
    println!("  Invoice ID: {}", invoice.id);
    println!(
        "  Period: {} to {}",
        invoice.period_start.format("%Y-%m-%d %H:%M"),
        invoice.period_end.format("%Y-%m-%d %H:%M")
    );
    println!("  Total Due: ${:.2}", invoice.total_dollars());

    Ok(())
}
