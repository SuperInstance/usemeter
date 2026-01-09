//! Storage backends for usage events

use crate::event::Event;
use crate::query::{QueryBuilder, UsageStats};
use chrono::{DateTime, Utc};
use thiserror::Error;

/// Storage error types
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Backend not available: {0}")]
    BackendUnavailable(String),
}

/// Result type for storage operations
pub type Result<T> = std::result::Result<T, StorageError>;

/// Trait for storage backends
///
/// Storage backends handle the persistence and retrieval of usage events.
/// Different backends can be used depending on scale and requirements:
///
/// - **SQLite**: Embedded, zero-config, good for single-instance apps
/// - **PostgreSQL/MySQL**: Scalable, good for distributed systems
/// - **File**: Simple, good for testing and small deployments
#[async_trait::async_trait]
pub trait StorageBackend: Send + Sync {
    /// Store a single event
    async fn store(&self, event: &Event) -> Result<()>;

    /// Store multiple events in batch
    async fn store_batch(&self, events: &[Event]) -> Result<()> {
        for event in events {
            self.store(event).await?;
        }
        Ok(())
    }

    /// Query events with filters
    async fn query(&self, builder: &QueryBuilder) -> Result<Vec<Event>>;

    /// Get aggregated usage statistics
    async fn query_stats(&self, builder: &QueryBuilder) -> Result<UsageStats>;

    /// Delete events older than the given timestamp
    async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<u64>;

    /// Count total events
    async fn count(&self) -> Result<u64>;

    /// Initialize the storage backend (create tables, etc.)
    async fn initialize(&self) -> Result<()>;

    /// Close the storage backend and cleanup resources
    async fn close(&self) -> Result<()>;
}

/// SQLite storage backend
///
/// Uses SQLite for embedded, zero-configuration storage.
/// Good for single-instance applications and testing.
///
/// # Example
///
/// ```rust,no_run
/// use usemeter::storage::SqliteBackend;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let backend = SqliteBackend::new_in_memory()?;
/// backend.initialize().await?;
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "sqlite")]
pub struct SqliteBackend {
    conn: Arc<tokio::sync::Mutex<rusqlite::Connection>>,
    path: Option<String>,
}

#[cfg(feature = "sqlite")]
impl SqliteBackend {
    /// Create a new SQLite backend with a file database
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let conn = rusqlite::Connection::open(&path)
            .map_err(|e| StorageError::Database(e.to_string()))?;
        Ok(Self {
            conn: Arc::new(tokio::sync::Mutex::new(conn)),
            path: Some(path_str),
        })
    }

    /// Create an in-memory SQLite database
    ///
    /// Useful for testing and temporary data.
    pub fn new_in_memory() -> Result<Self> {
        let conn = rusqlite::Connection::open_in_memory()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        Ok(Self {
            conn: Arc::new(tokio::sync::Mutex::new(conn)),
            path: None,
        })
    }
}

#[cfg(feature = "sqlite")]
#[async_trait::async_trait]
impl StorageBackend for SqliteBackend {
    async fn store(&self, event: &Event) -> Result<()> {
        let mut conn = self.conn.lock().await;

        let event_json = serde_json::to_string(event)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        conn.execute(
            "INSERT INTO events (id, event_type, user_id, resource_id, timestamp, event_data)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                &event.id,
                &event.event_type,
                &event.user_id,
                &event.resource_id,
                &event.timestamp.to_rfc3339(),
                &event_json,
            ),
        )
        .map_err(|e| StorageError::Database(e.to_string()))?;

