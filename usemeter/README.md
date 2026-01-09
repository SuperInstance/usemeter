# usemeter

[![crates.io](https://img.shields.io/crates/v/usemeter)](https://crates.io/crates/usemeter)
[![Documentation](https://docs.rs/usemeter/badge.svg)](https://docs.rs/usemeter)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![GitHub Actions](https://github.com/SuperInstance/usemeter/workflows/CI/badge.svg)](https://github.com/SuperInstance/usemeter/actions)

**Flexible, high-performance usage tracking, metering, and billing engine for applications.**

## ⚡ Quick Start

Track API calls, calculate costs, generate invoices, and monitor budgets in seconds:

```rust
use usemeter::{Meter, Event, StorageBackend};
use usemeter::storage::SqliteBackend;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create meter with SQLite storage
    let storage = SqliteBackend::new_in_memory()?;
    let meter = Meter::new(storage);
    meter.initialize().await?;

    // Record usage events
    let event = Event::builder()
        .event_type("api_call")
        .user_id("user-123")
        .timestamp(Utc::now())
        .metric("tokens", 1000)
        .metric("duration_ms", 250)
        .build()?;

    meter.record(event).await?;

    // Query usage
    let stats = meter.query()
        .user_id("user-123")
        .start_time(Utc::now() - chrono::Duration::hours(24))
        .execute_stats()
        .await?;

    println!("Total tokens: {:?}", stats.get_metric_sum("tokens"));
    Ok(())
}
```

## ✨ Features

- **📊 Event Tracking** - High-volume event ingestion with flexible schemas
- **🔧 Meter Configuration** - Define what to measure and how to aggregate
- **⏱️ Time Windows** - Support for hourly, daily, monthly aggregations
- **📈 Aggregation Functions** - Sum, average, max, min, count, percentiles
- **💰 Cost Calculation** - Flexible pricing models and billing rules
- **🗄️ Multiple Storage Backends** - SQLite, PostgreSQL, MySQL, file-based
- **📄 Report Generation** - CSV, JSON, and programmatic reports
- **🚨 Alerting** - Budget thresholds and unusual usage detection

## 🎯 Use Cases

- **API Metering** - Track API calls, tokens, bandwidth
- **Cost Allocation** - Calculate costs per team/user/project
- **Billing Systems** - Generate invoices and usage reports
- **Budget Monitoring** - Alert on spending thresholds
- **Usage Analytics** - Analyze usage patterns over time

## 📦 Installation

```toml
[dependencies]
usemeter = "0.1"
```

Enable features:

```toml
[dependencies]
usemeter = { version = "0.1", features = ["sqlite", "csv-reports"] }
```

## 🚀 Getting Started

### Basic Usage Tracking

```rust
use usemeter::{Event, Meter, StorageBackend};
use usemeter::storage::SqliteBackend;

let storage = SqliteBackend::new_in_memory()?;
let meter = Meter::new(storage);
meter.initialize().await?;

// Record events
let event = Event::builder()
    .event_type("api_call")
    .user_id("user-123")
    .metric("tokens", 1000)
    .tag("model", "claude-sonnet")
    .build()?;

meter.record(event).await?;
```

### Cost Calculation & Billing

```rust
use usemeter::{PricingRule, BillingEngine};

// Setup pricing
let mut billing = BillingEngine::new();
billing.add_rule(PricingRule::new(
    "tokens_pricing",
    "tokens",
    0.002,  // $0.00002 per token
    1.0,
));

// Calculate cost
let cost = billing.calculate_cost(
    &stats,
    &["tokens_pricing".to_string()],
)?;

println!("Total cost: ${:.2}", cost.total_cents / 100.0);
```

### Invoice Generation

```rust
let invoice = meter.generate_invoice(
    "user-123",
    start_date,
    end_date,
    &["tokens_pricing".to_string()],
).await?;

println!("Invoice {} - ${:.2}", invoice.id, invoice.total_dollars());
```

### Alert Rules

```rust
use usemeter::{AlertRule, AlertManager};

let mut alerts = AlertManager::new();
alerts.add_rule(AlertRule::budget_alert(
    "monthly_budget",
    "cost_cents",
    10000.0,  // $100
));

// Check for alerts
let triggered = alerts.evaluate(
    Some("user-123".to_string()),
    &metrics,
);
```

## 📚 Examples

- [`basic_metering.rs`](examples/basic_metering.rs) - Basic event tracking and querying
- [`custom_aggregations.rs`](examples/custom_aggregations.rs) - Advanced aggregation features
- [`billing_calculation.rs`](examples/billing_calculation.rs) - Cost calculation and invoicing
- [`with_privox.rs`](examples/with_privox.rs) - Integration with privox
- [`reports.rs`](examples/reports.rs) - Report generation

Run examples:

```bash
cargo run --example basic_metering
cargo run --example billing_calculation
```

## 🔧 Configuration

### Storage Backends

**SQLite (default, embedded):**
```rust
let storage = SqliteBackend::new_in_memory()?;
let storage = SqliteBackend::new("/path/to/events.db")?;
```

**File-based (simple):**
```rust
let storage = FileBackend::new("/path/to/events.jsonl")?;
```

### Pricing Models

**Simple linear pricing:**
```rust
let rule = PricingRule::new("tokens", "tokens", 0.002, 1.0);
```

**Tiered pricing:**
```rust
let tiers = vec![
    PricingTier {
        min_quantity: 0.0,
        max_quantity: 1000.0,
        unit_price_cents: 1.0,
        unit_quantity: 1.0,
    },
    PricingTier {
        min_quantity: 1000.0,
        max_quantity: 10000.0,
        unit_price_cents: 0.5,
        unit_quantity: 1.0,
    },
];
let rule = PricingRule::with_tiers("tiered_tokens", "tokens", tiers);
```

## 📖 Documentation

- [Documentation](https://docs.rs/usemeter)
- [Examples](examples/)
- [Contributing](CONTRIBUTING.md)

## 🔗 Integrations

usemeter works great with:

- **[privox](https://github.com/SuperInstance/privox)** - Privacy redaction engine
- **[tripartite-rs](https://github.com/SuperInstance/tripartite-rs)** - Multi-agent consensus
- **[knowledge-vault](https://github.com/SuperInstance/knowledge-vault)** - Vector database RAG

## 📊 Performance

- **Event ingestion**: 10,000+ events/second
- **Query performance**: Millisecond response times
- **Storage efficiency**: Compressed JSON storage
- **Memory usage**: <10MB for typical workloads

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## 📄 License

Licensed under either of:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE/LICENSE-2.0)

at your option.

## 🙏 Acknowledgments

Built with ❤️ for the [SuperInstance](https://superinstance.ai) ecosystem.

---

**Note**: usemeter is part of the SuperInstance ecosystem. Check out the other tools!
