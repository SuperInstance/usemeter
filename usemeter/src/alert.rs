//! Alerting system for budgets and unusual usage

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Alert error types
#[derive(Error, Debug)]
pub enum AlertError {
    #[error("Invalid alert rule: {0}")]
    InvalidRule(String),

    #[error("Alert evaluation failed: {0}")]
    EvaluationFailed(String),
}

/// Alert threshold type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertThreshold {
    /// Single value threshold
    Absolute(f64),

    /// Percentage threshold
    Percentage(f64),

    /// Standard deviation threshold
    StandardDeviations(f64),
}

/// Alert rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Rule name
    pub name: String,

    /// Metric to monitor
    pub metric: String,

    /// Threshold
    pub threshold: AlertThreshold,

    /// Alert type
    pub rule_type: AlertRuleType,

    /// Time window in seconds
    pub time_window_seconds: u64,

    /// Check interval in seconds
    pub check_interval_seconds: u64,

    /// Enabled
    pub enabled: bool,
}

/// Alert rule type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertRuleType {
    /// Alert when metric exceeds threshold
    AboveThreshold,

    /// Alert when metric falls below threshold
    BelowThreshold,

    /// Alert on unusual change (anomaly detection)
    Anomaly,
}

impl AlertRule {
    /// Create a budget alert (spending exceeds threshold)
    pub fn budget_alert(
        name: impl Into<String>,
        metric: impl Into<String>,
        threshold_cents: f64,
    ) -> Self {
        Self {
            name: name.into(),
            metric: metric.into(),
            threshold: AlertThreshold::Absolute(threshold_cents),
            rule_type: AlertRuleType::AboveThreshold,
            time_window_seconds: 30 * 24 * 60 * 60, // 30 days
            check_interval_seconds: 24 * 60 * 60,    // 1 day
            enabled: true,
        }
    }

    /// Create a quota alert (usage exceeds threshold)
    pub fn quota_alert(
        name: impl Into<String>,
        metric: impl Into<String>,
        threshold: f64,
    ) -> Self {
        Self {
            name: name.into(),
            metric: metric.into(),
            threshold: AlertThreshold::Absolute(threshold),
            rule_type: AlertRuleType::AboveThreshold,
            time_window_seconds: 24 * 60 * 60, // 1 day
            check_interval_seconds: 60 * 60,    // 1 hour
            enabled: true,
        }
    }

    /// Create an anomaly detection alert
    pub fn anomaly_alert(
        name: impl Into<String>,
        metric: impl Into<String>,
        threshold_std_devs: f64,
    ) -> Self {
        Self {
            name: name.into(),
            metric: metric.into(),
            threshold: AlertThreshold::StandardDeviations(threshold_std_devs),
            rule_type: AlertRuleType::Anomaly,
            time_window_seconds: 7 * 24 * 60 * 60, // 7 days
            check_interval_seconds: 60 * 60,        // 1 hour
            enabled: true,
        }
    }
}

/// Alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,

    /// Rule that triggered this alert
    pub rule_name: String,

    /// User/tenant ID
    pub user_id: Option<String>,

    /// Metric value
    pub metric_value: f64,

    /// Threshold value
    pub threshold_value: f64,

    /// Alert severity
    pub severity: AlertSeverity,

    /// Alert message
    pub message: String,

    /// Alert triggered at
    pub triggered_at: DateTime<Utc>,

    /// Alert acknowledged
    pub acknowledged: bool,

    /// Alert resolved
    pub resolved: bool,
}

/// Alert severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Info only
    Info,

    /// Warning
    Warning,

    /// Critical
    Critical,
}

impl Alert {
    /// Create a new alert
    pub fn new(
        rule_name: impl Into<String>,
        user_id: Option<String>,
        metric_value: f64,
        threshold_value: f64,
        severity: AlertSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            rule_name: rule_name.into(),
            user_id,
            metric_value,
            threshold_value,
            severity,
            message: message.into(),
            triggered_at: Utc::now(),
            acknowledged: false,
            resolved: false,
        }
    }

    /// Acknowledge the alert
    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
    }

    /// Resolve the alert
    pub fn resolve(&mut self) {
        self.resolved = true;
    }
}

/// Alert manager for evaluating rules and triggering alerts
pub struct AlertManager {
    /// Alert rules
    rules: Vec<AlertRule>,

