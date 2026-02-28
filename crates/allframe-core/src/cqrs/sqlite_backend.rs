//! SQLite-backed event store backend for offline-first deployments
//!
//! Provides a persistent event store using SQLite with WAL mode for
//! concurrent read/write access. All database operations use
//! `tokio::task::spawn_blocking` since rusqlite is synchronous.

#[cfg(feature = "cqrs-sqlite")]
mod inner {
    use std::collections::HashMap;
    use std::marker::PhantomData;
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use rusqlite::Connection;

    use super::super::{BackendStats, Event, EventStoreBackend};

    /// SQLite-backed event store backend.
    ///
    /// Uses WAL journal mode for concurrent read/write access.
    /// All operations are executed via `spawn_blocking` for async compatibility.
    #[derive(Clone)]
    pub struct SqliteEventStoreBackend<E: Event> {
        conn: Arc<Mutex<Connection>>,
        _phantom: PhantomData<E>,
    }

    impl<E: Event> SqliteEventStoreBackend<E> {
        /// Create a new SQLite event store backend at the given path.
        ///
        /// Enables WAL journal mode and creates the `events` and `snapshots`
        /// tables if they don't exist.
        pub async fn new(path: &str) -> Result<Self, String> {
            let path = path.to_string();
            let conn = tokio::task::spawn_blocking(move || {
                let conn = Connection::open(&path).map_err(|e| format!("SQLite open: {}", e))?;
                conn.execute_batch("PRAGMA journal_mode=WAL;")
                    .map_err(|e| format!("WAL pragma: {}", e))?;
                conn.execute_batch(
                    "CREATE TABLE IF NOT EXISTS events (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        aggregate_id TEXT NOT NULL,
                        event_data BLOB NOT NULL,
                        created_at TEXT NOT NULL DEFAULT (datetime('now'))
                    );
                    CREATE INDEX IF NOT EXISTS idx_events_aggregate ON events(aggregate_id);
                    CREATE TABLE IF NOT EXISTS snapshots (
                        aggregate_id TEXT PRIMARY KEY,
                        snapshot_data BLOB NOT NULL,
                        version INTEGER NOT NULL
                    );",
                )
                .map_err(|e| format!("Schema init: {}", e))?;
                Ok::<_, String>(conn)
            })
            .await
            .map_err(|e| format!("spawn_blocking: {}", e))??;

