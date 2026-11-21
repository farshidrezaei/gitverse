use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use crate::domain::{
    ApplicationError, CiCheckRun, PipelineState, PipelineSummary,
    ProviderAccount, ProviderKind, PullRequestChangedFile, PullRequestMergeResult,
    PullRequestState, RemoteProjectRepository, RemotePullRequest, RemoteReviewComment,
    WorkflowJob, WorkflowJobStep, WorkflowRun,
};

const DEFAULT_GITHUB_API_URL: &str = "https://api.github.com";
const GITHUB_USER_AGENT: &str = "GitVerse-Desktop";

/// HTTP client adapter communicating with the GitHub REST API.
#[derive(Debug, Clone)]
pub struct GitHubClient {
    client: Client,
    api_base_url: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubUserResponse {
    id: u64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRepositoryResponse {
    id: u64,
    name: String,
    full_name: String,
    description: Option<String>,
    private: bool,
    fork: bool,
    default_branch: Option<String>,
    clone_url: String,
    ssh_url: String,
    html_url: String,
    open_issues_count: Option<u32>,
    stargazers_count: Option<u32>,
    forks_count: Option<u32>,
    updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubPullRequestUser {
    login: String,
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubPullRequestBranch {
    #[serde(rename = "ref")]
    ref_name: String,
}

#[derive(Debug, Deserialize)]
struct GitHubPullRequestResponse {
    id: u64,
    number: u64,
    title: String,
    body: Option<String>,
    state: String,
    draft: Option<bool>,
    user: GitHubPullRequestUser,
    head: GitHubPullRequestBranch,
    base: GitHubPullRequestBranch,
    comments: Option<u32>,
    html_url: String,
    created_at: String,
    updated_at: String,
    merged_at: Option<String>,
}

impl GitHubClient {
    pub fn new(instance_url: Option<&str>) -> Result<Self, ApplicationError> {
        let api_base_url = match instance_url {
            Some(url) if !url.trim().is_empty() => {
                let trimmed = url.trim().trim_end_matches('/');
                if trimmed == "https://github.com" {
                    DEFAULT_GITHUB_API_URL.to_string()
                } else {
                    format!("{}/api/v3", trimmed)
                }
            }
            _ => DEFAULT_GITHUB_API_URL.to_string(),
        };

        let client = Client::builder()
            .build()
            .map_err(|error| ApplicationError::ProviderError(format!("Failed to build HTTP client: {}", error)))?;

        Ok(Self { client, api_base_url })
    }

    fn build_headers(&self, token: &str) -> Result<HeaderMap, ApplicationError> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(GITHUB_USER_AGENT));
        headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.github.v3+json"));

        let auth_value = format!("Bearer {}", token.trim());
        let mut header_value = HeaderValue::from_str(&auth_value).map_err(|_| {
            ApplicationError::AuthenticationError("Invalid personal access token format".into())
        })?;
        header_value.set_sensitive(true);
        headers.insert(AUTHORIZATION, header_value);

        Ok(headers)
    }

    /// Fetches the authenticated user profile to validate credentials.
    pub async fn get_authenticated_user(&self, token: &str, instance_url: &str) -> Result<ProviderAccount, ApplicationError> {
        let url = format!("{}/user", self.api_base_url);
        let headers = self.build_headers(token)?;

        let response = self.client.get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitHub network error: {}", error)))?;

        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(ApplicationError::AuthenticationError(
                "Invalid GitHub token or insufficient scope".into(),
            ));
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!(
                "GitHub API error (HTTP {}): {}",
                status, error_text
            )));
        }

