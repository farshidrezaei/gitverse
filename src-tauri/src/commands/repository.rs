use std::path::PathBuf;
use tauri::{AppHandle, State};
use crate::application::ApplicationState;
use crate::domain::{
    ApplicationError, DetailedRepositoryStatus, GitBinaryInformation, RecentRepositoryRecord, RepositoryOverview,
};

/// Tauri IPC command to discover and validate the native Git binary.
#[tauri::command]
pub async fn discover_git_binary(
    state: State<'_, ApplicationState>,
    custom_path: Option<String>,
) -> Result<GitBinaryInformation, ApplicationError> {
    state.discover_git(custom_path.as_deref()).await
}

/// Tauri IPC command to open a Git repository from a directory path.
#[tauri::command]
pub async fn open_repository(
    app_handle: AppHandle,
    state: State<'_, ApplicationState>,
    directory_path: String,
) -> Result<RepositoryOverview, ApplicationError> {
    let path = PathBuf::from(&directory_path);
    state.open_repository(app_handle, &path).await
}

/// Tauri IPC command to get currently active repository.
#[tauri::command]
pub async fn get_current_repository(
    state: State<'_, ApplicationState>,
) -> Result<Option<RepositoryOverview>, ApplicationError> {
    Ok(state.get_current_repository().await)
}

/// Tauri IPC command to close the currently active repository.
#[tauri::command]
pub async fn close_repository(
    state: State<'_, ApplicationState>,
) -> Result<(), ApplicationError> {
    state.close_current_repository().await;
    Ok(())
}

/// Tauri IPC command to get detailed status (staged, unstaged, conflicted files).
#[tauri::command]
pub async fn get_repository_status(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
) -> Result<DetailedRepositoryStatus, ApplicationError> {
    state.get_repository_status(directory_path.as_deref()).await
}

/// Tauri IPC command to stage a file.
#[tauri::command]
pub async fn stage_file(
    state: State<'_, ApplicationState>,
    file_path: String,
    directory_path: Option<String>,
) -> Result<(), ApplicationError> {
    state.stage_file(directory_path.as_deref(), &file_path).await
}

/// Tauri IPC command to unstage a file.
#[tauri::command]
pub async fn unstage_file(
    state: State<'_, ApplicationState>,
    file_path: String,
    directory_path: Option<String>,
) -> Result<(), ApplicationError> {
    state.unstage_file(directory_path.as_deref(), &file_path).await
}

/// Tauri IPC command to stage all changes.
#[tauri::command]
pub async fn stage_all(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
) -> Result<(), ApplicationError> {
    state.stage_all(directory_path.as_deref()).await
}

/// Tauri IPC command to unstage all changes.
#[tauri::command]
pub async fn unstage_all(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
) -> Result<(), ApplicationError> {
    state.unstage_all(directory_path.as_deref()).await
}

/// Tauri IPC command to discard changes for a file.
#[tauri::command]
pub async fn discard_file_changes(
    state: State<'_, ApplicationState>,
    file_path: String,
    is_untracked: bool,
    directory_path: Option<String>,
) -> Result<(), ApplicationError> {
    state.discard_file_changes(directory_path.as_deref(), &file_path, is_untracked).await
}

/// Tauri IPC command to commit staged changes.
#[tauri::command]
pub async fn commit(
    state: State<'_, ApplicationState>,
    message: String,
    is_amend: bool,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.commit(directory_path.as_deref(), &message, is_amend).await
}

/// Tauri IPC command to get structured file diff.
#[tauri::command]
pub async fn get_file_diff(
    state: State<'_, ApplicationState>,
    file_path: String,
    is_staged: bool,
    directory_path: Option<String>,
) -> Result<crate::domain::FileDiff, ApplicationError> {
    state.get_file_diff(directory_path.as_deref(), &file_path, is_staged).await
}

/// Tauri IPC command to extract Git commit history and compute deterministic DAG layout.
#[tauri::command]
pub async fn get_commit_graph(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
    skip: Option<usize>,
    limit: Option<usize>,
    branch_or_reference: Option<String>,
) -> Result<crate::domain::CommitGraphHistoryResponse, ApplicationError> {
    state.get_commit_graph(directory_path.as_deref(), skip, limit, branch_or_reference).await
}

/// Tauri IPC command to fetch repository references (branches, remotes, tags).
#[tauri::command]
pub async fn get_repository_references(
    state: State<'_, ApplicationState>,
    directory_path: Option<String>,
) -> Result<crate::domain::RepositoryReferences, ApplicationError> {
    state.get_repository_references(directory_path.as_deref()).await
}

/// Tauri IPC command to checkout or switch branches.
#[tauri::command]
pub async fn checkout_branch(
    state: State<'_, ApplicationState>,
    branch_name: String,
    create_if_missing: Option<bool>,
    start_point: Option<String>,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.checkout_branch(
        directory_path.as_deref(),
        &branch_name,
        create_if_missing.unwrap_or(false),
        start_point.as_deref(),
    ).await
}

/// Tauri IPC command to create a new branch.
#[tauri::command]
pub async fn create_branch(
    state: State<'_, ApplicationState>,
    branch_name: String,
    start_point: Option<String>,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.create_branch(directory_path.as_deref(), &branch_name, start_point.as_deref()).await
}

/// Tauri IPC command to delete a branch.
#[tauri::command]
pub async fn delete_branch(
    state: State<'_, ApplicationState>,
    branch_name: String,
    force: Option<bool>,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.delete_branch(directory_path.as_deref(), &branch_name, force.unwrap_or(false)).await
}

/// Tauri IPC command to fetch changes from remote.
#[tauri::command]
pub async fn fetch_remote(
    state: State<'_, ApplicationState>,
    remote_name: Option<String>,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.fetch_remote(directory_path.as_deref(), remote_name.as_deref()).await
}

/// Tauri IPC command to pull changes.
#[tauri::command]
pub async fn pull_changes(
    state: State<'_, ApplicationState>,
    remote_name: Option<String>,
    branch_name: Option<String>,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.pull_changes(directory_path.as_deref(), remote_name.as_deref(), branch_name.as_deref()).await
}

/// Tauri IPC command to push commits.
#[tauri::command]
pub async fn push_changes(
    state: State<'_, ApplicationState>,
    remote_name: Option<String>,
    branch_name: Option<String>,
    set_upstream: Option<bool>,
    directory_path: Option<String>,
) -> Result<String, ApplicationError> {
    state.push_changes(
        directory_path.as_deref(),
        remote_name.as_deref(),
        branch_name.as_deref(),
        set_upstream.unwrap_or(false),
    ).await
}

/// Tauri IPC command to fetch recent repositories from local SQLite database.
#[tauri::command]
pub async fn get_recent_repositories(
    state: State<'_, ApplicationState>,
    limit: Option<usize>,
) -> Result<Vec<RecentRepositoryRecord>, ApplicationError> {
    let limit_value = limit.unwrap_or(20);
    state.get_recent_repositories(limit_value)
}

/// Tauri IPC command to remove a repository from recent history.
#[tauri::command]
pub async fn remove_recent_repository(
    state: State<'_, ApplicationState>,
    directory_path: String,
) -> Result<(), ApplicationError> {
    state.remove_recent_repository(&directory_path)
}

