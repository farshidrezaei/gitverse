use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use reqwest::Client;
use serde::Deserialize;
use chrono::Utc;
use crate::domain::{
    ApplicationError, ProviderAccount, ProviderKind,
    RemoteProjectRepository, RemotePullRequest, PullRequestState,
};

const DEFAULT_GITLAB_API_URL: &str = "https://gitlab.com/api/v4";
const GITLAB_USER_AGENT: &str = "GitVerse-Desktop";

/// HTTP client adapter communicating with the GitLab REST API v4.
#[derive(Debug, Clone)]
pub struct GitLabClient {
    client: Client,
    api_base_url: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitLabUserResponse {
    id: u64,
    username: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
    web_url: String,
}

#[derive(Debug, Deserialize)]
struct GitLabProjectResponse {
    id: u64,
    name: String,
    path_with_namespace: String,
    description: Option<String>,
    visibility: Option<String>,
    forked_from_project: Option<serde_json::Value>,
    default_branch: Option<String>,
    http_url_to_repo: String,
    ssh_url_to_repo: String,
    web_url: String,
    open_issues_count: Option<u32>,
    star_count: Option<u32>,
    forks_count: Option<u32>,
    last_activity_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitLabMergeRequestAuthor {
    username: String,
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitLabMergeRequestResponse {
    id: u64,
    iid: u64,
    title: String,
    description: Option<String>,
    state: String,
    draft: Option<bool>,
    work_in_progress: Option<bool>,
    author: GitLabMergeRequestAuthor,
    source_branch: String,
    target_branch: String,
    user_notes_count: Option<u32>,
    web_url: String,
    created_at: String,
    updated_at: String,
    merged_at: Option<String>,
}

impl GitLabClient {
    pub fn new(instance_url: Option<&str>) -> Result<Self, ApplicationError> {
        let api_base_url = match instance_url {
            Some(url) if !url.trim().is_empty() => {
                let trimmed = url.trim().trim_end_matches('/');
                if trimmed == "https://gitlab.com" {
                    DEFAULT_GITLAB_API_URL.to_string()
                } else {
                    format!("{}/api/v4", trimmed)
                }
            }
            _ => DEFAULT_GITLAB_API_URL.to_string(),
        };

        let client = Client::builder()
            .build()
            .map_err(|error| ApplicationError::ProviderError(format!("Failed to build HTTP client: {}", error)))?;

        Ok(Self { client, api_base_url })
    }

    fn build_headers(&self, token: &str) -> Result<HeaderMap, ApplicationError> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(GITLAB_USER_AGENT));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let mut token_header = HeaderValue::from_str(token.trim()).map_err(|_| {
            ApplicationError::AuthenticationError("Invalid GitLab personal access token format".into())
        })?;
        token_header.set_sensitive(true);
        headers.insert("PRIVATE-TOKEN", token_header);

        Ok(headers)
    }