        let user_data: GitHubUserResponse = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse GitHub user response: {}", error))
        })?;

        let account = ProviderAccount {
            id: format!("github:{}", user_data.id),
            provider: ProviderKind::GitHub,
            username: user_data.login,
            display_name: user_data.name,
            email: user_data.email,
            avatar_url: user_data.avatar_url,
            instance_url: instance_url.to_string(),
            created_at_timestamp: Utc::now().timestamp(),
        };

        Ok(account)
    }

    /// Lists repositories accessible to the authenticated user.
    pub async fn list_repositories(
        &self,
        token: &str,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<RemoteProjectRepository>, ApplicationError> {
        let url = format!("{}/user/repos", self.api_base_url);
        let headers = self.build_headers(token)?;

        let response = self.client.get(&url)
            .headers(headers)
            .query(&[
                ("page", page.to_string()),
                ("per_page", per_page.clamp(1, 100).to_string()),
                ("sort", "updated".to_string()),
            ])
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitHub network error: {}", error)))?;

        if !response.status().is_success() {
            let error_message = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!("GitHub list repos failed: {}", error_message)));
        }

        let raw_repositories: Vec<GitHubRepositoryResponse> = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse GitHub repositories: {}", error))
        })?;

        let repositories = raw_repositories
            .into_iter()
            .map(|raw| {
                let updated_timestamp = raw.updated_at
                    .as_deref()
                    .and_then(|iso| chrono::DateTime::parse_from_rfc3339(iso).ok())
                    .map(|datetime| datetime.timestamp())
                    .unwrap_or_else(|| Utc::now().timestamp());

                RemoteProjectRepository {
                    id: format!("github:{}", raw.id),
                    name: raw.name,
                    full_name: raw.full_name,
                    description: raw.description,
                    is_private: raw.private,
                    is_fork: raw.fork,
                    default_branch: raw.default_branch.unwrap_or_else(|| "main".to_string()),
                    clone_url_http: raw.clone_url,
                    clone_url_ssh: raw.ssh_url,
                    html_url: raw.html_url,
                    open_issues_count: raw.open_issues_count.unwrap_or(0),
                    stars_count: raw.stargazers_count.unwrap_or(0),
                    forks_count: raw.forks_count.unwrap_or(0),
                    updated_at_timestamp: updated_timestamp,
                }
            })
            .collect();

        Ok(repositories)
    }

    /// Lists pull requests for a given repository owner and name.
    pub async fn list_pull_requests(
        &self,
        token: &str,
        owner: &str,
        repository_name: &str,
        state_filter: Option<&str>,
    ) -> Result<Vec<RemotePullRequest>, ApplicationError> {
        let url = format!("{}/repos/{}/{}/pulls", self.api_base_url, owner, repository_name);
        let headers = self.build_headers(token)?;

        let state = match state_filter {
            Some("open") => "open",
            Some("closed") | Some("merged") => "closed",
            _ => "all",
        };

        let response = self.client.get(&url)
            .headers(headers)
            .query(&[("state", state), ("per_page", "50")])
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitHub network error: {}", error)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!("Failed to fetch pull requests: {}", error_text)));
        }

        let raw_pulls: Vec<GitHubPullRequestResponse> = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse pull requests: {}", error))
        })?;

        let mut pull_requests: Vec<RemotePullRequest> = raw_pulls
            .into_iter()
            .filter_map(|raw| {
                let state = match raw.state.as_str() {
                    "closed" if raw.merged_at.is_some() => PullRequestState::Merged,
                    "closed" => PullRequestState::Closed,
                    _ => PullRequestState::Open,
                };

                // Filter specifically for merged vs closed if requested
                if let Some(target_state) = state_filter {
                    match target_state {
                        "open" if state != PullRequestState::Open => return None,
                        "closed" if state != PullRequestState::Closed => return None,
                        "merged" if state != PullRequestState::Merged => return None,
                        _ => {}
                    }
                }

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

                Some(RemotePullRequest {
                    id: format!("github:{}", raw.id),
                    number: raw.number,
                    title: raw.title,
                    description: raw.body,
                    state,
                    author_username: raw.user.login,
                    author_avatar_url: raw.user.avatar_url,
                    source_branch: raw.head.ref_name,
                    target_branch: raw.base.ref_name,
                    is_draft: raw.draft.unwrap_or(false),
                    comments_count: raw.comments.unwrap_or(0),
                    html_url: raw.html_url,
                    created_at_timestamp: created_timestamp,
                    updated_at_timestamp: updated_timestamp,
                    merged_at_timestamp: merged_timestamp,
                    pipeline_status: None,
                })
            })
            .collect();

        // Populate pipeline status for the first few open pull requests
        for pull_request in pull_requests.iter_mut().take(5) {
            if let Ok(summary) = self.get_commit_pipeline_summary(token, owner, repository_name, &pull_request.source_branch).await {
                pull_request.pipeline_status = summary;
            }
        }

        Ok(pull_requests)
    }

    /// Fetches commit status summary (CI check runs and commit status) for a reference or SHA.
    pub async fn get_commit_pipeline_summary(
        &self,
        token: &str,
        owner: &str,
        repository: &str,
        reference_or_sha: &str,
    ) -> Result<Option<PipelineSummary>, ApplicationError> {
        let url = format!("{}/repos/{}/{}/commits/{}/check-runs", self.api_base_url, owner, repository, reference_or_sha);
        let headers = self.build_headers(token)?;

        let response = self.client.get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitHub network error: {}", error)))?;

        if !response.status().is_success() {
            return Ok(None);
        }

        #[derive(Deserialize)]
        struct CheckRunsPayload {
            total_count: u32,
            check_runs: Vec<GitHubCheckRunItem>,
        }

        #[derive(Deserialize)]
        struct GitHubCheckRunItem {
            status: String,
            conclusion: Option<String>,
            html_url: Option<String>,
        }

        let payload: CheckRunsPayload = match response.json().await {
            Ok(parsed) => parsed,
            Err(_) => return Ok(None),
        };

        if payload.total_count == 0 {
            return Ok(None);
        }

        let mut success_count = 0;
        let mut failed_count = 0;
        let mut has_running = false;

        for check in &payload.check_runs {
            match check.conclusion.as_deref() {
                Some("success") => success_count += 1,
                Some("failure") | Some("timed_out") | Some("cancelled") => failed_count += 1,
                _ => {
                    if check.status == "in_progress" || check.status == "queued" {
                        has_running = true;
                    }
                }
            }
        }

        let pipeline_state = if failed_count > 0 {
            PipelineState::Failed
        } else if has_running {
            PipelineState::Running
        } else if success_count > 0 {
            PipelineState::Success
        } else {
            PipelineState::Pending
        };

        let first_url = payload.check_runs.first().and_then(|check| check.html_url.clone());

        Ok(Some(PipelineSummary {
            state: pipeline_state,
            web_url: first_url,
            commit_sha: Some(reference_or_sha.to_string()),
            total_checks_count: payload.total_count,
            successful_checks_count: success_count,
            failed_checks_count: failed_count,
        }))
    }

    /// Fetches detailed check runs for a commit.
    pub async fn list_check_runs(
        &self,
        token: &str,
        owner: &str,
        repository: &str,
        commit_sha: &str,
    ) -> Result<Vec<CiCheckRun>, ApplicationError> {
        let url = format!("{}/repos/{}/{}/commits/{}/check-runs", self.api_base_url, owner, repository, commit_sha);
        let headers = self.build_headers(token)?;

        let response = self.client.get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitHub network error: {}", error)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!("Failed to fetch check runs: {}", error_text)));
        }

        #[derive(Deserialize)]
        struct CheckRunsPayload {
            check_runs: Vec<GitHubCheckRunItem>,
        }

        #[derive(Deserialize)]
        struct GitHubCheckRunItem {
            id: u64,
            name: String,
            status: String,
            conclusion: Option<String>,
            html_url: Option<String>,
            started_at: Option<String>,
            completed_at: Option<String>,
        }

        let payload: CheckRunsPayload = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse check runs: {}", error))
        })?;

        let checks = payload.check_runs.into_iter().map(|item| {
            let started_timestamp = item.started_at
                .as_deref()
                .and_then(|iso| chrono::DateTime::parse_from_rfc3339(iso).ok())
                .map(|datetime| datetime.timestamp());
            let completed_timestamp = item.completed_at
                .as_deref()
                .and_then(|iso| chrono::DateTime::parse_from_rfc3339(iso).ok())
                .map(|datetime| datetime.timestamp());

            CiCheckRun {
                id: format!("github:{}", item.id),
                name: item.name,
                status: item.status,
                conclusion: item.conclusion,
                html_url: item.html_url,
                started_at_timestamp: started_timestamp,
                completed_at_timestamp: completed_timestamp,
            }
        }).collect();

        Ok(checks)
    }

    pub async fn get_pull_request_comments(
        &self,
        token: &str,
        owner: &str,
        repository: &str,
        pull_number: u64,
    ) -> Result<Vec<RemoteReviewComment>, ApplicationError> {
        let url = format!("{}/repos/{}/{}/issues/{}/comments", self.api_base_url, owner, repository, pull_number);
        let headers = self.build_headers(token)?;

        let response = self.client.get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitHub network error: {}", error)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!("Failed to fetch PR comments: {}", error_text)));
        }

        #[derive(Deserialize)]
        struct GitHubCommentItem {
            id: u64,
            body: Option<String>,
            user: GitHubPullRequestUser,
            created_at: String,
            updated_at: String,
        }

        let items: Vec<GitHubCommentItem> = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse comments: {}", error))
        })?;

        let comments = items.into_iter().map(|item| {
            let created_timestamp = chrono::DateTime::parse_from_rfc3339(&item.created_at)
                .map(|datetime| datetime.timestamp())
                .unwrap_or_else(|_| Utc::now().timestamp());
            let updated_timestamp = chrono::DateTime::parse_from_rfc3339(&item.updated_at)
                .map(|datetime| datetime.timestamp())
                .unwrap_or_else(|_| Utc::now().timestamp());

            RemoteReviewComment {
                id: format!("github:{}", item.id),
                author_username: item.user.login,
                author_avatar_url: item.user.avatar_url,
                body: item.body.unwrap_or_default(),
                path: None,
                line_number: None,
                created_at_timestamp: created_timestamp,
                updated_at_timestamp: updated_timestamp,
            }
        }).collect();

        Ok(comments)
    }

    pub async fn create_pull_request_comment(
        &self,
        token: &str,
        owner: &str,
        repository: &str,
        pull_number: u64,
        body: &str,
    ) -> Result<RemoteReviewComment, ApplicationError> {
        let url = format!("{}/repos/{}/{}/issues/{}/comments", self.api_base_url, owner, repository, pull_number);
        let headers = self.build_headers(token)?;

        #[derive(Serialize)]
        struct NewCommentPayload<'a> {
            body: &'a str,
        }

        let response = self.client.post(&url)
            .headers(headers)
            .json(&NewCommentPayload { body })
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitHub network error: {}", error)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!("Failed to post comment: {}", error_text)));
        }

        #[derive(Deserialize)]
        struct GitHubCommentItem {
            id: u64,
            body: Option<String>,
            user: GitHubPullRequestUser,
            created_at: String,
            updated_at: String,
        }

        let item: GitHubCommentItem = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse created comment: {}", error))
        })?;

        let created_timestamp = chrono::DateTime::parse_from_rfc3339(&item.created_at)
            .map(|datetime| datetime.timestamp())
            .unwrap_or_else(|_| Utc::now().timestamp());
        let updated_timestamp = chrono::DateTime::parse_from_rfc3339(&item.updated_at)
            .map(|datetime| datetime.timestamp())
            .unwrap_or_else(|_| Utc::now().timestamp());

        Ok(RemoteReviewComment {
            id: format!("github:{}", item.id),
            author_username: item.user.login,
            author_avatar_url: item.user.avatar_url,
            body: item.body.unwrap_or_default(),
            path: None,
            line_number: None,
            created_at_timestamp: created_timestamp,
            updated_at_timestamp: updated_timestamp,
        })
    }

    pub async fn get_pull_request_files(
        &self,
        token: &str,
        owner: &str,
        repository: &str,
        pull_number: u64,
    ) -> Result<Vec<PullRequestChangedFile>, ApplicationError> {
        let url = format!("{}/repos/{}/{}/pulls/{}/files", self.api_base_url, owner, repository, pull_number);
        let headers = self.build_headers(token)?;

        let response = self.client.get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitHub network error: {}", error)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!("Failed to fetch PR files: {}", error_text)));
        }

        #[derive(Deserialize)]
        struct GitHubFileItem {
            filename: String,
            previous_filename: Option<String>,
            status: String,
            additions: u32,
            deletions: u32,
            changes: u32,
            patch: Option<String>,
        }

        let items: Vec<GitHubFileItem> = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse PR files: {}", error))
        })?;

        let files = items.into_iter().map(|item| {
            PullRequestChangedFile {
                file_path: item.filename,
                previous_file_path: item.previous_filename,
                status: item.status,
                additions_count: item.additions,
                deletions_count: item.deletions,
                changes_count: item.changes,
                patch: item.patch,
            }
        }).collect();

        Ok(files)
    }

    pub async fn submit_pull_request_review(
        &self,
        token: &str,
        owner: &str,
        repository: &str,
        pull_number: u64,
        action: &str,
        body: Option<&str>,
    ) -> Result<bool, ApplicationError> {
        let url = format!("{}/repos/{}/{}/pulls/{}/reviews", self.api_base_url, owner, repository, pull_number);
        let headers = self.build_headers(token)?;

        #[derive(Serialize)]
        struct ReviewPayload<'a> {
            event: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            body: Option<&'a str>,
        }

        let event = match action.to_uppercase().as_str() {
            "APPROVE" => "APPROVE",
            "REQUEST_CHANGES" => "REQUEST_CHANGES",
            _ => "COMMENT",
        };

        let response = self.client.post(&url)
            .headers(headers)
            .json(&ReviewPayload {
                event,
                body: body.filter(|text| !text.trim().is_empty()),
            })
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitHub network error: {}", error)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!("Failed to submit review: {}", error_text)));
        }

        Ok(true)
    }

    pub async fn merge_pull_request(
        &self,
        token: &str,
        owner: &str,
        repository: &str,
        pull_number: u64,
        merge_method: Option<&str>,
        commit_title: Option<&str>,
        commit_message: Option<&str>,
    ) -> Result<PullRequestMergeResult, ApplicationError> {
        let url = format!("{}/repos/{}/{}/pulls/{}/merge", self.api_base_url, owner, repository, pull_number);
        let headers = self.build_headers(token)?;

        #[derive(Serialize)]
        struct MergePayload<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            commit_title: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            commit_message: Option<&'a str>,
            merge_method: &'a str,
        }

        let normalized_method = match merge_method.unwrap_or("merge") {
            "squash" => "squash",
            "rebase" => "rebase",
            _ => "merge",
        };

        let response = self.client.put(&url)
            .headers(headers)
            .json(&MergePayload {
                commit_title: commit_title.filter(|text| !text.trim().is_empty()),
                commit_message: commit_message.filter(|text| !text.trim().is_empty()),
                merge_method: normalized_method,
            })
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitHub network error: {}", error)))?;

        #[derive(Deserialize)]
        struct GitHubMergeApiResponse {
            sha: Option<String>,
            merged: Option<bool>,
            message: Option<String>,
        }

        let status = response.status();
        let payload: GitHubMergeApiResponse = response.json().await.unwrap_or(GitHubMergeApiResponse {
            sha: None,
            merged: Some(status.is_success()),
            message: None,
        });

        if !status.is_success() && payload.merged != Some(true) {
            return Err(ApplicationError::ProviderError(
                payload.message.unwrap_or_else(|| "Failed to merge pull request".into())
            ));
        }

        Ok(PullRequestMergeResult {
            is_merged: payload.merged.unwrap_or(true),
            message: payload.message.unwrap_or_else(|| "Pull request merged successfully".into()),
            commit_sha: payload.sha,
        })
    }

    pub async fn update_pull_request_state(
        &self,
        token: &str,
        owner: &str,
        repository: &str,
        pull_number: u64,
        new_state: &str,
    ) -> Result<bool, ApplicationError> {
        let url = format!("{}/repos/{}/{}/pulls/{}", self.api_base_url, owner, repository, pull_number);
        let headers = self.build_headers(token)?;

        #[derive(Serialize)]
        struct StatePayload<'a> {
            state: &'a str,
        }

        let target_state = if new_state == "open" { "open" } else { "closed" };

        let response = self.client.patch(&url)
            .headers(headers)
            .json(&StatePayload { state: target_state })
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitHub network error: {}", error)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!("Failed to update PR state: {}", error_text)));
        }

        Ok(true)
    }

    /// Fetches workflow runs for a repository.
    pub async fn list_workflow_runs(
        &self,
        token: &str,
        owner: &str,
        repository: &str,
        branch: Option<&str>,
    ) -> Result<Vec<WorkflowRun>, ApplicationError> {
        let mut url = format!("{}/repos/{}/{}/actions/runs?per_page=30", self.api_base_url, owner, repository);
        if let Some(branch_name) = branch {
            if !branch_name.trim().is_empty() {
                url.push_str(&format!("&branch={}", branch_name.trim()));
            }
        }
        let headers = self.build_headers(token)?;

        let response = self.client.get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitHub network error: {}", error)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!("Failed to fetch workflow runs: {}", error_text)));
        }

        #[derive(Deserialize)]
        struct WorkflowRunsPayload {
            workflow_runs: Vec<GitHubWorkflowRunItem>,
        }

        #[derive(Deserialize)]
        struct GitHubActor {
            login: String,
            avatar_url: Option<String>,
        }

        #[derive(Deserialize)]
        struct GitHubWorkflowRunItem {
            id: u64,
            name: Option<String>,
            display_title: Option<String>,
            head_branch: Option<String>,
            head_sha: Option<String>,
            status: Option<String>,
            conclusion: Option<String>,
            event: Option<String>,
            run_number: Option<u32>,
            actor: Option<GitHubActor>,
            created_at: Option<String>,
            updated_at: Option<String>,
            html_url: Option<String>,
        }

        let payload: WorkflowRunsPayload = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse workflow runs: {}", error))
        })?;

        let runs = payload.workflow_runs.into_iter().map(|item| {
            let created_timestamp = item.created_at
                .as_deref()
                .and_then(|iso| chrono::DateTime::parse_from_rfc3339(iso).ok())
                .map(|datetime| datetime.timestamp())
                .unwrap_or(0);
            let updated_timestamp = item.updated_at
                .as_deref()
                .and_then(|iso| chrono::DateTime::parse_from_rfc3339(iso).ok())
                .map(|datetime| datetime.timestamp())
                .unwrap_or(0);

            WorkflowRun {
                id: item.id.to_string(),
                workflow_name: item.name.unwrap_or_else(|| "Workflow".into()),
                display_title: item.display_title.unwrap_or_else(|| "Workflow Run".into()),
                head_branch: item.head_branch.unwrap_or_default(),
                head_commit_sha: item.head_sha.unwrap_or_default(),
                status: item.status.unwrap_or_else(|| "completed".into()),
                conclusion: item.conclusion,
                event: item.event.unwrap_or_else(|| "push".into()),
                run_number: item.run_number.unwrap_or(1),
                author_username: item.actor.as_ref().map(|a| a.login.clone()).unwrap_or_default(),
                author_avatar_url: item.actor.and_then(|a| a.avatar_url),
                created_at_timestamp: created_timestamp,
                updated_at_timestamp: updated_timestamp,
                html_url: item.html_url,
            }
        }).collect();

        Ok(runs)
    }

    /// Fetches all jobs for a specific workflow run.
    pub async fn get_workflow_run_jobs(
        &self,
        token: &str,
        owner: &str,
        repository: &str,
        run_id: u64,
    ) -> Result<Vec<WorkflowJob>, ApplicationError> {
        let url = format!("{}/repos/{}/{}/actions/runs/{}/jobs", self.api_base_url, owner, repository, run_id);
        let headers = self.build_headers(token)?;

        let response = self.client.get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitHub network error: {}", error)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!("Failed to fetch workflow jobs: {}", error_text)));
        }

        #[derive(Deserialize)]
        struct JobsPayload {
            jobs: Vec<GitHubJobItem>,
        }

        #[derive(Deserialize)]
        struct GitHubStepItem {
            name: String,
            status: String,
            conclusion: Option<String>,
            number: u32,
            started_at: Option<String>,
            completed_at: Option<String>,
        }

        #[derive(Deserialize)]
        struct GitHubJobItem {
            id: u64,
            name: String,
            status: String,
            conclusion: Option<String>,
            started_at: Option<String>,
            completed_at: Option<String>,
            steps: Option<Vec<GitHubStepItem>>,
            html_url: Option<String>,
        }

        let payload: JobsPayload = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse workflow jobs: {}", error))
        })?;

        let jobs = payload.jobs.into_iter().map(|item| {
            let started_timestamp = item.started_at
                .as_deref()
                .and_then(|iso| chrono::DateTime::parse_from_rfc3339(iso).ok())
                .map(|datetime| datetime.timestamp());
            let completed_timestamp = item.completed_at
                .as_deref()
                .and_then(|iso| chrono::DateTime::parse_from_rfc3339(iso).ok())
                .map(|datetime| datetime.timestamp());

            let steps = item.steps.unwrap_or_default().into_iter().map(|s| {
                let s_started = s.started_at
                    .as_deref()
                    .and_then(|iso| chrono::DateTime::parse_from_rfc3339(iso).ok())
                    .map(|dt| dt.timestamp());
                let s_completed = s.completed_at
                    .as_deref()
                    .and_then(|iso| chrono::DateTime::parse_from_rfc3339(iso).ok())
                    .map(|dt| dt.timestamp());

                WorkflowJobStep {
                    name: s.name,
                    status: s.status,
                    conclusion: s.conclusion,
                    number: s.number,
                    started_at_timestamp: s_started,
                    completed_at_timestamp: s_completed,
                }
            }).collect();

            WorkflowJob {
                id: item.id.to_string(),
                run_id: run_id.to_string(),
                name: item.name,
                status: item.status,
                conclusion: item.conclusion,
                started_at_timestamp: started_timestamp,
                completed_at_timestamp: completed_timestamp,
                steps,
                html_url: item.html_url,
            }
        }).collect();

        Ok(jobs)
    }

    /// Triggers a rerun of a workflow run.
    pub async fn rerun_workflow_run(
        &self,
        token: &str,
        owner: &str,
        repository: &str,
        run_id: u64,
        failed_only: bool,
    ) -> Result<bool, ApplicationError> {
        let endpoint = if failed_only { "rerun-failed-jobs" } else { "rerun" };
        let url = format!("{}/repos/{}/{}/actions/runs/{}/{}", self.api_base_url, owner, repository, run_id, endpoint);
        let headers = self.build_headers(token)?;

        let response = self.client.post(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitHub network error: {}", error)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!("Failed to rerun workflow: {}", error_text)));
        }

        Ok(true)
    }

    /// Cancels an in-progress workflow run.
    pub async fn cancel_workflow_run(
        &self,
        token: &str,
        owner: &str,
        repository: &str,
        run_id: u64,
    ) -> Result<bool, ApplicationError> {
        let url = format!("{}/repos/{}/{}/actions/runs/{}/cancel", self.api_base_url, owner, repository, run_id);
        let headers = self.build_headers(token)?;

        let response = self.client.post(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitHub network error: {}", error)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!("Failed to cancel workflow run: {}", error_text)));
        }

        Ok(true)
    }

    /// Fetches execution logs for a workflow job.
    pub async fn get_job_logs(
        &self,
        token: &str,
        owner: &str,
        repository: &str,
        job_id: u64,
    ) -> Result<String, ApplicationError> {
        let url = format!("{}/repos/{}/{}/actions/jobs/{}/logs", self.api_base_url, owner, repository, job_id);
        let headers = self.build_headers(token)?;

        let response = self.client.get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitHub network error: {}", error)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!("Failed to fetch job logs: {}", error_text)));
        }

        let logs = response.text().await.unwrap_or_default();
        Ok(logs)
    }

    /// Creates a new pull request on GitHub.
    pub async fn create_pull_request(
        &self,
        token: &str,
        owner: &str,
        repository: &str,
        title: &str,
        description: Option<&str>,
        source_branch: &str,
        target_branch: &str,
        is_draft: bool,
    ) -> Result<RemotePullRequest, ApplicationError> {
        let url = format!("{}/repos/{}/{}/pulls", self.api_base_url, owner, repository);
        let headers = self.build_headers(token)?;

        #[derive(Serialize)]
        struct CreatePullPayload<'a> {
            title: &'a str,
            head: &'a str,
            base: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            body: Option<&'a str>,
            draft: bool,
        }

        let response = self.client.post(&url)
            .headers(headers)
            .json(&CreatePullPayload {
                title,
                head: source_branch,
                base: target_branch,
                body: description,
                draft: is_draft,
            })
            .send()
            .await
            .map_err(|error| ApplicationError::ProviderError(format!("GitHub network error: {}", error)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!("Failed to create pull request: {}", error_text)));
        }

        let raw: GitHubPullRequestResponse = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse created pull request: {}", error))
        })?;

        let created_timestamp = chrono::DateTime::parse_from_rfc3339(&raw.created_at)
            .map(|datetime| datetime.timestamp())
            .unwrap_or_else(|_| Utc::now().timestamp());

        let updated_timestamp = chrono::DateTime::parse_from_rfc3339(&raw.updated_at)
            .map(|datetime| datetime.timestamp())
            .unwrap_or_else(|_| Utc::now().timestamp());

        Ok(RemotePullRequest {
            id: format!("github:{}", raw.id),
            number: raw.number,
            title: raw.title,
            description: raw.body,
            state: PullRequestState::Open,
            author_username: raw.user.login,
            author_avatar_url: raw.user.avatar_url,
            source_branch: raw.head.ref_name,
            target_branch: raw.base.ref_name,
            is_draft: raw.draft.unwrap_or(false),
            comments_count: 0,
            html_url: raw.html_url,
            created_at_timestamp: created_timestamp,
            updated_at_timestamp: updated_timestamp,
            merged_at_timestamp: None,
            pipeline_status: None,
        })
    }
}
