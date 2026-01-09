//! Billing engine for calculating costs and generating invoices

use crate::event::{Event, MetricValue};
use crate::query::UsageStats;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Billing error types
#[derive(Error, Debug)]
pub enum BillingError {
    #[error("Invalid pricing rule: {0}")]
    InvalidRule(String),

    #[error("Cost calculation failed: {0}")]
    CalculationFailed(String),

    #[error("Invoice generation failed: {0}")]
    InvoiceFailed(String),

    #[error("Report generation failed: {0}")]
    ReportFailed(String),
}

/// Pricing rule for calculating costs
///
/// Defines how to calculate costs based on usage metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingRule {
    /// Rule name
    pub name: String,

    /// Metric to price (e.g., "tokens", "api_calls")
    pub metric: String,

    /// Unit price (in cents)
    pub unit_price_cents: f64,

    /// Unit quantity (e.g., 1000 tokens, 1 API call)
    pub unit_quantity: f64,

    /// Tiered pricing (optional)
    pub tiers: Option<Vec<PricingTier>>,
}

impl PricingRule {
    /// Create a simple pricing rule
    pub fn new(
        name: impl Into<String>,
        metric: impl Into<String>,
        unit_price_cents: f64,
        unit_quantity: f64,
    ) -> Self {
        Self {
            name: name.into(),
            metric: metric.into(),
            unit_price_cents,
            unit_quantity,
            tiers: None,
        }
    }

    /// Create a tiered pricing rule
    pub fn with_tiers(
        name: impl Into<String>,
        metric: impl Into<String>,
        tiers: Vec<PricingTier>,
    ) -> Self {
        Self {
            name: name.into(),
            metric: metric.into(),
            unit_price_cents: 0.0,
            unit_quantity: 1.0,
            tiers: Some(tiers),
        }
    }

    /// Calculate cost for a given metric value
    pub fn calculate_cost(&self, value: f64) -> f64 {
        if let Some(tiers) = &self.tiers {
            // Tiered pricing
            let mut remaining = value;
            let mut total_cost = 0.0;

            for tier in tiers {
                if remaining <= 0.0 {
                    break;
                }

                let tier_amount = remaining.min(tier.max_quantity - tier.min_quantity);
                let tier_cost = (tier_amount / tier.unit_quantity) * tier.unit_price_cents;
                total_cost += tier_cost;
                remaining -= tier_amount;
            }

            total_cost
        } else {
            // Simple linear pricing
            (value / self.unit_quantity) * self.unit_price_cents
        }
    }
}

/// Pricing tier for tiered pricing models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingTier {
    /// Minimum quantity for this tier
    pub min_quantity: f64,

    /// Maximum quantity for this tier
    pub max_quantity: f64,

    /// Unit price (in cents)
    pub unit_price_cents: f64,

    /// Unit quantity
    pub unit_quantity: f64,
}

/// Cost calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostCalculation {
    /// Total cost in cents
    pub total_cents: f64,

    /// Breakdown by metric
    pub breakdown: HashMap<String, MetricCost>,

    /// Calculated at
    pub calculated_at: DateTime<Utc>,
}

/// Cost for a single metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricCost {
    /// Metric name
    pub metric: String,

    /// Value
    pub value: f64,

    /// Cost in cents
    pub cost_cents: f64,

    /// Pricing rule used
    pub rule_name: String,
}

/// Billing engine for calculating costs
pub struct BillingEngine {
    /// Pricing rules by name
    rules: HashMap<String, PricingRule>,
}

impl BillingEngine {
    /// Create a new billing engine
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    /// Add a pricing rule
    pub fn add_rule(&mut self, rule: PricingRule) {
        self.rules.insert(rule.name.clone(), rule);
    }

    /// Get a pricing rule
    pub fn get_rule(&self, name: &str) -> Option<&PricingRule> {
        self.rules.get(name)
    }

    /// Calculate cost from usage stats
    pub fn calculate_cost(
        &self,
        stats: &UsageStats,
        rule_names: &[String],
    ) -> Result<CostCalculation, BillingError> {
        let mut total_cents = 0.0;
        let mut breakdown = HashMap::new();

        for rule_name in rule_names {
            let rule = self.rules.get(rule_name).ok_or_else(|| {
                BillingError::InvalidRule(format!("Rule '{}' not found", rule_name))
            })?;

            let value = stats.get_metric_sum(&rule.metric).unwrap_or(0.0);

            let cost = rule.calculate_cost(value);
            total_cents += cost;

            breakdown.insert(
                rule.metric.clone(),
                MetricCost {
                    metric: rule.metric.clone(),
                    value,
                    cost_cents: cost,
                    rule_name: rule.name.clone(),
                },
            );
        }

        Ok(CostCalculation {
            total_cents,
            breakdown,
            calculated_at: Utc::now(),
        })
    }

    /// Calculate cost for a single event
    pub fn calculate_event_cost(
        &self,
        event: &Event,
        rule_names: &[String],
    ) -> Result<CostCalculation, BillingError> {
        let mut total_cents = 0.0;
        let mut breakdown = HashMap::new();

        for rule_name in rule_names {
            let rule = self.rules.get(rule_name).ok_or_else(|| {
                BillingError::InvalidRule(format!("Rule '{}' not found", rule_name))
            })?;

            let value = event
                .get_metric(&rule.metric)
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            let cost = rule.calculate_cost(value);
            total_cents += cost;

            breakdown.insert(
                rule.metric.clone(),
                MetricCost {
                    metric: rule.metric.clone(),
                    value,
                    cost_cents: cost,
                    rule_name: rule.name.clone(),
                },
            );
        }

        Ok(CostCalculation {
            total_cents,
            breakdown,
            calculated_at: Utc::now(),
        })
    }
}

