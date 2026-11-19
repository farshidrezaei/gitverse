use std::path::Path;
use std::sync::Mutex;
use rusqlite::{params, Connection};
use uuid::Uuid;
use chrono::Utc;
use crate::domain::{ApplicationError, ProviderAccount, ProviderKind, RecentRepositoryRecord};

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

            CREATE TABLE IF NOT EXISTS provider_accounts (
                id TEXT PRIMARY KEY,
                provider TEXT NOT NULL,
                username TEXT NOT NULL,
                display_name TEXT,
                email TEXT,
                avatar_url TEXT,
                instance_url TEXT NOT NULL,
                created_at_timestamp INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_provider_accounts_lookup
            ON provider_accounts(provider, username);
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
            is_missing: false,
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
                let repository_path: String = row.get(1)?;
                let is_missing = !std::path::Path::new(&repository_path).is_dir();
                Ok(RecentRepositoryRecord {
                    id: row.get(0)?,
                    path: repository_path,
                    name: row.get(2)?,
                    last_opened_timestamp: row.get(3)?,
                    is_pinned: is_pinned_int == 1,
                    is_missing,
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

    /// Clears all repositories from the recents list.
    pub fn clear_recent_repositories(&self) -> Result<(), ApplicationError> {
        let connection = self.connection.lock().map_err(|_| {
            ApplicationError::PersistenceError("Failed to acquire database lock".into())
        })?;

        connection.execute(
            "DELETE FROM recent_repositories",
            [],
        ).map_err(|error| ApplicationError::PersistenceError(format!("Clearing recent repositories failed: {}", error)))?;

        Ok(())
    }

    /// Stores or updates a connected remote Git provider account profile.
    pub fn save_provider_account(&self, account: &ProviderAccount) -> Result<(), ApplicationError> {
        let connection = self.connection.lock().map_err(|_| {
            ApplicationError::PersistenceError("Failed to acquire database lock".into())
        })?;

        let provider_string = match account.provider {
            ProviderKind::GitHub => "github",
            ProviderKind::GitLab => "gitlab",
        };

        connection.execute(
            r#"
            INSERT INTO provider_accounts (
                id, provider, username, display_name, email, avatar_url, instance_url, created_at_timestamp
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                display_name = excluded.display_name,
                email = excluded.email,
                avatar_url = excluded.avatar_url,
                instance_url = excluded.instance_url
            "#,
            params![
                account.id,
                provider_string,
                account.username,
                account.display_name,
                account.email,
                account.avatar_url,
                account.instance_url,
                account.created_at_timestamp,
            ],
        ).map_err(|error| ApplicationError::PersistenceError(format!("Failed to save provider account: {}", error)))?;

        Ok(())
    }

    /// Lists all connected provider accounts.
    pub fn list_provider_accounts(&self) -> Result<Vec<ProviderAccount>, ApplicationError> {
        let connection = self.connection.lock().map_err(|_| {
            ApplicationError::PersistenceError("Failed to acquire database lock".into())
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT id, provider, username, display_name, email, avatar_url, instance_url, created_at_timestamp
            FROM provider_accounts
            ORDER BY created_at_timestamp ASC
            "#
        ).map_err(|error| ApplicationError::PersistenceError(format!("Query preparation failed: {}", error)))?;

        let rows = statement.query_map([], |row| {
            let provider_raw: String = row.get(1)?;
            let provider = match provider_raw.as_str() {
                "gitlab" => ProviderKind::GitLab,
                _ => ProviderKind::GitHub,
            };

            Ok(ProviderAccount {
                id: row.get(0)?,
                provider,
                username: row.get(2)?,
                display_name: row.get(3)?,
                email: row.get(4)?,
                avatar_url: row.get(5)?,
                instance_url: row.get(6)?,
                created_at_timestamp: row.get(7)?,
            })
        }).map_err(|error| ApplicationError::PersistenceError(format!("Query execution failed: {}", error)))?;

        let mut accounts = Vec::new();
        for row in rows {
            if let Ok(account) = row {
                accounts.push(account);
            }
        }

        Ok(accounts)
    }

    /// Deletes a connected provider account by ID.
    pub fn delete_provider_account(&self, account_identifier: &str) -> Result<bool, ApplicationError> {
        let connection = self.connection.lock().map_err(|_| {
            ApplicationError::PersistenceError("Failed to acquire database lock".into())
        })?;

        let rows_affected = connection.execute(
            "DELETE FROM provider_accounts WHERE id = ?1",
            params![account_identifier],
        ).map_err(|error| ApplicationError::PersistenceError(format!("Failed to delete provider account: {}", error)))?;

        Ok(rows_affected > 0)
    }
}
