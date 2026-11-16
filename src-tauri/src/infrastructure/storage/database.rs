use std::path::Path;
use std::sync::Mutex;
use rusqlite::{params, Connection};
use uuid::Uuid;
use chrono::Utc;
use crate::domain::{ApplicationError, RecentRepositoryRecord};

/// Persistent SQLite store for recent repositories, bookmarks, and local workspace state.
pub struct SQLiteRepositoryStore {
    connection: Mutex<Connection>,
}

impl SQLiteRepositoryStore {
    pub fn new(database_path: &Path) -> Result<Self, ApplicationError> {
        let connection = Connection::open(database_path)
            .map_err(|error| ApplicationError::PersistenceError(format!("Failed to open SQLite database: {}", error)))?;

        let store = Self {
            connection: Mutex::new(connection),
        };

        store.run_migrations()?;
        Ok(store)
    }

    /// Initializes local SQLite database tables if not existing.
    fn run_migrations(&self) -> Result<(), ApplicationError> {
        let connection = self.connection.lock().map_err(|_| {
            ApplicationError::PersistenceError("Failed to acquire database lock".into())
        })?;

        connection.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS recent_repositories (
                id TEXT PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                last_opened_timestamp INTEGER NOT NULL,
                is_pinned INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_recent_repositories_last_opened
            ON recent_repositories(last_opened_timestamp DESC);
            "#
        ).map_err(|error| ApplicationError::PersistenceError(format!("Migration failed: {}", error)))?;

        Ok(())
    }

    /// Records or updates an opened repository path in the recent repositories history.
    pub fn record_opened_repository(&self, path: &str, name: &str) -> Result<RecentRepositoryRecord, ApplicationError> {
        let connection = self.connection.lock().map_err(|_| {
            ApplicationError::PersistenceError("Failed to acquire database lock".into())
        })?;

        let timestamp = Utc::now().timestamp();
        let existing_id: Option<String> = connection
            .query_row(
                "SELECT id FROM recent_repositories WHERE path = ?1",
                params![path],
                |row| row.get(0),
            )
            .ok();

        let id = if let Some(existing) = existing_id {
            connection.execute(
                "UPDATE recent_repositories SET name = ?1, last_opened_timestamp = ?2 WHERE id = ?3",
                params![name, timestamp, existing],
            ).map_err(|error| ApplicationError::PersistenceError(format!("Update failed: {}", error)))?;
            existing
        } else {
            let new_id = Uuid::new_v4().to_string();
            connection.execute(
                "INSERT INTO recent_repositories (id, path, name, last_opened_timestamp, is_pinned) VALUES (?1, ?2, ?3, ?4, 0)",
                params![new_id, path, name, timestamp],
            ).map_err(|error| ApplicationError::PersistenceError(format!("Insert failed: {}", error)))?;
            new_id
        };

        Ok(RecentRepositoryRecord {
            id,
            path: path.to_string(),
            name: name.to_string(),
            last_opened_timestamp: timestamp,
            is_pinned: false,
        })
    }

    /// Retrieves all recent repositories ordered by last opened timestamp.
    pub fn get_recent_repositories(&self, limit: usize) -> Result<Vec<RecentRepositoryRecord>, ApplicationError> {
        let connection = self.connection.lock().map_err(|_| {
            ApplicationError::PersistenceError("Failed to acquire database lock".into())
        })?;

        let mut statement = connection
            .prepare(
                "SELECT id, path, name, last_opened_timestamp, is_pinned FROM recent_repositories ORDER BY is_pinned DESC, last_opened_timestamp DESC LIMIT ?1",
            )
            .map_err(|error| ApplicationError::PersistenceError(format!("Query preparation failed: {}", error)))?;

        let rows = statement
            .query_map(params![limit as i64], |row| {
                let is_pinned_int: i32 = row.get(4)?;
                Ok(RecentRepositoryRecord {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    name: row.get(2)?,
                    last_opened_timestamp: row.get(3)?,
                    is_pinned: is_pinned_int == 1,
                })
            })
            .map_err(|error| ApplicationError::PersistenceError(format!("Query execution failed: {}", error)))?;

        let mut results = Vec::new();
        for row in rows {
            if let Ok(record) = row {
                results.push(record);
            }
        }

        Ok(results)
    }

    /// Removes a repository from the recents list.
    pub fn remove_recent_repository(&self, path: &str) -> Result<(), ApplicationError> {
        let connection = self.connection.lock().map_err(|_| {
            ApplicationError::PersistenceError("Failed to acquire database lock".into())
        })?;

        connection.execute(
            "DELETE FROM recent_repositories WHERE path = ?1",
            params![path],
        ).map_err(|error| ApplicationError::PersistenceError(format!("Deletion failed: {}", error)))?;

        Ok(())
    }
}
