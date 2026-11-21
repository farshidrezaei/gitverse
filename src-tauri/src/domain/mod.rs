use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod branch;
pub mod graph;
pub mod provider;
pub mod validation;
pub use branch::*;
pub use graph::*;
pub use provider::*;
pub use validation::*;

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

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Remote provider error: {0}")]
    ProviderError(String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("Provider rate limit exceeded. Retry after {retry_after_seconds} seconds")]
    RateLimitExceeded { retry_after_seconds: u64 },

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
    #[serde(default)]
    pub is_missing: bool,
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

/// Represents Git operations that may be currently in progress (e.g. merge, rebase, cherry-pick).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryOperationState {
    #[default]
    None,
    Merging,
    Rebasing,
    CherryPicking,
    Reverting,
}

/// Detailed 3-way conflict stages and working file content for a conflicted file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConflictFileDetails {
    pub file_path: String,
    pub base_content: Option<String>,
    pub ours_content: Option<String>,
    pub theirs_content: Option<String>,
    pub working_file_content: String,
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
    pub operation_state: RepositoryOperationState,
}

/// Diff line type representation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiffLineType {
    Context,
    Addition,
    Deletion,
    Header,
    Modification,
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
    pub old_content_base64: Option<String>,
    pub new_content_base64: Option<String>,
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

/// Digital signature status of a Git commit (GPG / SSH / S/MIME).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommitSignatureStatus {
    Verified,
    Unverified,
    None,
}

/// Information about a commit's cryptographic signature.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitSignature {
    pub status: CommitSignatureStatus,
    pub signer_name: Option<String>,
    pub key_id: Option<String>,
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
    pub signature: Option<CommitSignature>,
}

/// Represents an entry in the repository stash stack.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StashRecord {
    pub index: usize,
    pub stash_identifier: String,
    pub message: String,
    pub commit_hash: String,
    pub timestamp_seconds: i64,
}

/// A single record in the repository's reference log (Reflog).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReflogRecord {
    pub selector: String,
    pub commit_hash: String,
    pub full_hash: String,
    pub action: String,
    pub timestamp_seconds: i64,
    pub author_name: String,
}

/// Detailed view of a stash entry with changed files and statistics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StashDetails {
    pub index: usize,
    pub stash_identifier: String,
    pub message: String,
    pub commit_hash: String,
    pub timestamp_seconds: i64,
    pub total_additions: u32,
    pub total_deletions: u32,
    pub changed_files: Vec<CommitChangedFile>,
}

/// Represents the outcome of merging a branch.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MergeBranchOutcome {
    pub is_successful: bool,
    pub has_conflicts: bool,
    pub message: String,
}

/// Represents the outcome of rebasing on top of an upstream branch.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RebaseBranchOutcome {
    pub is_successful: bool,
    pub has_conflicts: bool,
    pub message: String,
}

/// An item in an interactive rebase plan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RebasePlanItem {
    pub action: String,
    pub commit_hash: String,
    pub short_hash: String,
    pub original_subject: String,
    pub new_message: Option<String>,
    pub author_name: String,
}

/// Request to execute an interactive rebase.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InteractiveRebaseRequest {
    pub base_commit_hash: String,
    pub plan_items: Vec<RebasePlanItem>,
}

/// System diagnostics and environment information for voluntary bug reporting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApplicationDiagnostics {
    pub app_version: String,
    pub os_name: String,
    pub os_arch: String,
    pub git_binary_path: String,
    pub git_version: String,
    pub current_repository_path: Option<String>,
}

/// Detailed comparison between two commits or branches.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitComparison {
    pub base_hash: String,
    pub target_hash: String,
    pub base_subject: Option<String>,
    pub target_subject: Option<String>,
    pub total_additions: u32,
    pub total_deletions: u32,
    pub changed_files: Vec<CommitChangedFile>,
}

/// Directory entry returned by filesystem folder inspection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FolderEntry {
    pub name: String,
    pub path: String,
    pub is_git_repository: bool,
}

