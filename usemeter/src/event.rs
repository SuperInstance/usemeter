//! Event types and builders for usage tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Error types for event creation and validation
#[derive(Error, Debug)]
pub enum EventError {
    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid metric value: {0}")]
    InvalidMetric(String),

    #[error("Event validation failed: {0}")]
    ValidationFailed(String),
}

/// A usage event representing a single measurable action
///
/// Events are the core unit of usage tracking. They can represent
/// API calls, compute operations, storage operations, or any other
/// billable or trackable action.
///
/// # Example
///
/// ```rust
/// use usemeter::{Event, EventBuilder, MetricValue};
/// use chrono::Utc;
///
/// let event = Event::builder()
///     .event_type("api_call")
///     .user_id("user-123")
///     .timestamp(Utc::now())
///     .metric("tokens", 1000)
///     .metric("duration_ms", 250)
///     .tag("model", "claude-sonnet")
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique event identifier
    pub id: String,

    /// Type of event (e.g., "api_call", "storage_read", "compute")
    pub event_type: String,

    /// User or account ID
    pub user_id: String,

    /// Optional resource ID (e.g., specific model, database, etc.)
    pub resource_id: Option<String>,

    /// When the event occurred
    pub timestamp: DateTime<Utc>,

    /// Metrics (quantitative measurements)
    pub metrics: HashMap<String, MetricValue>,

    /// Tags (categorical dimensions for grouping)
    pub tags: HashMap<String, String>,

    /// Optional custom properties
    pub properties: HashMap<String, serde_json::Value>,
}

impl Event {
    /// Create a new event builder
    pub fn builder() -> EventBuilder {
        EventBuilder::default()
    }

    /// Get a metric value by name
    pub fn get_metric(&self, name: &str) -> Option<&MetricValue> {
        self.metrics.get(name)
    }

    /// Get a tag value by name
    pub fn get_tag(&self, name: &str) -> Option<&String> {
        self.tags.get(name)
    }

    /// Calculate a derived metric from existing metrics
    ///
    /// # Example
    ///
    /// ```rust
    /// # use usemeter::{Event, EventBuilder, MetricValue};
    /// # use chrono::Utc;
    /// # let event = Event::builder()
    /// #     .event_type("test")
    /// #     .user_id("test")
    /// #     .timestamp(Utc::now())
    /// #     .metric("input_tokens", 1000)
    /// #     .metric("output_tokens", 500)
    /// #     .build()
    /// #     .unwrap();
    /// // Calculate total tokens
    /// let total = event.derive_metric("total_tokens", |metrics| {
    ///     let input = metrics.get("input_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
    ///     let output = metrics.get("output_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
    ///     MetricValue::Integer(input + output)
    /// });
    /// ```
    pub fn derive_metric<F>(&self, name: &str, f: F) -> Option<MetricValue>
    where
        F: FnOnce(&HashMap<String, MetricValue>) -> MetricValue,
    {
        Some(f(&self.metrics))
    }
}

/// Builder for creating Event instances
#[derive(Default)]
pub struct EventBuilder {
    id: Option<String>,
    event_type: Option<String>,
    user_id: Option<String>,
    resource_id: Option<String>,
    timestamp: Option<DateTime<Utc>>,
    metrics: HashMap<String, MetricValue>,
    tags: HashMap<String, String>,
    properties: HashMap<String, serde_json::Value>,
}

impl EventBuilder {
    /// Set the event ID (will generate UUID if not set)
    pub fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    /// Set the event type (required)
    pub fn event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = Some(event_type.into());
        self
    }

    /// Set the user ID (required)
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set the resource ID (optional)
    pub fn resource_id(mut self, resource_id: impl Into<String>) -> Self {
        self.resource_id = Some(resource_id.into());
        self
    }

    /// Set the timestamp (defaults to UTC now if not set)
    pub fn timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Add a metric
    pub fn metric(mut self, name: impl Into<String>, value: impl Into<MetricValue>) -> Self {
        self.metrics.insert(name.into(), value.into());
        self
    }

    /// Add multiple metrics
    pub fn metrics(mut self, metrics: HashMap<String, MetricValue>) -> Self {
        self.metrics.extend(metrics);
        self
    }

    /// Add a tag
    pub fn tag(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(name.into(), value.into());
        self
    }

    /// Add multiple tags
    pub fn tags(mut self, tags: HashMap<String, String>) -> Self {
        self.tags.extend(tags);
        self
    }

    /// Add a custom property
    pub fn property(mut self, name: impl Into<String>, value: serde_json::Value) -> Self {
        self.properties.insert(name.into(), value);
        self
    }

    /// Build the event
    ///
    /// # Errors
    ///
    /// Returns error if required fields (event_type, user_id) are missing.
    pub fn build(self) -> Result<Event, EventError> {
        let event_type = self
            .event_type
            .ok_or_else(|| EventError::MissingField("event_type".to_string()))?;

        let user_id = self
            .user_id
            .ok_or_else(|| EventError::MissingField("user_id".to_string()))?;

        Ok(Event {
            id: self.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            event_type,
            user_id,
            resource_id: self.resource_id,
            timestamp: self.timestamp.unwrap_or_else(Utc::now),
            metrics: self.metrics,
            tags: self.tags,
            properties: self.properties,
        })
    }
}

