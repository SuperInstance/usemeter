//! Main meter API for tracking usage

use crate::{
    alert::AlertManager,
    billing::{
        BillingEngine, CostCalculation, Invoice, PricingRule, Report, ReportData, ReportType,
    },
    event::Event,
    query::{QueryBuilder, UsageStats},
    storage::StorageBackend,
};
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// Main meter for tracking usage
///
/// The Meter is the primary API for usemeter. It handles:
/// - Recording usage events
/// - Querying usage data
/// - Calculating costs
/// - Generating reports and invoices
/// - Monitoring alerts
///
/// # Example
///
/// ```rust,no_run
/// use usemeter::{Meter, Event, StorageBackend};
/// use usemeter::storage::SqliteBackend;
/// use chrono::Utc;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let storage = SqliteBackend::new_in_memory()?;
/// storage.initialize().await?;
///
/// let meter = Meter::new(storage);
///
/// let event = Event::builder()
///     .event_type("api_call")
///     .user_id("user-123")
///     .timestamp(Utc::now())
///     .metric("tokens", 1000)
///     .build()?;
///
/// meter.record(event).await?;
/// # Ok(())
/// # }
/// ```
pub struct Meter<S: StorageBackend> {
    storage: Arc<S>,
    billing: Arc<BillingEngine>,
    alerts: Arc<tokio::sync::Mutex<AlertManager>>,
}

impl<S: StorageBackend> Meter<S> {
    /// Create a new meter with the given storage backend
    pub fn new(storage: S) -> Self {
        Self {
            storage: Arc::new(storage),
            billing: Arc::new(BillingEngine::new()),
            alerts: Arc::new(tokio::sync::Mutex::new(AlertManager::new())),
        }
    }

    /// Initialize the meter (calls storage initialization)
    pub async fn initialize(&self) -> Result<(), Error> {
        self.storage
            .initialize()
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;
        Ok(())
    }

    /// Record a usage event
    pub async fn record(&self, event: Event) -> Result<(), Error> {
        self.storage
            .store(&event)
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        tracing::debug!("Recorded event: {}", event.id);
        Ok(())
    }

    /// Record multiple usage events in a batch
    pub async fn record_batch(&self, events: Vec<Event>) -> Result<(), Error> {
        self.storage
            .store_batch(&events)
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        tracing::debug!("Recorded batch of {} events", events.len());
        Ok(())
    }

    /// Create a new query
    pub fn query(&self) -> QueryBuilder {
        QueryBuilder::default()
    }

    /// Execute a query and return events
    pub async fn query_events(&self, builder: &QueryBuilder) -> Result<Vec<Event>, Error> {
        self.storage
            .query(builder)
            .await
            .map_err(|e| Error::Storage(e.to_string()))
    }

    /// Execute a query and return statistics
    pub async fn query_stats(&self, builder: &QueryBuilder) -> Result<UsageStats, Error> {
        self.storage
            .query_stats(builder)
            .await
            .map_err(|e| Error::Storage(e.to_string()))
    }

    /// Get billing engine reference
    pub fn billing(&self) -> &BillingEngine {
        &self.billing
    }

    /// Add a pricing rule
    pub async fn add_pricing_rule(&self, rule: PricingRule) {
        let mut billing = Arc::clone(&self.billing);
        // Note: This would need interior mutability. For now, we'll require mutable reference.
        // In production, you'd use RwLock or similar.
        tracing::warn!("Adding pricing rules requires mutable access to billing engine");
    }

    /// Calculate cost for a time period
    pub async fn calculate_cost(
        &self,
        user_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        rule_names: &[String],
    ) -> Result<CostCalculation, Error> {
        let query = QueryBuilder::default()
            .user_id(user_id)
            .start_time(start)
            .end_time(end);

        let stats = self.query_stats(&query).await?;
        let billing = &*self.billing;

        billing
            .calculate_cost(&stats, rule_names)
            .map_err(|e| Error::Billing(e.to_string()))
    }

    /// Generate an invoice for a time period
    pub async fn generate_invoice(
        &self,
        user_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        rule_names: &[String],
    ) -> Result<Invoice, Error> {
        let cost = self.calculate_cost(user_id, start, end, rule_names).await?;

        Ok(Invoice::new(user_id, start, end, cost))
    }

    /// Generate a usage report
    pub async fn generate_report(
        &self,
        user_id: Option<&str>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        rule_names: &[String],
    ) -> Result<Report, Error> {
        let query = if let Some(uid) = user_id {
            QueryBuilder::default()
                .user_id(uid)
                .start_time(start)
                .end_time(end)
        } else {
            QueryBuilder::default().start_time(start).end_time(end)
        };

        let stats = self.query_stats(&query).await?;

        // Calculate cost
        let cost = self
            .billing
            .calculate_cost(&stats, rule_names)
            .map_err(|e| Error::Billing(e.to_string()))?;

        let data = ReportData {
            period_start: start,
            period_end: end,
            user_id: user_id.map(|s| s.to_string()),
            total_events: stats.total_events,
            total_cost_cents: cost.total_cents,
            metrics: stats.metric_sums,
        };

        Ok(Report::new(ReportType::Usage, data))
    }

    /// Get alert manager
    pub async fn alerts(&self) -> tokio::sync::MutexGuard<'_, AlertManager> {
        self.alerts.lock().await
    }

    /// Evaluate alert rules
    pub async fn evaluate_alerts(
        &self,
        user_id: Option<String>,
        metrics: &std::collections::HashMap<String, f64>,
    ) -> Result<Vec<crate::alert::Alert>, Error> {
        let mut manager = self.alerts.lock().await;
        Ok(manager.evaluate(user_id, metrics))
    }

    /// Delete events older than the given timestamp
    pub async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<u64, Error> {
        self.storage
            .delete_before(timestamp)
            .await
            .map_err(|e| Error::Storage(e.to_string()))
    }

    /// Get total event count
    pub async fn count(&self) -> Result<u64, Error> {
        self.storage
            .count()
            .await
            .map_err(|e| Error::Storage(e.to_string()))
    }
}

/// Error type for Meter operations
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Billing error: {0}")]
    Billing(String),

    #[error("Query error: {0}")]
    Query(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::SqliteBackend;

    #[tokio::test]
    async fn test_meter_record_and_query() {
        let storage = SqliteBackend::new_in_memory().unwrap();
        storage.initialize().await.unwrap();

        let meter = Meter::new(storage);
        meter.initialize().await.unwrap();

        // Record an event
        let event = Event::builder()
            .event_type("api_call")
            .user_id("user-123")
            .metric("tokens", 1000)
            .build()
            .unwrap();

        meter.record(event).await.unwrap();

        // Query events
        let query = QueryBuilder::default().user_id("user-123");
        let events = meter.query_events(&query).await.unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].user_id, "user-123");
    }

    #[tokio::test]
    async fn test_meter_stats() {
        let storage = SqliteBackend::new_in_memory().unwrap();
        storage.initialize().await.unwrap();

        let meter = Meter::new(storage);
        meter.initialize().await.unwrap();

        // Record multiple events
        for i in 0..5 {
            let event = Event::builder()
                .event_type("api_call")
                .user_id("user-123")
                .metric("tokens", (i + 1) * 100)
                .build()
                .unwrap();

            meter.record(event).await.unwrap();
        }

        // Query stats
        let query = QueryBuilder::default().user_id("user-123");
        let stats = meter.query_stats(&query).await.unwrap();

        assert_eq!(stats.total_events, 5);
        assert_eq!(stats.get_metric_sum("tokens"), Some(1500.0));
    }
}
