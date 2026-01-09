//! # usemeter - Usage Tracking, Metering, and Billing Engine
//!
//! A flexible, high-performance usage tracking and billing system for applications.
//!
//! ## Features
//!
//! - **Event Tracking**: High-volume event ingestion with flexible schemas
//! - **Meter Configuration**: Define what to measure and how to aggregate
//! - **Time Windows**: Support for hourly, daily, monthly aggregations
//! - **Aggregation Functions**: Sum, average, max, min, count, percentile
//! - **Cost Calculation**: Flexible pricing models and billing rules
//! - **Multiple Storage Backends**: SQLite, PostgreSQL, MySQL, or file-based
//! - **Report Generation**: CSV, JSON, and programmatic reports
//! - **Alerting**: Budget thresholds and unusual usage detection
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use usemeter::{Meter, Event, StorageBackend};
//! use usemeter::storage::SqliteBackend;
//! use chrono::Utc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create meter with SQLite storage
//!     let storage = SqliteBackend::new_in_memory()?;
//!     let meter = Meter::new(storage);
//!     meter.initialize().await?;
//!
//!     // Record usage events
//!     let event = Event::builder()
//!         .event_type("api_call")
//!         .user_id("user-123")
//!         .timestamp(Utc::now())
//!         .metric("tokens", 1000)
//!         .metric("duration_ms", 250)
//!         .build()?;
//!
//!     meter.record(event).await?;
//!
//!     // Query usage
//!     let stats = meter.query()
//!         .user_id("user-123")
//!         .start_time(Utc::now() - chrono::Duration::hours(24))
//!         .execute_stats()
//!         .await?;
//!
//!     println!("Total tokens: {:?}", stats.get_metric_sum("tokens"));
//!     Ok(())
//! }
//! ```

pub mod aggregation;
pub mod alert;
pub mod billing;
pub mod event;
pub mod meter;
pub mod query;
pub mod storage;

pub use aggregation::{Aggregation, AggregationFn, TimeWindow};
pub use alert::{Alert, AlertManager, AlertRule, AlertThreshold};
pub use billing::{BillingEngine, CostCalculation, Invoice, PricingRule, Report};
pub use event::{Event, EventBuilder, MetricValue};
pub use meter::{Error, Meter};
pub use query::{QueryBuilder, QueryError, UsageStats};
pub use storage::{FileBackend, SqliteBackend, StorageBackend};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::aggregation::{Aggregation, TimeWindow};
    pub use crate::billing::{BillingEngine, PricingRule};
    pub use crate::query::{QueryBuilder, UsageStats};
    pub use crate::storage::{SqliteBackend, StorageBackend};
    pub use crate::{Error, Event, EventBuilder, Meter, MetricValue};
}

/// usemeter error types
pub mod errors {
    pub use crate::alert::AlertError;
    pub use crate::billing::BillingError;
    pub use crate::event::EventError;
    pub use crate::query::QueryError;
    pub use crate::storage::StorageError;
}