/// Metric value types
///
/// Metrics can be integers (for counts), floats (for durations or rates),
/// or strings (for categorical metrics).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum MetricValue {
    /// Integer metric (e.g., token count, request count)
    Integer(i64),

    /// Float metric (e.g., duration in milliseconds, cost in cents)
    Float(f64),

    /// String metric (e.g., status code, error type)
    String(String),
}

impl MetricValue {
    /// Convert to i64 if possible
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            MetricValue::Integer(i) => Some(*i),
            MetricValue::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    /// Convert to f64 if possible
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            MetricValue::Integer(i) => Some(*i as f64),
            MetricValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Convert to string reference
    pub fn as_str(&self) -> Option<&str> {
        match self {
            MetricValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Check if metric is zero
    pub fn is_zero(&self) -> bool {
        match self {
            MetricValue::Integer(i) => *i == 0,
            MetricValue::Float(f) => *f == 0.0,
            _ => false,
        }
    }
}

// Implement conversions from common types
impl From<i64> for MetricValue {
    fn from(value: i64) -> Self {
        MetricValue::Integer(value)
    }
}

impl From<i32> for MetricValue {
    fn from(value: i32) -> Self {
        MetricValue::Integer(value as i64)
    }
}

impl From<u64> for MetricValue {
    fn from(value: u64) -> Self {
        MetricValue::Integer(value as i64)
    }
}

impl From<u32> for MetricValue {
    fn from(value: u32) -> Self {
        MetricValue::Integer(value as i64)
    }
}

impl From<f64> for MetricValue {
    fn from(value: f64) -> Self {
        MetricValue::Float(value)
    }
}

impl From<f32> for MetricValue {
    fn from(value: f32) -> Self {
        MetricValue::Float(value as f64)
    }
}

impl From<String> for MetricValue {
    fn from(value: String) -> Self {
        MetricValue::String(value)
    }
}

impl From<&str> for MetricValue {
    fn from(value: &str) -> Self {
        MetricValue::String(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_builder() {
        let event = Event::builder()
            .event_type("api_call")
            .user_id("user-123")
            .metric("tokens", 1000)
            .tag("model", "claude-sonnet")
            .build()
            .unwrap();

        assert_eq!(event.event_type, "api_call");
        assert_eq!(event.user_id, "user-123");
        assert_eq!(event.get_metric("tokens"), Some(&MetricValue::Integer(1000)));
        assert_eq!(event.get_tag("model"), Some(&"claude-sonnet".to_string()));
    }

    #[test]
    fn test_event_builder_missing_fields() {
        let result = Event::builder()
            .event_type("api_call")
            // Missing user_id
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_metric_value_conversions() {
        let int_val = MetricValue::Integer(100);
        assert_eq!(int_val.as_i64(), Some(100));
        assert_eq!(int_val.as_f64(), Some(100.0));

        let float_val = MetricValue::Float(99.99);
        assert_eq!(float_val.as_f64(), Some(99.99));
        assert_eq!(float_val.as_i64(), Some(99));

        let string_val = MetricValue::String("test".to_string());
        assert_eq!(string_val.as_str(), Some("test"));
        assert_eq!(string_val.as_i64(), None);
    }

    #[test]
    fn test_derive_metric() {
        let event = Event::builder()
            .event_type("test")
            .user_id("user-1")
            .metric("input_tokens", 1000)
            .metric("output_tokens", 500)
            .build()
            .unwrap();

        let total = event.derive_metric("total_tokens", |metrics| {
            let input = metrics.get("input_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
            let output = metrics.get("output_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
            MetricValue::Integer(input + output)
        });

        assert_eq!(total, Some(MetricValue::Integer(1500)));
    }
}