/// Result of querying a directory for subfolders.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirectoryListing {
    pub current_path: String,
    pub parent_path: Option<String>,
    pub folders: Vec<FolderEntry>,
    pub is_current_git_repository: bool,
}

/// A single annotated line within a Git file blame.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlameHunk {
    pub commit_hash: String,
    pub author_name: String,
    pub author_email: String,
    pub author_timestamp: i64,
    pub commit_summary: String,
    pub original_line_number: u32,
    pub final_line_number: u32,
    pub line_content: String,
}

/// Complete annotated history of all lines in a file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileBlame {
    pub file_path: String,
    pub lines: Vec<BlameHunk>,
}

/// Represents a Git worktree linked to a repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitWorktreeSummary {
    pub directory_path: String,
    pub head_commit_hash: String,
    pub branch_name: Option<String>,
    pub is_bare: bool,
    pub is_locked: bool,
    pub lock_reason: Option<String>,
    pub is_prunable: bool,
    pub is_main: bool,
}

/// Represents the state of an interactive Git bisect session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BisectState {
    pub is_active: bool,
    pub current_commit_hash: Option<String>,
    pub current_commit_summary: Option<String>,
    pub revisions_remaining: Option<u32>,
    pub estimated_steps_remaining: Option<u32>,
    pub culprit_commit_hash: Option<String>,
    pub culprit_commit_summary: Option<String>,
    pub log_output: String,
}

/// Result of verifying if a patch file applies cleanly without conflicts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PatchCheckResult {
    pub can_apply: bool,
    pub error_message: Option<String>,
    pub affected_files: Vec<String>,
}

/// Represents a Git submodule linked within a repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitSubmoduleSummary {
    pub path: String,
    pub name: String,
    pub url: Option<String>,
    pub branch: Option<String>,
    pub current_commit_hash: String,
    pub is_initialized: bool,
    pub has_uncommitted_changes: bool,
    pub has_revision_divergence: bool,
}

/// Individual contributor commit volume metrics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContributorInsight {
    pub name: String,
    pub email: String,
    pub commit_count: u32,
    pub percentage: f32,
}

/// Aggregated commit activity count by weekday (0 = Sunday, 1 = Monday, ... 6 = Saturday).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivityDay {
    pub day_of_week: u8,
    pub count: u32,
}

/// Comprehensive analytics, velocity metrics, and commit statistics for a repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepositoryInsights {
    pub total_commits: u32,
    pub total_contributors: u32,
    pub top_contributors: Vec<ContributorInsight>,
    pub activity_by_weekday: Vec<ActivityDay>,
    pub lines_added: u64,
    pub lines_deleted: u64,
}

/// A single historical commit revision entry touching a specific file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileRevisionEntry {
    pub commit_hash: String,
    pub author_name: String,
    pub author_email: String,
    pub author_timestamp: i64,
    pub commit_summary: String,
    pub change_status: String,
}

/// Chronological history timeline of a file across commits.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileHistoryTimeline {
    pub file_path: String,
    pub revisions: Vec<FileRevisionEntry>,
    pub total_revisions: u32,
}

/// A single matching line from repository code search.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchMatchEntry {
    pub file_path: String,
    pub line_number: u32,
    pub line_content: String,
}

/// Aggregated results from a code or history search.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositorySearchResult {
    pub query: String,
    pub total_matches: u32,
    pub file_count: u32,
    pub matches: Vec<SearchMatchEntry>,
}

/// Request payload to execute multi-commit cherry-pick or revert sequence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitQueueOperationRequest {
    pub commit_hashes: Vec<String>,
    pub should_commit: bool,
    pub custom_message: Option<String>,
}

/// Result status of a multi-commit queue operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitQueueOperationResult {
    pub is_successful: bool,
    pub completed_commits: Vec<String>,
    pub failed_commit_hash: Option<String>,
    pub conflict_files: Vec<String>,
    pub message: String,
}






