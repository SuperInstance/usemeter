//! Query builder and usage statistics

use crate::event::MetricValue;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Query error types
#[derive(Error, Debug)]
pub enum QueryError {
    #[error("Invalid query parameter: {0}")]
    InvalidParameter(String),

    #[error("Query execution failed: {0}")]
    ExecutionFailed(String),
}

/// Query builder for filtering usage events
///
/// # Example
///
/// ```rust,no_run
/// use usemeter::{Meter, Query};
/// use chrono::{Utc, Duration};
///
/// # async fn example(meter: Meter) -> Result<(), Box<dyn std::error::Error>> {
/// let stats = meter.query()
///     .user_id("user-123")
///     .event_type("api_call")
///     .start_time(Utc::now() - Duration::hours(24))
///     .execute()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct QueryBuilder {
    user_id: Option<String>,
    event_type: Option<String>,
    resource_id: Option<String>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    limit: Option<usize>,
}

impl QueryBuilder {
    /// Filter by user ID
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Filter by event type
    pub fn event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = Some(event_type.into());
        self
    }

    /// Filter by resource ID
    pub fn resource_id(mut self, resource_id: impl Into<String>) -> Self {
        self.resource_id = Some(resource_id.into());
        self
    }

    /// Filter by start time (inclusive)
    pub fn start_time(mut self, start: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self
    }

    /// Filter by end time (inclusive)
    pub fn end_time(mut self, end: DateTime<Utc>) -> Self {
        self.end_time = Some(end);
        self
    }

    /// Set maximum number of results
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Execute query and return events
    pub async fn execute(&self) -> Result<Vec<crate::event::Event>, QueryError> {
        // This is a placeholder - the actual execution happens in the storage backend
        Err(QueryError::ExecutionFailed("Use Meter::query_events instead".to_string()))
    }

    /// Execute query and return events
    pub async fn execute_events(&self) -> Result<Vec<crate::event::Event>, QueryError> {
        // This is a placeholder - the actual execution happens in the storage backend
        Err(QueryError::ExecutionFailed("Use Meter::query_events instead".to_string()))
    }

    /// Execute query and return statistics
    pub async fn execute_stats(&self) -> Result<UsageStats, QueryError> {
        // This is a placeholder - the actual execution happens in the storage backend
        Err(QueryError::ExecutionFailed("Use Meter::query_stats instead".to_string()))
    }
}

/// Usage statistics from a query
///
/// Contains aggregated metrics and counts for the queried events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    /// Total number of events matching the query
    pub total_events: u64,

    /// Sum of each metric across all events
    pub metric_sums: HashMap<String, f64>,

    /// Count of each metric (number of events with that metric)
    pub metric_counts: HashMap<String, u64>,

    /// Query time range (if specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Utc>>,
}

impl UsageStats {
    /// Create empty stats
    pub fn empty() -> Self {
        Self {
            total_events: 0,
            metric_sums: HashMap::new(),
            metric_counts: HashMap::new(),
            start_time: None,
            end_time: None,
        }
    }

    /// Get the sum of a metric by name
    pub fn get_metric_sum(&self, name: &str) -> Option<f64> {
        self.metric_sums.get(name).copied()
    }

    /// Get the count of a metric by name
    pub fn get_metric_count(&self, name: &str) -> Option<u64> {
        self.metric_counts.get(name).copied()
    }

    /// Calculate the average of a metric
    pub fn get_metric_average(&self, name: &str) -> Option<f64> {
        let sum = self.get_metric_sum(name)?;
        let count = self.get_metric_count(name)?;
        if count == 0 {
            None
        } else {
            Some(sum / count as f64)
        }
    }

    /// Get all metric names
    pub fn metric_names(&self) -> impl Iterator<Item = &String> {
        self.metric_sums.keys()
    }
}

impl Default for UsageStats {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder() {
        let builder = QueryBuilder::default()
            .user_id("user-123")
            .event_type("api_call");

        assert_eq!(builder.user_id, Some("user-123".to_string()));
        assert_eq!(builder.event_type, Some("api_call".to_string()));
    }

    #[test]
    fn test_usage_stats_empty() {
        let stats = UsageStats::empty();
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.get_metric_sum("tokens"), None);
    }

    #[test]
    fn test_usage_stats_calculations() {
        let mut stats = UsageStats::empty();
        stats.total_events = 10;
        stats.metric_sums.insert("tokens".to_string(), 1000.0);
        stats.metric_counts.insert("tokens".to_string(), 10);

        assert_eq!(stats.get_metric_sum("tokens"), Some(1000.0));
        assert_eq!(stats.get_metric_count("tokens"), Some(10));
        assert_eq!(stats.get_metric_average("tokens"), Some(100.0));
    }
}
