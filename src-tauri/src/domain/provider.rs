use serde::{Deserialize, Serialize};

/// Supported remote Git forge provider platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    GitHub,
    GitLab,
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitHub => write!(formatter, "GitHub"),
            Self::GitLab => write!(formatter, "GitLab"),
        }
    }
}

/// Feature capabilities supported by a provider integration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub can_create_pull_request: bool,
    pub can_review_pull_request: bool,
    pub can_view_pipelines: bool,
    pub can_list_issues: bool,
    pub can_manage_releases: bool,
}

impl Default for ProviderCapabilities {
    fn default() -> Self {
        Self {
            can_create_pull_request: true,
            can_review_pull_request: true,
            can_view_pipelines: true,
            can_list_issues: true,
            can_manage_releases: false,
        }
    }
}

/// Current API rate limit status for an authenticated session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimitStatus {
    pub limit: u32,
    pub remaining: u32,
    pub resets_at_timestamp: i64,
}

/// Connected user account profile on a remote Git provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderAccount {
    pub id: String,
    pub provider: ProviderKind,
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub instance_url: String,
    pub created_at_timestamp: i64,
}

/// Lifecycle state of a remote pull or merge request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PullRequestState {
    Open,
    Closed,
    Merged,
}

/// Unified pull request (GitHub) or merge request (GitLab) model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemotePullRequest {
    pub id: String,
    pub number: u64,
    pub title: String,
    pub description: Option<String>,
    pub state: PullRequestState,
    pub author_username: String,
    pub author_avatar_url: Option<String>,
    pub source_branch: String,
    pub target_branch: String,
    pub is_draft: bool,
    pub comments_count: u32,
    pub html_url: String,
    pub created_at_timestamp: i64,
    pub updated_at_timestamp: i64,
    pub merged_at_timestamp: Option<i64>,
    pub pipeline_status: Option<PipelineSummary>,
}

/// Overall status of automated CI/CD pipeline or check suite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PipelineState {
    Success,
    Failed,
    Running,
    Pending,
    Canceled,
    Skipped,
}

/// Summary of an automated CI/CD pipeline run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineSummary {
    pub state: PipelineState,
    pub web_url: Option<String>,
    pub commit_sha: Option<String>,
    pub total_checks_count: u32,
    pub successful_checks_count: u32,
    pub failed_checks_count: u32,
}

/// Detailed CI check run (e.g. GitHub Actions check or GitLab job).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CiCheckRun {
    pub id: String,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub html_url: Option<String>,
    pub started_at_timestamp: Option<i64>,
    pub completed_at_timestamp: Option<i64>,
}

/// Remote review comment on a pull request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteReviewComment {
    pub id: String,
    pub author_username: String,
    pub author_avatar_url: Option<String>,
    pub body: String,
    pub path: Option<String>,
    pub line_number: Option<u32>,
    pub created_at_timestamp: i64,
    pub updated_at_timestamp: i64,
}

/// Changed file in a remote pull request with diff stats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullRequestChangedFile {
    pub file_path: String,
    pub previous_file_path: Option<String>,
    pub status: String,
    pub additions_count: u32,
    pub deletions_count: u32,
    pub changes_count: u32,
    pub patch: Option<String>,
}

/// Action taken when submitting a formal pull request review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PullRequestReviewAction {
    Approve,
    RequestChanges,
    Comment,
}

/// Merge strategy method for merging a remote pull request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestMergeMethod {
    Merge,
    Squash,
    Rebase,
}

/// Result returned from executing a remote pull request merge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullRequestMergeResult {
    pub is_merged: bool,
    pub message: String,
    pub commit_sha: Option<String>,
}

/// Remote repository project discovered from an authenticated provider account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteProjectRepository {
    pub id: String,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub is_private: bool,
    pub is_fork: bool,
    pub default_branch: String,
    pub clone_url_http: String,
    pub clone_url_ssh: String,
    pub html_url: String,
    pub open_issues_count: u32,
    pub stars_count: u32,
    pub forks_count: u32,
    pub updated_at_timestamp: i64,
}
