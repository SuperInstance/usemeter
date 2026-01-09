//! Session management for token isolation
//!
//! Sessions provide isolated namespaces for tokens, enabling multiple applications
//! or contexts to use the same vault without token name conflicts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{VaultError, VaultResult};

/// A session for token isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: String,

    /// Session name (human-readable)
    pub name: String,

    /// Session creation time
    pub created_at: DateTime<Utc>,

    /// Last access time
    pub last_accessed: DateTime<Utc>,

    /// Whether session is active
    pub active: bool,

    /// Session metadata (optional)
    pub metadata: Option<serde_json::Value>,
}

impl Session {
    /// Create a new session
    pub fn new(id: String, name: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            created_at: now,
            last_accessed: now,
            active: true,
            metadata: None,
        }
    }

    /// Update last accessed time
    pub fn touch(&mut self) {
        self.last_accessed = Utc::now();
    }

    /// Deactivate session
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Activate session
    pub fn activate(&mut self) {
        self.active = true;
    }
}

/// Session manager for handling multiple sessions
pub struct SessionManager {
    sessions: Vec<Session>,
}

impl SessionManager {
    /// Create new session manager
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
        }
    }

    /// Create a new session
    pub fn create_session(&mut self, id: String, name: String) -> VaultResult<Session> {
        if self.sessions.iter().any(|s| s.id == id) {
            return Err(VaultError::Internal(format!(
                "Session with ID '{}' already exists",
                id
            )));
        }

        let session = Session::new(id, name);
        self.sessions.push(session.clone());
        Ok(session)
    }

    /// Get session by ID
    pub fn get_session(&self, id: &str) -> VaultResult<Session> {
        self.sessions
            .iter()
            .find(|s| s.id == id)
            .cloned()
            .ok_or_else(|| VaultError::SessionNotFound(id.to_string()))
    }

    /// Update session last accessed time
    pub fn touch_session(&mut self, id: &str) -> VaultResult<()> {
        let session = self
            .sessions
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or_else(|| VaultError::SessionNotFound(id.to_string()))?;

        session.touch();
        Ok(())
    }

    /// List all active sessions
    pub fn list_active_sessions(&self) -> Vec<Session> {
        self.sessions
            .iter()
            .filter(|s| s.active)
            .cloned()
            .collect()
    }

    /// List all sessions
    pub fn list_all_sessions(&self) -> Vec<Session> {
        self.sessions.clone()
    }

    /// Deactivate session
    pub fn deactivate_session(&mut self, id: &str) -> VaultResult<()> {
        let session = self
            .sessions
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or_else(|| VaultError::SessionNotFound(id.to_string()))?;

        session.deactivate();
        Ok(())
    }

    /// Delete session
    pub fn delete_session(&mut self, id: &str) -> VaultResult<()> {
        let index = self
            .sessions
            .iter()
            .position(|s| s.id == id)
            .ok_or_else(|| VaultError::SessionNotFound(id.to_string()))?;

        self.sessions.remove(index);
        Ok(())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let mut manager = SessionManager::new();
        let session = manager
            .create_session("session1".to_string(), "Test Session".to_string())
            .unwrap();

        assert_eq!(session.id, "session1");
        assert_eq!(session.name, "Test Session");
        assert!(session.active);
    }

    #[test]
    fn test_duplicate_session_fails() {
        let mut manager = SessionManager::new();
        manager
            .create_session("session1".to_string(), "Test".to_string())
            .unwrap();

        let result = manager.create_session("session1".to_string(), "Test2".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_get_session() {
        let mut manager = SessionManager::new();
        manager
            .create_session("session1".to_string(), "Test".to_string())
            .unwrap();

        let session = manager.get_session("session1").unwrap();
        assert_eq!(session.id, "session1");
    }

    #[test]
    fn test_get_nonexistent_session_fails() {
        let mut manager = SessionManager::new();
        let result = manager.get_session("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_deactivate_session() {
        let mut manager = SessionManager::new();
        manager
            .create_session("session1".to_string(), "Test".to_string())
            .unwrap();

        manager.deactivate_session("session1").unwrap();
        let session = manager.get_session("session1").unwrap();
        assert!(!session.active);
    }

    #[test]
    fn test_list_active_sessions() {
        let mut manager = SessionManager::new();
        manager
            .create_session("session1".to_string(), "Test1".to_string())
            .unwrap();
        manager
            .create_session("session2".to_string(), "Test2".to_string())
            .unwrap();

        manager.deactivate_session("session1").unwrap();

        let active = manager.list_active_sessions();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "session2");
    }

    #[test]
    fn test_delete_session() {
        let mut manager = SessionManager::new();
        manager
            .create_session("session1".to_string(), "Test".to_string())
            .unwrap();

        manager.delete_session("session1").unwrap();
        let result = manager.get_session("session1");
        assert!(result.is_err());
    }
}