impl Default for BillingEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Invoice for billing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    /// Invoice ID
    pub id: String,

    /// User ID
    pub user_id: String,

    /// Invoice period start
    pub period_start: DateTime<Utc>,

    /// Invoice period end
    pub period_end: DateTime<Utc>,

    /// Cost calculation
    pub cost: CostCalculation,

    /// Invoice created at
    pub created_at: DateTime<Utc>,

    /// Invoice status
    pub status: InvoiceStatus,
}

/// Invoice status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvoiceStatus {
    /// Draft - not yet finalized
    Draft,

    /// Pending - awaiting payment
    Pending,

    /// Paid
    Paid,

    /// Overdue
    Overdue,

    /// Cancelled
    Cancelled,
}

impl Invoice {
    /// Create a new invoice
    pub fn new(
        user_id: impl Into<String>,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        cost: CostCalculation,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.into(),
            period_start,
            period_end,
            cost,
            created_at: Utc::now(),
            status: InvoiceStatus::Draft,
        }
    }

    /// Get total amount in cents
    pub fn total_cents(&self) -> f64 {
        self.cost.total_cents
    }

    /// Get total amount in dollars
    pub fn total_dollars(&self) -> f64 {
        self.cost.total_cents / 100.0
    }
}

/// Report generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    /// Report ID
    pub id: String,

    /// Report type
    pub report_type: ReportType,

    /// Report data
    pub data: ReportData,

    /// Generated at
    pub generated_at: DateTime<Utc>,
}

/// Report type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportType {
    /// Usage report
    Usage,

    /// Cost report
    Cost,

    /// Invoice report
    Invoice,
}

/// Report data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportData {
    /// Report period
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,

    /// User stats (if single user report)
    pub user_id: Option<String>,

    /// Total events
    pub total_events: u64,

    /// Total cost in cents
    pub total_cost_cents: f64,

    /// Metric breakdown
    pub metrics: HashMap<String, f64>,
}

impl Report {
    /// Create a new report
    pub fn new(report_type: ReportType, data: ReportData) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            report_type,
            data,
            generated_at: Utc::now(),
        }
    }

    /// Export to CSV (if csv feature is enabled)
    #[cfg(feature = "csv-reports")]
    pub fn to_csv(&self) -> Result<String, BillingError> {
        let mut w = csv::Writer::from_writer(vec![]);

        // Write header
        w.serialize(&[
            "Period Start",
            "Period End",
            "User ID",
            "Total Events",
            "Total Cost (Cents)",
        ])
        .map_err(|e| BillingError::ReportFailed(e.to_string()))?;

        // Write data
        w.serialize(&[
            self.data.period_start.to_rfc3339(),
            self.data.period_end.to_rfc3339(),
            self.data.user_id.as_deref().unwrap_or("").to_string(),
            self.data.total_events.to_string(),
            self.data.total_cost_cents.to_string(),
        ])
        .map_err(|e| BillingError::ReportFailed(e.to_string()))?;

        let csv_bytes = w
            .into_inner()
            .map_err(|e| BillingError::ReportFailed(e.to_string()))?;

        String::from_utf8(csv_bytes).map_err(|e| BillingError::ReportFailed(e.to_string()))
    }

    /// Export to JSON
    pub fn to_json(&self) -> Result<String, BillingError> {
        serde_json::to_string_pretty(self).map_err(|e| BillingError::ReportFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pricing_rule_simple() {
        let rule = PricingRule::new("tokens", "tokens", 0.002, 1.0); // $0.00002 per token

        let cost = rule.calculate_cost(1000.0); // 1000 tokens
        assert!((cost - 2.0).abs() < 0.01); // 2 cents
    }

    #[test]
    fn test_pricing_rule_tiered() {
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

        let cost = rule.calculate_cost(1500.0);
        // First 1000: 1000 * 1.0 = 1000 cents
        // Next 500: 500 * 0.5 = 250 cents
        // Total: 1250 cents
        assert!((cost - 1250.0).abs() < 0.01);
    }

    #[test]
    fn test_billing_engine() {
        let mut engine = BillingEngine::new();
        engine.add_rule(PricingRule::new("tokens", "tokens", 0.002, 1.0));

        let mut stats = UsageStats::empty();
        stats.total_events = 10;
        stats.metric_sums.insert("tokens".to_string(), 10000.0);
        stats.metric_counts.insert("tokens".to_string(), 10);

        let cost = engine
            .calculate_cost(&stats, &["tokens".to_string()])
            .unwrap();

        assert!((cost.total_cents - 20.0).abs() < 0.01); // 10K tokens @ $0.00002/token
    }

    #[test]
    fn test_invoice() {
        let cost = CostCalculation {
            total_cents: 100.0,
            breakdown: HashMap::new(),
            calculated_at: Utc::now(),
        };

        let invoice = Invoice::new(
            "user-123",
            Utc::now() - chrono::Duration::days(30),
            Utc::now(),
            cost,
        );

        assert_eq!(invoice.user_id, "user-123");
        assert_eq!(invoice.total_cents(), 100.0);
        assert_eq!(invoice.total_dollars(), 1.0);
    }
}
