use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod branch;
pub mod graph;
pub use branch::*;
pub use graph::*;

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

/// Representation of a single file's change status in the working tree or staging index.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileChangeStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    TypeChanged,
    Untracked,
    Conflicted,
}

/// Detailed file status entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileStatusEntry {
    pub path: String,
    pub original_path: Option<String>,
    pub status: FileChangeStatus,
    pub is_staged: bool,
}

/// Complete parsed snapshot of working tree and staging area.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DetailedRepositoryStatus {
    pub branch_name: Option<String>,
    pub head_commit_hash: Option<String>,
    pub upstream_branch_name: Option<String>,
    pub ahead_count: u32,
    pub behind_count: u32,
    pub is_detached_head: bool,
    pub staged_files: Vec<FileStatusEntry>,
    pub unstaged_files: Vec<FileStatusEntry>,
    pub conflicted_files: Vec<FileStatusEntry>,
}

/// Diff line type representation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiffLineType {
    Context,
    Addition,
    Deletion,
    Header,
}

/// Individual line in a diff hunk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub old_line_number: Option<u32>,
    pub new_line_number: Option<u32>,
    pub content: String,
}

/// A contiguous block (hunk) of changes in a file diff.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiffHunk {
    pub header: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

/// Structured diff result for a file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileDiff {
    pub old_path: Option<String>,
    pub new_path: String,
    pub is_binary: bool,
    pub is_new_file: bool,
    pub is_deleted_file: bool,
    pub hunks: Vec<DiffHunk>,
    pub total_additions: u32,
    pub total_deletions: u32,
}

/// Represents a file modified in a specific commit with status and diff counts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitChangedFile {
    pub path: String,
    pub original_path: Option<String>,
    pub status: FileChangeStatus,
    pub additions: u32,
    pub deletions: u32,
}

/// Comprehensive commit inspection details including metadata and changed files list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitDetails {
    pub hash: String,
    pub parent_hashes: Vec<String>,
    pub author_name: String,
    pub author_email: String,
    pub author_timestamp: i64,
    pub committer_name: String,
    pub committer_email: String,
    pub committer_timestamp: i64,
    pub subject: String,
    pub body: String,
    pub decorations: Vec<GitCommitDecoration>,
    pub total_additions: u32,
    pub total_deletions: u32,
    pub changed_files: Vec<CommitChangedFile>,
}



