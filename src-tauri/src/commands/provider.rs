use tauri::State;
use crate::application::ApplicationState;
use crate::domain::{
    ApplicationError, ProviderAccount, PullRequestChangedFile, PullRequestMergeResult,
    RemoteProjectRepository, RemotePullRequest, RemoteReviewComment,
};

/// Tauri IPC command to connect a GitHub account via personal access token.
#[tauri::command]
pub async fn connect_github_account(
    state: State<'_, ApplicationState>,
    personal_access_token: String,
    instance_url: Option<String>,
) -> Result<ProviderAccount, ApplicationError> {
    state.connect_github_account(&personal_access_token, instance_url.as_deref()).await
}

/// Tauri IPC command to connect a GitLab account via personal access token.
#[tauri::command]
pub async fn connect_gitlab_account(
    state: State<'_, ApplicationState>,
    personal_access_token: String,
    instance_url: Option<String>,
) -> Result<ProviderAccount, ApplicationError> {
    state.connect_gitlab_account(&personal_access_token, instance_url.as_deref()).await
}

/// Tauri IPC command to list all connected remote provider accounts.
#[tauri::command]
pub async fn list_provider_accounts(
    state: State<'_, ApplicationState>,
) -> Result<Vec<ProviderAccount>, ApplicationError> {
    state.list_provider_accounts().await
}

/// Tauri IPC command to disconnect a remote provider account and remove credentials.
#[tauri::command]
pub async fn disconnect_provider_account(
    state: State<'_, ApplicationState>,
    account_identifier: String,
) -> Result<bool, ApplicationError> {
    state.disconnect_provider_account(&account_identifier).await
}

/// Tauri IPC command to fetch remote projects/repositories for an account.
#[tauri::command]
pub async fn get_provider_repositories(
    state: State<'_, ApplicationState>,
    account_identifier: String,
    page: Option<u32>,
    per_page: Option<u32>,
) -> Result<Vec<RemoteProjectRepository>, ApplicationError> {
    state.get_provider_repositories(&account_identifier, page, per_page).await
}

/// Tauri IPC command to fetch pull requests or merge requests for a repository.
#[tauri::command]
pub async fn get_provider_pull_requests(
    state: State<'_, ApplicationState>,
    account_identifier: String,
    repository_full_name: String,
    state_filter: Option<String>,
) -> Result<Vec<RemotePullRequest>, ApplicationError> {
    state.get_provider_pull_requests(&account_identifier, &repository_full_name, state_filter.as_deref()).await
}

/// Tauri IPC command to fetch commit pipeline status summary.
#[tauri::command]
pub async fn get_commit_pipeline_summary(
    state: State<'_, ApplicationState>,
    account_identifier: String,
    repository_full_name: String,
    reference_or_sha: String,
) -> Result<Option<crate::domain::PipelineSummary>, ApplicationError> {
    state.get_commit_pipeline_summary(&account_identifier, &repository_full_name, &reference_or_sha).await
}

/// Tauri IPC command to fetch detailed check runs or CI jobs.
#[tauri::command]
pub async fn list_check_runs(
    state: State<'_, ApplicationState>,
    account_identifier: String,
    repository_full_name: String,
    reference_or_pipeline_id: String,
) -> Result<Vec<crate::domain::CiCheckRun>, ApplicationError> {
    state.list_check_runs(&account_identifier, &repository_full_name, &reference_or_pipeline_id).await
}

/// Tauri IPC command to fetch comments on a pull or merge request.
#[tauri::command]
pub async fn get_pull_request_comments(
    state: State<'_, ApplicationState>,
    account_identifier: String,
    repository_full_name: String,
    pull_request_number: u64,
) -> Result<Vec<RemoteReviewComment>, ApplicationError> {
    state.get_pull_request_comments(&account_identifier, &repository_full_name, pull_request_number).await
}

/// Tauri IPC command to post a new comment to a pull or merge request.
#[tauri::command]
pub async fn create_pull_request_comment(
    state: State<'_, ApplicationState>,
    account_identifier: String,
    repository_full_name: String,
    pull_request_number: u64,
    body: String,
) -> Result<RemoteReviewComment, ApplicationError> {
    state.create_pull_request_comment(&account_identifier, &repository_full_name, pull_request_number, &body).await
}

/// Tauri IPC command to fetch changed files and diff patches for a pull request.
#[tauri::command]
pub async fn get_pull_request_files(
    state: State<'_, ApplicationState>,
    account_identifier: String,
    repository_full_name: String,
    pull_request_number: u64,
) -> Result<Vec<PullRequestChangedFile>, ApplicationError> {
    state.get_pull_request_files(&account_identifier, &repository_full_name, pull_request_number).await
}

/// Tauri IPC command to submit a formal review (Approve, Request Changes, Comment) on a PR.
#[tauri::command]
pub async fn submit_pull_request_review(
    state: State<'_, ApplicationState>,
    account_identifier: String,
    repository_full_name: String,
    pull_request_number: u64,
    action: String,
    body: Option<String>,
) -> Result<bool, ApplicationError> {
    state.submit_pull_request_review(&account_identifier, &repository_full_name, pull_request_number, &action, body.as_deref()).await
}

/// Tauri IPC command to merge a pull or merge request.
#[tauri::command]
pub async fn merge_provider_pull_request(
    state: State<'_, ApplicationState>,
    account_identifier: String,
    repository_full_name: String,
    pull_request_number: u64,
    merge_method: Option<String>,
    commit_title: Option<String>,
    commit_message: Option<String>,
    delete_source_branch: Option<bool>,
) -> Result<PullRequestMergeResult, ApplicationError> {
    state.merge_provider_pull_request(
        &account_identifier,
        &repository_full_name,
        pull_request_number,
        merge_method.as_deref(),
        commit_title.as_deref(),
        commit_message.as_deref(),
        delete_source_branch,
    ).await
}

/// Tauri IPC command to update pull request state (closed or open).
#[tauri::command]
pub async fn update_pull_request_state(
    state: State<'_, ApplicationState>,
    account_identifier: String,
    repository_full_name: String,
    pull_request_number: u64,
    new_state: String,
) -> Result<bool, ApplicationError> {
    state.update_pull_request_state(&account_identifier, &repository_full_name, pull_request_number, &new_state).await
}