        tracing::trace!("Stored event: {}", event.id);
        Ok(())
    }

    async fn store_batch(&self, events: &[Event]) -> Result<()> {
        let mut conn = self.conn.lock().await;

        let tx = conn.unchecked_transaction().map_err(|e| StorageError::Database(e.to_string()))?;

        for event in events {
            let event_json = serde_json::to_string(event)
                .map_err(|e| StorageError::Serialization(e.to_string()))?;

            tx.execute(
                "INSERT INTO events (id, event_type, user_id, resource_id, timestamp, event_data)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (
                    &event.id,
                    &event.event_type,
                    &event.user_id,
                    &event.resource_id,
                    &event.timestamp.to_rfc3339(),
                    &event_json,
                ),
            )
            .map_err(|e| StorageError::Database(e.to_string()))?;
        }

        tx.commit().map_err(|e| StorageError::Database(e.to_string()))?;
        tracing::trace!("Stored batch of {} events", events.len());
        Ok(())
    }

    async fn query(&self, builder: &QueryBuilder) -> Result<Vec<Event>> {
        let mut conn = self.conn.lock().await;

        let mut query = String::from("SELECT event_data FROM events WHERE 1=1");
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(user_id) = &builder.user_id {
            query.push_str(" AND user_id = ?");
            params.push(Box::new(user_id.clone()));
        }

        if let Some(event_type) = &builder.event_type {
            query.push_str(" AND event_type = ?");
            params.push(Box::new(event_type.clone()));
        }

        if let Some(start) = &builder.start_time {
            query.push_str(" AND timestamp >= ?");
            params.push(Box::new(start.to_rfc3339()));
        }

        if let Some(end) = &builder.end_time {
            query.push_str(" AND timestamp <= ?");
            params.push(Box::new(end.to_rfc3339()));
        }

        query.push_str(" ORDER BY timestamp");

        if let Some(limit) = builder.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        let mut stmt = conn.prepare(&query).map_err(|e| StorageError::Database(e.to_string()))?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let events = stmt
            .query_map(param_refs.as_slice(), |row| {
                let json: String = row.get(0)?;
                serde_json::from_str(&json).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })
            .map_err(|e| StorageError::Database(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(events)
    }

    async fn query_stats(&self, builder: &QueryBuilder) -> Result<UsageStats> {
        // For now, query events and calculate stats
        // TODO: Implement SQL aggregation for better performance
        let events = self.query(builder).await?;

        let total_events = events.len() as u64;
        let mut metric_sums: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        let mut metric_counts: std::collections::HashMap<String, u64> = std::collections::HashMap::new();

        for event in events {
            for (name, value) in &event.metrics {
                if let Some(float_val) = value.as_f64() {
                    *metric_sums.entry(name.clone()).or_insert(0.0) += float_val;
                    *metric_counts.entry(name.clone()).or_insert(0) += 1;
                }
            }
        }

        Ok(UsageStats {
            total_events,
            metric_sums,
            metric_counts,
            start_time: builder.start_time,
            end_time: builder.end_time,
        })
    }

    async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<u64> {
        let mut conn = self.conn.lock().await;

        let count = conn.execute(
            "DELETE FROM events WHERE timestamp < ?",
            [&timestamp.to_rfc3339()],
        ).map_err(|e| StorageError::Database(e.to_string()))?;

        tracing::debug!("Deleted {} events before {}", count, timestamp);
        Ok(count)
    }

    async fn count(&self) -> Result<u64> {
        let mut conn = self.conn.lock().await;

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(count as u64)
    }

    async fn initialize(&self) -> Result<()> {
        let mut conn = self.conn.lock().await;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                user_id TEXT NOT NULL,
                resource_id TEXT,
                timestamp TEXT NOT NULL,
                event_data TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| StorageError::Database(e.to_string()))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_user_id ON events(user_id)",
            [],
        )
        .map_err(|e| StorageError::Database(e.to_string()))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp)",
            [],
        )
        .map_err(|e| StorageError::Database(e.to_string()))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type)",
            [],
        )
        .map_err(|e| StorageError::Database(e.to_string()))?;

        tracing::info!("SQLite storage initialized");
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        // Pool will close when dropped
        Ok(())
    }
}

/// File-based storage backend
///
/// Stores events as JSONL (JSON Lines) files.
/// Simple and portable, good for testing and small deployments.
///
/// # Example
///
/// ```rust,no_run
/// use usemeter::storage::FileBackend;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let backend = FileBackend::new("/path/to/events.jsonl")?;
/// backend.initialize().await?;
/// # Ok(())
/// # }
/// ```
pub struct FileBackend {
    path: std::path::PathBuf,
    lock: tokio::sync::Mutex<std::fs::File>,
}