    /// Historical data for anomaly detection
    history: std::collections::HashMap<String, Vec<f64>>,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            history: HashMap::new(),
        }
    }

    /// Add an alert rule
    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.push(rule);
    }

    /// Remove an alert rule
    pub fn remove_rule(&mut self, name: &str) -> bool {
        if let Some(pos) = self.rules.iter().position(|r| r.name == name) {
            self.rules.remove(pos);
            return true;
        }
        false
    }

    /// Get all rules
    pub fn rules(&self) -> &[AlertRule] {
        &self.rules
    }

    /// Evaluate rules against current metrics
    ///
    /// Returns a list of triggered alerts.
    pub fn evaluate(
        &mut self,
        user_id: Option<String>,
        metrics: &std::collections::HashMap<String, f64>,
    ) -> Vec<Alert> {
        let mut alerts = Vec::new();

        // Collect enabled rules first to avoid borrow issues
        let enabled_rules: Vec<_> = self.rules.iter()
            .filter(|r| r.enabled)
            .map(|r| (r.metric.clone(), r.clone()))
            .collect();

        for (metric_name, rule) in enabled_rules {
            if let Some(&value) = metrics.get(&metric_name) {
                if let Some(alert) = self.evaluate_rule(&rule, user_id.clone(), value) {
                    alerts.push(alert);
                }
            }
        }

        alerts
    }

    /// Evaluate a single rule
    fn evaluate_rule(
        &mut self,
        rule: &AlertRule,
        user_id: Option<String>,
        value: f64,
    ) -> Option<Alert> {
        match &rule.rule_type {
            AlertRuleType::AboveThreshold => {
                let threshold = match &rule.threshold {
                    AlertThreshold::Absolute(v) => *v,
                    AlertThreshold::Percentage(_p) => {
                        // For percentage, need baseline - skip for now
                        return None;
                    }
                    AlertThreshold::StandardDeviations(std_dev) => {
                        // Calculate baseline from history
                        let baseline = self.calculate_baseline(&rule.metric);
                        let std_dev_val = self.calculate_std_dev(&rule.metric);
                        baseline + (std_dev_val * std_dev)
                    }
                };

                if value > threshold {
                    Some(Alert::new(
                        &rule.name,
                        user_id,
                        value,
                        threshold,
                        AlertSeverity::Warning,
                        format!(
                            "{} exceeded threshold: {:.2} > {:.2}",
                            rule.metric, value, threshold
                        ),
                    ))
                } else {
                    None
                }
            }

            AlertRuleType::BelowThreshold => {
                let threshold = match &rule.threshold {
                    AlertThreshold::Absolute(v) => *v,
                    _ => return None,
                };

                if value < threshold {
                    Some(Alert::new(
                        &rule.name,
                        user_id,
                        value,
                        threshold,
                        AlertSeverity::Info,
                        format!(
                            "{} below threshold: {:.2} < {:.2}",
                            rule.metric, value, threshold
                        ),
                    ))
                } else {
                    None
                }
            }

            AlertRuleType::Anomaly => {
                // Update history
                self.history
                    .entry(rule.metric.clone())
                    .or_insert_with(Vec::new)
                    .push(value);

                let std_devs = match &rule.threshold {
                    AlertThreshold::StandardDeviations(s) => *s,
                    _ => return None,
                };

                let baseline = self.calculate_baseline(&rule.metric);
                let std_dev = self.calculate_std_dev(&rule.metric);

                if std_dev > 0.0 {
                    let z_score = (value - baseline) / std_dev;
                    if z_score.abs() > std_devs {
                        Some(Alert::new(
                            &rule.name,
                            user_id,
                            value,
                            baseline,
                            AlertSeverity::Critical,
                            format!(
                                "Anomaly detected in {}: {:.2} deviates {:.2}σ from baseline {:.2}",
                                rule.metric, value, z_score, baseline
                            ),
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    /// Calculate baseline (mean) for a metric
    fn calculate_baseline(&self, metric: &str) -> f64 {
        if let Some(values) = self.history.get(metric) {
            if values.is_empty() {
                return 0.0;
            }
            let sum: f64 = values.iter().sum();
            sum / values.len() as f64
        } else {
            0.0
        }
    }

    /// Calculate standard deviation for a metric
    fn calculate_std_dev(&self, metric: &str) -> f64 {
        if let Some(values) = self.history.get(metric) {
            if values.len() < 2 {
                return 0.0;
            }

            let mean = self.calculate_baseline(metric);
            let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
            variance.sqrt()
        } else {
            0.0
        }
    }

    /// Clear history for a metric
    pub fn clear_history(&mut self, metric: &str) {
        self.history.remove(metric);
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_rule_budget() {
        let rule = AlertRule::budget_alert("monthly_budget", "cost_cents", 10000.0);
        assert_eq!(rule.name, "monthly_budget");
        assert_eq!(rule.metric, "cost_cents");
        assert!(matches!(rule.rule_type, AlertRuleType::AboveThreshold));
    }

    #[test]
    fn test_alert_manager() {
        let mut manager = AlertManager::new();
        manager.add_rule(AlertRule::budget_alert("budget", "cost", 100.0));

        let mut metrics = std::collections::HashMap::new();
        metrics.insert("cost".to_string(), 150.0);

        let alerts = manager.evaluate(Some("user-123".to_string()), &metrics);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].metric_value, 150.0);
        assert_eq!(alerts[0].threshold_value, 100.0);
    }

    #[test]
    fn test_alert_acknowledge() {
        let mut alert = Alert::new(
            "test_rule",
            Some("user-123".to_string()),
            100.0,
            50.0,
            AlertSeverity::Warning,
            "Test message".to_string(),
        );

        assert!(!alert.acknowledged);
        alert.acknowledge();
        assert!(alert.acknowledged);
    }
}
