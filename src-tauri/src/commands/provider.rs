use tauri::State;
use crate::application::ApplicationState;
use crate::domain::{ApplicationError, ProviderAccount, RemoteProjectRepository, RemotePullRequest};

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
