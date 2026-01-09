//! Audit logging for vault operations
//!
//! Comprehensive audit trail for all vault operations, supporting
//! compliance and security monitoring.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Audit entry for a vault operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique entry ID
    pub id: String,

    /// Timestamp of operation
    pub timestamp: DateTime<Utc>,

    /// Operation type
    pub operation: AuditOperation,

    /// Target (token name, session ID, etc.)
    pub target: String,

    /// User/actor who performed operation
    pub actor: String,

    /// Operation result
    pub result: AuditResult,

    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Types of auditable operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditOperation {
    /// Token created
    TokenCreate,

    /// Token read
    TokenRead,

    /// Token updated
    TokenUpdate,

    /// Token deleted
    TokenDelete,

    /// Token rotated
    TokenRotate,

    /// Session created
    SessionCreate,

    /// Session deleted
    SessionDelete,

    /// Vault unlocked
    VaultUnlock,

    /// Vault locked
    VaultLock,

    /// Authentication failed
    AuthFailed,

    /// Export operation
    Export,

    /// Import operation
    Import,

    /// Custom operation
    Custom(String),
}

/// Operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditResult {
    Success,
    Failure(String),
}

impl AuditEntry {
    /// Create new audit entry
    pub fn new(
        operation: AuditOperation,
        target: String,
        actor: String,
        result: AuditResult,
    ) -> Self {
        Self {
            id: Self::generate_id(),
            timestamp: Utc::now(),
            operation,
            target,
            actor,
            result,
            metadata: None,
        }
    }

    /// Generate unique ID
    fn generate_id() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();

        (0..16)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Convert to JSON
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Parse from JSON
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }
}

/// In-memory audit log
#[derive(Debug, Clone)]
pub struct AuditLog {
    entries: Vec<AuditEntry>,
    max_entries: usize,
}

impl AuditLog {
    /// Create new audit log
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    /// Create audit log with specific capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            max_entries: capacity,
        }
    }

    /// Add entry to log
    pub fn log(&mut self, entry: AuditEntry) {
        self.entries.push(entry);

        // Enforce max entries (FIFO)
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    /// Get all entries
    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    /// Filter entries by operation
    pub fn filter_by_operation(&self, operation: &AuditOperation) -> Vec<AuditEntry> {
        self.entries
            .iter()
            .filter(|e| &e.operation == operation)
            .cloned()
            .collect()
    }

    /// Filter entries by actor
    pub fn filter_by_actor(&self, actor: &str) -> Vec<AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.actor == actor)
            .cloned()
            .collect()
    }

    /// Filter entries by time range
    pub fn filter_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Get failed operations
    pub fn failures(&self) -> Vec<AuditEntry> {
        self.entries
            .iter()
            .filter(|e| matches!(e.result, AuditResult::Failure(_)))
            .cloned()
            .collect()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Export to JSON
    pub fn export_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self.entries)
    }

    /// Import from JSON
    pub fn import_json(&mut self, json: &str) -> serde_json::Result<()> {
        let entries: Vec<AuditEntry> = serde_json::from_str(json)?;
        self.entries = entries;
        Ok(())
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_audit_entry() {
        let entry = AuditEntry::new(
            AuditOperation::TokenCreate,
            "my_token".to_string(),
            "user1".to_string(),
            AuditResult::Success,
        );

        assert_eq!(entry.target, "my_token");
        assert_eq!(entry.actor, "user1");
        assert!(matches!(entry.result, AuditResult::Success));
    }

    #[test]
    fn test_audit_log() {
        let mut log = AuditLog::new();

        let entry1 = AuditEntry::new(
            AuditOperation::TokenCreate,
            "token1".to_string(),
            "user1".to_string(),
            AuditResult::Success,
        );

        let entry2 = AuditEntry::new(
            AuditOperation::TokenRead,
            "token1".to_string(),
            "user1".to_string(),
            AuditResult::Success,
        );

        log.log(entry1);
        log.log(entry2);

        assert_eq!(log.entries().len(), 2);
    }

    #[test]
    fn test_filter_by_operation() {
        let mut log = AuditLog::new();

        log.log(AuditEntry::new(
            AuditOperation::TokenCreate,
            "token1".to_string(),
            "user1".to_string(),
            AuditResult::Success,
        ));

        log.log(AuditEntry::new(
            AuditOperation::TokenRead,
            "token1".to_string(),
            "user1".to_string(),
            AuditResult::Success,
        ));

        let creates = log.filter_by_operation(&AuditOperation::TokenCreate);
        assert_eq!(creates.len(), 1);
    }

    #[test]
    fn test_failures() {
        let mut log = AuditLog::new();

        log.log(AuditEntry::new(
            AuditOperation::TokenRead,
            "token1".to_string(),
            "user1".to_string(),
            AuditResult::Success,
        ));

        log.log(AuditEntry::new(
            AuditOperation::TokenRead,
            "token2".to_string(),
            "user1".to_string(),
            AuditResult::Failure("Not found".to_string()),
        ));

        let failures = log.failures();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].target, "token2");
    }

    #[test]
    fn test_export_import() {
        let mut log = AuditLog::new();

        log.log(AuditEntry::new(
            AuditOperation::TokenCreate,
            "token1".to_string(),
            "user1".to_string(),
            AuditResult::Success,
        ));

        let json = log.export_json().unwrap();

        let mut log2 = AuditLog::new();
        log2.import_json(&json).unwrap();

        assert_eq!(log2.entries().len(), 1);
    }
}
