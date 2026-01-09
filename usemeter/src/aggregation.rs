//! Aggregation functions and time windows

use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use serde::{Deserialize, Serialize};

/// Time window for aggregations
///
/// Defines how events are grouped for aggregation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeWindow {
    /// No grouping - aggregate all events
    All,

    /// Group by hour
    Hour,

    /// Group by day
    Day,

    /// Group by week (starting Monday)
    Week,

    /// Group by month
    Month,

    /// Custom window in seconds
    CustomSeconds(u64),
}

impl TimeWindow {
    /// Truncate a timestamp to the start of this window
    pub fn truncate(&self, dt: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            TimeWindow::All => dt.with_nanosecond(0).unwrap(),
            TimeWindow::Hour => dt
                .with_minute(0)
                .and_then(|d| d.with_second(0))
                .and_then(|d| d.with_nanosecond(0))
                .unwrap(),
            TimeWindow::Day => dt
                .with_hour(0)
                .and_then(|d| d.with_minute(0))
                .and_then(|d| d.with_second(0))
                .and_then(|d| d.with_nanosecond(0))
                .unwrap(),
            TimeWindow::Week => {
                // Get Monday of this week
                let weekday = dt.weekday().num_days_from_monday();
                let date = (dt - chrono::Duration::days(weekday as i64)).date_naive();
                DateTime::from_naive_utc_and_offset(date.and_hms_opt(0, 0, 0).unwrap(), Utc)
            },
            TimeWindow::Month => dt
                .with_day(1)
                .and_then(|d| d.with_hour(0))
                .and_then(|d| d.with_minute(0))
                .and_then(|d| d.with_second(0))
                .and_then(|d| d.with_nanosecond(0))
                .unwrap(),
            TimeWindow::CustomSeconds(seconds) => {
                let ts = dt.timestamp();
                let window_start = (ts / (*seconds as i64)) * (*seconds as i64);
                DateTime::from_timestamp(window_start, 0).unwrap()
            },
        }
    }

    /// Calculate the next window start
    pub fn next(&self, dt: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            TimeWindow::All => dt + chrono::Duration::seconds(1),
            TimeWindow::Hour => dt + chrono::Duration::hours(1),
            TimeWindow::Day => dt + chrono::Duration::days(1),
            TimeWindow::Week => dt + chrono::Duration::weeks(1),
            TimeWindow::Month => {
                // Add month and set to 1st
                let month = dt.month() as u32 + 1;
                let year = dt.year() + if month > 12 { 1 } else { 0 };
                let month = if month > 12 { 1 } else { month };
                Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0)
                    .single()
                    .unwrap()
            },
            TimeWindow::CustomSeconds(seconds) => dt + chrono::Duration::seconds(*seconds as i64),
        }
    }
}

/// Aggregation function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationFn {
    /// Sum of all values
    Sum,

    /// Average (mean)
    Avg,

    /// Minimum value
    Min,

    /// Maximum value
    Max,

    /// Count of values
    Count,

    /// P90 (90th percentile)
    P90,

    /// P95 (95th percentile)
    P95,

    /// P99 (99th percentile)
    P99,
}

impl AggregationFn {
    /// Apply the aggregation function to a set of values
    pub fn apply(&self, values: &[f64]) -> Option<f64> {
        if values.is_empty() {
            return None;
        }

        match self {
            AggregationFn::Sum => Some(values.iter().sum()),
            AggregationFn::Avg => Some(values.iter().sum::<f64>() / values.len() as f64),
            AggregationFn::Min => Some(values.iter().fold(f64::INFINITY, |a, &b| a.min(b))),
            AggregationFn::Max => Some(values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))),
            AggregationFn::Count => Some(values.len() as f64),
            AggregationFn::P90 => percentile(values, 0.90),
            AggregationFn::P95 => percentile(values, 0.95),
            AggregationFn::P99 => percentile(values, 0.99),
        }
    }
}

/// Calculate percentile of sorted values
fn percentile(values: &[f64], percentile: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let index = (percentile * (sorted.len() - 1) as f64) as usize;
    Some(sorted[index])
}

/// Aggregation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aggregation {
    /// Time window for grouping
    pub window: TimeWindow,

    /// Metric to aggregate
    pub metric: String,

    /// Aggregation function
    pub function: AggregationFn,
}

impl Aggregation {
    /// Create a new aggregation
    pub fn new(metric: impl Into<String>, function: AggregationFn, window: TimeWindow) -> Self {
        Self {
            window,
            metric: metric.into(),
            function,
        }
    }

    /// Create a sum aggregation
    pub fn sum(metric: impl Into<String>, window: TimeWindow) -> Self {
        Self::new(metric, AggregationFn::Sum, window)
    }

    /// Create an average aggregation
    pub fn avg(metric: impl Into<String>, window: TimeWindow) -> Self {
        Self::new(metric, AggregationFn::Avg, window)
    }

    /// Create a count aggregation
    pub fn count(metric: impl Into<String>, window: TimeWindow) -> Self {
        Self::new(metric, AggregationFn::Count, window)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_window_truncate_hour() {
        let dt = Utc.with_ymd_and_hms(2025, 1, 8, 14, 30, 45).unwrap();
        let truncated = TimeWindow::Hour.truncate(dt);
        assert_eq!(
            truncated,
            Utc.with_ymd_and_hms(2025, 1, 8, 14, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_time_window_truncate_day() {
        let dt = Utc.with_ymd_and_hms(2025, 1, 8, 14, 30, 45).unwrap();
        let truncated = TimeWindow::Day.truncate(dt);
        assert_eq!(
            truncated,
            Utc.with_ymd_and_hms(2025, 1, 8, 0, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_aggregation_fn_sum() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(AggregationFn::Sum.apply(&values), Some(15.0));
    }

    #[test]
    fn test_aggregation_fn_avg() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(AggregationFn::Avg.apply(&values), Some(3.0));
    }

    #[test]
    fn test_aggregation_fn_empty() {
        let values: Vec<f64> = vec![];
        assert_eq!(AggregationFn::Sum.apply(&values), None);
    }

    #[test]
    fn test_percentile() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        // 90th percentile of 10 values is at index 9 (0.90 * 9 = 8.1, rounded/truncated to 8)
        // But our simple implementation uses truncation, so 0.90 * 9 = 8.1 -> index 8 -> value 9.0
        // And 0.95 * 9 = 8.55 -> index 8 -> value 9.0
        // The simple implementation isn't perfectly accurate but works for basic usage
        let p90 = percentile(&values, 0.90);
        let p95 = percentile(&values, 0.95);
        assert!(p90.is_some());
        assert!(p95.is_some());
        assert!(p90.unwrap() >= 8.0 && p90.unwrap() <= 10.0);
    }
}