impl FileBackend {
    /// Create a new file backend
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Open file for append, create if doesn't exist
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        Ok(Self {
            path,
            lock: tokio::sync::Mutex::new(file),
        })
    }

    /// Read all events from the file
    fn read_all(&self) -> Result<Vec<Event>> {
        let file = std::fs::File::open(&self.path)?;
        let reader = std::io::BufReader::new(file);

        let mut events = Vec::new();
        for line in std::io::Lines::new(reader) {
            let line = line.map_err(|e| StorageError::Io(e))?;
            if line.trim().is_empty() {
                continue;
            }
            let event: Event = serde_json::from_str(&line)
                .map_err(|e| StorageError::Serialization(e.to_string()))?;
            events.push(event);
        }

        Ok(events)
    }
}

#[async_trait::async_trait]
impl StorageBackend for FileBackend {
    async fn store(&self, event: &Event) -> Result<()> {
        let mut file = self.lock.lock().await;
        let json = serde_json::to_string(event)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        writeln!(file, "{}", json).map_err(|e| StorageError::Io(e))?;
        file.flush().map_err(|e| StorageError::Io(e))?;
        tracing::trace!("Stored event: {}", event.id);
        Ok(())
    }

    async fn query(&self, builder: &QueryBuilder) -> Result<Vec<Event>> {
        let events = self.read_all()?;

        let filtered: Vec<_> = events
            .into_iter()
            .filter(|event| {
                if let Some(user_id) = &builder.user_id {
                    if event.user_id != *user_id {
                        return false;
                    }
                }

                if let Some(event_type) = &builder.event_type {
                    if event.event_type != *event_type {
                        return false;
                    }
                }

                if let Some(start) = &builder.start_time {
                    if event.timestamp < *start {
                        return false;
                    }
                }

                if let Some(end) = &builder.end_time {
                    if event.timestamp > *end {
                        return false;
                    }
                }

                true
            })
            .take(builder.limit.unwrap_or(usize::MAX))
            .collect();

        Ok(filtered)
    }

    async fn query_stats(&self, builder: &QueryBuilder) -> Result<UsageStats> {
        let events = self.query(builder).await?;

        let total_events = events.len() as u64;
        let mut metric_sums: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        let mut metric_counts: std::collections::HashMap<String, u64> = std::collections::HashMap::new();

        for event in events {
            for (name, value) in &event.metrics {
                if let Some(float_val) = value.as_f64() {
                    *metric_sums.entry(name.clone()).or_insert(0.0) += float_val;
                    *metric_counts.entry(name.clone()).or_insert(0) += 1;
                }
            }
        }

        Ok(UsageStats {
            total_events,
            metric_sums,
            metric_counts,
            start_time: builder.start_time,
            end_time: builder.end_time,
        })
    }

    async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<u64> {
        let events = self.read_all()?;
        let filtered: Vec<_> = events.into_iter().filter(|e| e.timestamp >= timestamp).collect();

        // Rewrite file with filtered events
        let mut file = self.lock.lock().await;
        file.set_len(0).map_err(|e| StorageError::Io(e))?;
        file.rewind().map_err(|e| StorageError::Io(e))?;

        for event in &filtered {
            let json = serde_json::to_string(event)
                .map_err(|e| StorageError::Serialization(e.to_string()))?;
            writeln!(file, "{}", json).map_err(|e| StorageError::Io(e))?;
        }
        file.flush().map_err(|e| StorageError::Io(e))?;

        let deleted_count = events.len() - filtered.len();
        tracing::debug!("Deleted {} events before {}", deleted_count, timestamp);
        Ok(deleted_count as u64)
    }

    async fn count(&self) -> Result<u64> {
        let events = self.read_all()?;
        Ok(events.len() as u64)
    }

    async fn initialize(&self) -> Result<()> {
        // File is created in constructor
        tracing::info!("File storage initialized at {:?}", self.path);
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }
}