    /// Fetches authenticated GitLab user profile to validate credentials.
    pub async fn get_authenticated_user(&self, token: &str, instance_url: &str) -> Result<ProviderAccount, ApplicationError> {
        let url = format!("{}/user", self.api_base_url);
        let headers = self.build_headers(token)?;

        let response = self.client.get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitLab network error: {}", error)))?;

        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(ApplicationError::AuthenticationError(
                "Invalid GitLab token or insufficient permissions".into(),
            ));
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!(
                "GitLab API error (HTTP {}): {}",
                status, error_text
            )));
        }

        let user_data: GitLabUserResponse = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse GitLab user response: {}", error))
        })?;

        let account = ProviderAccount {
            id: format!("gitlab:{}", user_data.id),
            provider: ProviderKind::GitLab,
            username: user_data.username,
            display_name: user_data.name,
            email: user_data.email,
            avatar_url: user_data.avatar_url,
            instance_url: instance_url.to_string(),
            created_at_timestamp: Utc::now().timestamp(),
        };

        Ok(account)
    }

    /// Lists projects accessible to the authenticated GitLab user.
    pub async fn list_projects(
        &self,
        token: &str,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<RemoteProjectRepository>, ApplicationError> {
        let url = format!("{}/projects", self.api_base_url);
        let headers = self.build_headers(token)?;

        let response = self.client.get(&url)
            .headers(headers)
            .query(&[
                ("membership", "true"),
                ("simple", "true"),
                ("page", &page.to_string()),
                ("per_page", &per_page.clamp(1, 100).to_string()),
                ("order_by", "last_activity_at"),
                ("sort", "desc"),
            ])
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitLab network error: {}", error)))?;

        if !response.status().is_success() {
            let error_message = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!("GitLab list projects failed: {}", error_message)));
        }

        let raw_projects: Vec<GitLabProjectResponse> = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse GitLab projects: {}", error))
        })?;

        let projects = raw_projects
            .into_iter()
            .map(|raw| {
                let updated_timestamp = raw.last_activity_at
                    .as_deref()
                    .and_then(|iso| chrono::DateTime::parse_from_rfc3339(iso).ok())
                    .map(|datetime| datetime.timestamp())
                    .unwrap_or_else(|| Utc::now().timestamp());

                let is_private = raw.visibility.as_deref() != Some("public");
                let is_fork = raw.forked_from_project.is_some();

                RemoteProjectRepository {
                    id: format!("gitlab:{}", raw.id),
                    name: raw.name,
                    full_name: raw.path_with_namespace,
                    description: raw.description,
                    is_private,
                    is_fork,
                    default_branch: raw.default_branch.unwrap_or_else(|| "main".to_string()),
                    clone_url_http: raw.http_url_to_repo,
                    clone_url_ssh: raw.ssh_url_to_repo,
                    html_url: raw.web_url,
                    open_issues_count: raw.open_issues_count.unwrap_or(0),
                    stars_count: raw.star_count.unwrap_or(0),
                    forks_count: raw.forks_count.unwrap_or(0),
                    updated_at_timestamp: updated_timestamp,
                }
            })
            .collect();

        Ok(projects)
    }

    /// Lists merge requests for a project ID or encoded path.
    pub async fn list_merge_requests(
        &self,
        token: &str,
        project_id_or_path: &str,
        state_filter: Option<&str>,
    ) -> Result<Vec<RemotePullRequest>, ApplicationError> {
        let encoded_project = urlencoding_simple(project_id_or_path);
        let url = format!("{}/projects/{}/merge_requests", self.api_base_url, encoded_project);
        let headers = self.build_headers(token)?;

        let state = match state_filter {
            Some("open") => "opened",
            Some("closed") => "closed",
            Some("merged") => "merged",
            _ => "all",
        };

        let response = self.client.get(&url)
            .headers(headers)
            .query(&[("state", state), ("per_page", "50")])
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitLab network error: {}", error)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!("Failed to fetch merge requests: {}", error_text)));
        }

        let raw_mrs: Vec<GitLabMergeRequestResponse> = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse merge requests: {}", error))
        })?;

        let merge_requests = raw_mrs
            .into_iter()
            .map(|raw| {
                let state = match raw.state.as_str() {
                    "merged" => PullRequestState::Merged,
                    "closed" => PullRequestState::Closed,
                    _ => PullRequestState::Open,
                };

                let created_timestamp = chrono::DateTime::parse_from_rfc3339(&raw.created_at)
                    .map(|datetime| datetime.timestamp())
                    .unwrap_or_else(|_| Utc::now().timestamp());

                let updated_timestamp = chrono::DateTime::parse_from_rfc3339(&raw.updated_at)
                    .map(|datetime| datetime.timestamp())
                    .unwrap_or_else(|_| Utc::now().timestamp());

                let merged_timestamp = raw.merged_at
                    .as_deref()
                    .and_then(|iso| chrono::DateTime::parse_from_rfc3339(iso).ok())
                    .map(|datetime| datetime.timestamp());

                let is_draft = raw.draft.unwrap_or(false) || raw.work_in_progress.unwrap_or(false);

                RemotePullRequest {
                    id: format!("gitlab:{}", raw.id),
                    number: raw.iid,
                    title: raw.title,
                    description: raw.description,
                    state,
                    author_username: raw.author.username,
                    author_avatar_url: raw.author.avatar_url,
                    source_branch: raw.source_branch,
                    target_branch: raw.target_branch,
                    is_draft,
                    comments_count: raw.user_notes_count.unwrap_or(0),
                    html_url: raw.web_url,
                    created_at_timestamp: created_timestamp,
                    updated_at_timestamp: updated_timestamp,
                    merged_at_timestamp: merged_timestamp,
                    pipeline_status: None,
                }
            })
            .collect();

        Ok(merge_requests)
    }
}

fn urlencoding_simple(raw: &str) -> String {
    raw.replace('/', "%2F")
}
