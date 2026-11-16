use std::path::PathBuf;
use tauri::State;
use crate::application::ApplicationState;
use crate::domain::{
    ApplicationError, GitBinaryInformation, RecentRepositoryRecord, RepositoryOverview,
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
    state: State<'_, ApplicationState>,
    directory_path: String,
) -> Result<RepositoryOverview, ApplicationError> {
    let path = PathBuf::from(&directory_path);
    state.open_repository(&path).await
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