            Ok(Self {
                conn: Arc::new(Mutex::new(conn)),
                _phantom: PhantomData,
            })
        }
    }

    #[async_trait]
    impl<E: Event> EventStoreBackend<E> for SqliteEventStoreBackend<E> {
        async fn append(&self, aggregate_id: &str, events: Vec<E>) -> Result<(), String> {
            let conn = Arc::clone(&self.conn);
            let agg_id = aggregate_id.to_string();
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().map_err(|e| format!("Lock: {}", e))?;
                let tx = conn
                    .unchecked_transaction()
                    .map_err(|e| format!("Begin tx: {}", e))?;
                {
                    let mut stmt = tx
                        .prepare_cached(
                            "INSERT INTO events (aggregate_id, event_data) VALUES (?1, ?2)",
                        )
                        .map_err(|e| format!("Prepare: {}", e))?;
                    for event in &events {
                        let data = serde_json::to_vec(event)
                            .map_err(|e| format!("Serialize: {}", e))?;
                        stmt.execute(rusqlite::params![agg_id, data])
                            .map_err(|e| format!("Insert: {}", e))?;
                    }
                }
                tx.commit().map_err(|e| format!("Commit: {}", e))?;
                Ok(())
            })
            .await
            .map_err(|e| format!("spawn_blocking: {}", e))?
        }

        async fn get_events(&self, aggregate_id: &str) -> Result<Vec<E>, String> {
            let conn = Arc::clone(&self.conn);
            let agg_id = aggregate_id.to_string();
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().map_err(|e| format!("Lock: {}", e))?;
                let mut stmt = conn
                    .prepare_cached(
                        "SELECT event_data FROM events WHERE aggregate_id = ?1 ORDER BY id",
                    )
                    .map_err(|e| format!("Prepare: {}", e))?;
                let rows = stmt
                    .query_map(rusqlite::params![agg_id], |row| {
                        row.get::<_, Vec<u8>>(0)
                    })
                    .map_err(|e| format!("Query: {}", e))?;
                let mut events = Vec::new();
                for row in rows {
                    let data = row.map_err(|e| format!("Row: {}", e))?;
                    let event: E = serde_json::from_slice(&data)
                        .map_err(|e| format!("Deserialize: {}", e))?;
                    events.push(event);
                }
                Ok(events)
            })
            .await
            .map_err(|e| format!("spawn_blocking: {}", e))?
        }

        async fn get_all_events(&self) -> Result<Vec<E>, String> {
            let conn = Arc::clone(&self.conn);
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().map_err(|e| format!("Lock: {}", e))?;
                let mut stmt = conn
                    .prepare_cached("SELECT event_data FROM events ORDER BY id")
                    .map_err(|e| format!("Prepare: {}", e))?;
                let rows = stmt
                    .query_map([], |row| row.get::<_, Vec<u8>>(0))
                    .map_err(|e| format!("Query: {}", e))?;
                let mut events = Vec::new();
                for row in rows {
                    let data = row.map_err(|e| format!("Row: {}", e))?;
                    let event: E = serde_json::from_slice(&data)
                        .map_err(|e| format!("Deserialize: {}", e))?;
                    events.push(event);
                }
                Ok(events)
            })
            .await
            .map_err(|e| format!("spawn_blocking: {}", e))?
        }

        async fn get_events_after(
            &self,
            aggregate_id: &str,
            version: u64,
        ) -> Result<Vec<E>, String> {
            let conn = Arc::clone(&self.conn);
            let agg_id = aggregate_id.to_string();
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().map_err(|e| format!("Lock: {}", e))?;
                // Events are 1-indexed by rowid within aggregate; we skip `version` events
                let mut stmt = conn
                    .prepare_cached(
                        "SELECT event_data FROM events WHERE aggregate_id = ?1 ORDER BY id LIMIT -1 OFFSET ?2",
                    )
                    .map_err(|e| format!("Prepare: {}", e))?;
                let rows = stmt
                    .query_map(rusqlite::params![agg_id, version as i64], |row| {
                        row.get::<_, Vec<u8>>(0)
                    })
                    .map_err(|e| format!("Query: {}", e))?;
                let mut events = Vec::new();
                for row in rows {
                    let data = row.map_err(|e| format!("Row: {}", e))?;
                    let event: E = serde_json::from_slice(&data)
                        .map_err(|e| format!("Deserialize: {}", e))?;
                    events.push(event);
                }
                Ok(events)
            })
            .await
            .map_err(|e| format!("spawn_blocking: {}", e))?
        }

        async fn save_snapshot(
            &self,
            aggregate_id: &str,
            snapshot_data: Vec<u8>,
            version: u64,
        ) -> Result<(), String> {
            let conn = Arc::clone(&self.conn);
            let agg_id = aggregate_id.to_string();
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().map_err(|e| format!("Lock: {}", e))?;
                conn.execute(
                    "INSERT OR REPLACE INTO snapshots (aggregate_id, snapshot_data, version) VALUES (?1, ?2, ?3)",
                    rusqlite::params![agg_id, snapshot_data, version as i64],
                )
                .map_err(|e| format!("Snapshot save: {}", e))?;
                Ok(())
            })
            .await
            .map_err(|e| format!("spawn_blocking: {}", e))?
        }

        async fn get_latest_snapshot(&self, aggregate_id: &str) -> Result<(Vec<u8>, u64), String> {
            let conn = Arc::clone(&self.conn);
            let agg_id = aggregate_id.to_string();
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().map_err(|e| format!("Lock: {}", e))?;
                conn.query_row(
                    "SELECT snapshot_data, version FROM snapshots WHERE aggregate_id = ?1",
                    rusqlite::params![agg_id],
                    |row| {
                        let data: Vec<u8> = row.get(0)?;
                        let version: i64 = row.get(1)?;
                        Ok((data, version as u64))
                    },
                )
                .map_err(|e| format!("Snapshot get: {}", e))
            })
            .await
            .map_err(|e| format!("spawn_blocking: {}", e))?
        }

        async fn flush(&self) -> Result<(), String> {
            let conn = Arc::clone(&self.conn);
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().map_err(|e| format!("Lock: {}", e))?;
                conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
                    .map_err(|e| format!("Checkpoint: {}", e))?;
                Ok(())
            })
            .await
            .map_err(|e| format!("spawn_blocking: {}", e))?
        }

        async fn stats(&self) -> BackendStats {
            let conn = Arc::clone(&self.conn);
            tokio::task::spawn_blocking(move || {
                let conn = match conn.lock() {
                    Ok(c) => c,
                    Err(_) => return BackendStats::default(),
                };

                let total_events: u64 = conn
                    .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
                    .unwrap_or(0);

                let total_aggregates: u64 = conn
                    .query_row(
                        "SELECT COUNT(DISTINCT aggregate_id) FROM events",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);

                let total_snapshots: u64 = conn
                    .query_row("SELECT COUNT(*) FROM snapshots", [], |row| row.get(0))
                    .unwrap_or(0);

                let journal_mode: String = conn
                    .query_row("PRAGMA journal_mode", [], |row| row.get(0))
                    .unwrap_or_default();

                let mut backend_specific = HashMap::new();
                backend_specific.insert("journal_mode".to_string(), journal_mode);
                backend_specific.insert("backend_type".to_string(), "sqlite".to_string());

                BackendStats {
                    total_events,
                    total_aggregates,
                    total_snapshots,
                    backend_specific,
                }
            })
            .await
            .unwrap_or_default()
        }
    }
}

#[cfg(feature = "cqrs-sqlite")]
pub use inner::*;

#[cfg(not(feature = "cqrs-sqlite"))]
mod placeholder {
    use std::marker::PhantomData;

    /// Placeholder when `cqrs-sqlite` feature is not enabled.
    pub struct SqliteEventStoreBackend<E> {
        _phantom: PhantomData<E>,
    }
}

#[cfg(not(feature = "cqrs-sqlite"))]
pub use placeholder::*;
