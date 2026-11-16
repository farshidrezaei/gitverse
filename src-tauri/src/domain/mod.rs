use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Domain errors representing failures across Git and workspace operations.
#[derive(Debug, Error, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
pub enum ApplicationError {
    #[error("Git executable was not found on the system: {0}")]
    GitBinaryNotFound(String),

    #[error("Git execution failed with exit code {exit_code}: {message}")]
    GitExecutionFailed { exit_code: i32, message: String },

    #[error("The path is not a valid Git repository: {0}")]
    InvalidRepository(String),

    #[error("Filesystem error: {0}")]
    FilesystemError(String),

    #[error("Database persistence error: {0}")]
    PersistenceError(String),

    #[error("Operation cancelled or timed out: {0}")]
    TimeoutOrCancelled(String),

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

/// Metadata and validation details of a discovered Git executable on the host system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitBinaryInformation {
    pub executable_path: String,
    pub version_string: String,
    pub is_valid: bool,
    pub has_porcelain_v2_support: bool,
}

/// High-level overview of an open Git repository on the host machine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryOverview {
    pub path: String,
    pub name: String,
    pub current_branch_name: Option<String>,
    pub head_commit_hash: Option<String>,
    pub is_empty: bool,
    pub is_detached_head: bool,
    pub working_tree_is_clean: bool,
    pub ahead_count: u32,
    pub behind_count: u32,
}

/// Persisted record of a previously opened repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecentRepositoryRecord {
    pub id: String,
    pub path: String,
    pub name: String,
    pub last_opened_timestamp: i64,
    pub is_pinned: bool,
}
