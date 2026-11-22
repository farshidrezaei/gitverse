use crate::domain::{
    ApplicationError, CiCheckRun, PipelineState, PipelineSummary, ProviderAccount, ProviderKind,
    PullRequestChangedFile, PullRequestMergeResult, PullRequestState, RemoteProjectRepository,
    RemotePullRequest, RemoteReviewComment, WorkflowJob, WorkflowRun,
};
use chrono::Utc;
use reqwest::Client;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};

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

        let client = Client::builder().build().map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to build HTTP client: {}", error))
        })?;

        Ok(Self {
            client,
            api_base_url,
        })
    }

    fn build_headers(&self, token: &str) -> Result<HeaderMap, ApplicationError> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(GITLAB_USER_AGENT));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let mut token_header = HeaderValue::from_str(token.trim()).map_err(|_| {
            ApplicationError::AuthenticationError(
                "Invalid GitLab personal access token format".into(),
            )
        })?;
        token_header.set_sensitive(true);
        headers.insert("PRIVATE-TOKEN", token_header);

        Ok(headers)
    }

    /// Fetches authenticated GitLab user profile to validate credentials.
    pub async fn get_authenticated_user(
        &self,
        token: &str,
        instance_url: &str,
    ) -> Result<ProviderAccount, ApplicationError> {
        let url = format!("{}/user", self.api_base_url);
        let headers = self.build_headers(token)?;

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| {
                ApplicationError::ProviderError(format!("GitLab network error: {}", error))
            })?;

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
            ApplicationError::ProviderError(format!(
                "Failed to parse GitLab user response: {}",
                error
            ))
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

        let response = self
            .client
            .get(&url)
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
            .map_err(|error| {
                ApplicationError::ProviderError(format!("GitLab network error: {}", error))
            })?;

        if !response.status().is_success() {
            let error_message = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!(
                "GitLab list projects failed: {}",
                error_message
            )));
        }

        let raw_projects: Vec<GitLabProjectResponse> = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse GitLab projects: {}", error))
        })?;

        let projects = raw_projects
            .into_iter()
            .map(|raw| {
                let updated_timestamp = raw
                    .last_activity_at
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
        let url = format!(
            "{}/projects/{}/merge_requests",
            self.api_base_url, encoded_project
        );
        let headers = self.build_headers(token)?;

        let state = match state_filter {
            Some("open") => "opened",
            Some("closed") => "closed",
            Some("merged") => "merged",
            _ => "all",
        };

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .query(&[("state", state), ("per_page", "50")])
            .send()
            .await
            .map_err(|error| {
                ApplicationError::ProviderError(format!("GitLab network error: {}", error))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!(
                "Failed to fetch merge requests: {}",
                error_text
            )));
        }

        let raw_mrs: Vec<GitLabMergeRequestResponse> = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse merge requests: {}", error))
        })?;

        let mut merge_requests: Vec<RemotePullRequest> = raw_mrs
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

                let merged_timestamp = raw
                    .merged_at
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

        // Populate pipeline status for the first few open merge requests
        for merge_request in merge_requests.iter_mut().take(5) {
            if let Ok(summary) = self
                .get_commit_pipeline_summary(
                    token,
                    project_id_or_path,
                    &merge_request.source_branch,
                )
                .await
            {
                merge_request.pipeline_status = summary;
            }
        }

        Ok(merge_requests)
    }

    /// Fetches commit pipeline summary for a GitLab commit or branch.
    pub async fn get_commit_pipeline_summary(
        &self,
        token: &str,
        project_id_or_path: &str,
        reference_or_sha: &str,
    ) -> Result<Option<PipelineSummary>, ApplicationError> {
        let encoded_project = urlencoding_simple(project_id_or_path);
        let url = format!(
            "{}/projects/{}/pipelines",
            self.api_base_url, encoded_project
        );
        let headers = self.build_headers(token)?;

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .query(&[("ref", reference_or_sha), ("per_page", "1")])
            .send()
            .await
            .map_err(|error| {
                ApplicationError::ProviderError(format!("GitLab network error: {}", error))
            })?;

        if !response.status().is_success() {
            return Ok(None);
        }

        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct GitLabPipelineItem {
            id: u64,
            status: String,
            web_url: String,
            sha: String,
        }

        let pipelines: Vec<GitLabPipelineItem> = match response.json().await {
            Ok(parsed) => parsed,
            Err(_) => return Ok(None),
        };

        let latest = match pipelines.into_iter().next() {
            Some(pipeline) => pipeline,
            None => return Ok(None),
        };

        let pipeline_state = match latest.status.as_str() {
            "success" => PipelineState::Success,
            "failed" => PipelineState::Failed,
            "running" => PipelineState::Running,
            "canceled" => PipelineState::Canceled,
            "skipped" => PipelineState::Skipped,
            _ => PipelineState::Pending,
        };

        Ok(Some(PipelineSummary {
            state: pipeline_state,
            web_url: Some(latest.web_url),
            commit_sha: Some(latest.sha),
            total_checks_count: 1,
            successful_checks_count: if pipeline_state == PipelineState::Success {
                1
            } else {
                0
            },
            failed_checks_count: if pipeline_state == PipelineState::Failed {
                1
            } else {
                0
            },
        }))
    }

    /// Fetches detailed jobs/checks for a pipeline ID in GitLab.
    pub async fn list_pipeline_jobs(
        &self,
        token: &str,
        project_id_or_path: &str,
        pipeline_id: u64,
    ) -> Result<Vec<CiCheckRun>, ApplicationError> {
        let encoded_project = urlencoding_simple(project_id_or_path);
        let url = format!(
            "{}/projects/{}/pipelines/{}/jobs",
            self.api_base_url, encoded_project, pipeline_id
        );
        let headers = self.build_headers(token)?;

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| {
                ApplicationError::ProviderError(format!("GitLab network error: {}", error))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!(
                "Failed to fetch pipeline jobs: {}",
                error_text
            )));
        }

        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct GitLabJobItem {
            id: u64,
            name: String,
            status: String,
            web_url: String,
            started_at: Option<String>,
            finished_at: Option<String>,
        }

        let jobs: Vec<GitLabJobItem> = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse pipeline jobs: {}", error))
        })?;

        let checks = jobs
            .into_iter()
            .map(|item| {
                let started_timestamp = item
                    .started_at
                    .as_deref()
                    .and_then(|iso| chrono::DateTime::parse_from_rfc3339(iso).ok())
                    .map(|datetime| datetime.timestamp());
                let completed_timestamp = item
                    .finished_at
                    .as_deref()
                    .and_then(|iso| chrono::DateTime::parse_from_rfc3339(iso).ok())
                    .map(|datetime| datetime.timestamp());

                let conclusion = match item.status.as_str() {
                    "success" => Some("success".to_string()),
                    "failed" => Some("failure".to_string()),
                    "canceled" => Some("cancelled".to_string()),
                    _ => None,
                };

                CiCheckRun {
                    id: format!("gitlab:{}", item.id),
                    name: item.name,
                    status: item.status,
                    conclusion,
                    html_url: Some(item.web_url),
                    started_at_timestamp: started_timestamp,
                    completed_at_timestamp: completed_timestamp,
                }
            })
            .collect();

        Ok(checks)
    }

    pub async fn get_merge_request_comments(
        &self,
        token: &str,
        project_full_name: &str,
        mr_iid: u64,
    ) -> Result<Vec<RemoteReviewComment>, ApplicationError> {
        let encoded_project = urlencoding_simple(project_full_name);
        let url = format!(
            "{}/projects/{}/merge_requests/{}/notes",
            self.api_base_url, encoded_project, mr_iid
        );
        let headers = self.build_headers(token)?;

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| {
                ApplicationError::ProviderError(format!("GitLab network error: {}", error))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!(
                "Failed to fetch MR comments: {}",
                error_text
            )));
        }

        #[derive(Deserialize)]
        struct GitLabNoteItem {
            id: u64,
            body: String,
            system: bool,
            author: GitLabMergeRequestAuthor,
            created_at: String,
            updated_at: String,
        }

        let items: Vec<GitLabNoteItem> = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse MR comments: {}", error))
        })?;

        let comments = items
            .into_iter()
            .filter(|item| !item.system)
            .map(|item| {
                let created_timestamp = chrono::DateTime::parse_from_rfc3339(&item.created_at)
                    .map(|datetime| datetime.timestamp())
                    .unwrap_or_else(|_| Utc::now().timestamp());
                let updated_timestamp = chrono::DateTime::parse_from_rfc3339(&item.updated_at)
                    .map(|datetime| datetime.timestamp())
                    .unwrap_or_else(|_| Utc::now().timestamp());

                RemoteReviewComment {
                    id: format!("gitlab:{}", item.id),
                    author_username: item.author.username,
                    author_avatar_url: item.author.avatar_url,
                    body: item.body,
                    path: None,
                    line_number: None,
                    created_at_timestamp: created_timestamp,
                    updated_at_timestamp: updated_timestamp,
                }
            })
            .collect();

        Ok(comments)
    }

    pub async fn create_merge_request_comment(
        &self,
        token: &str,
        project_full_name: &str,
        mr_iid: u64,
        body: &str,
    ) -> Result<RemoteReviewComment, ApplicationError> {
        let encoded_project = urlencoding_simple(project_full_name);
        let url = format!(
            "{}/projects/{}/merge_requests/{}/notes",
            self.api_base_url, encoded_project, mr_iid
        );
        let headers = self.build_headers(token)?;

        #[derive(Serialize)]
        struct NewNotePayload<'a> {
            body: &'a str,
        }

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&NewNotePayload { body })
            .send()
            .await
            .map_err(|error| {
                ApplicationError::ProviderError(format!("GitLab network error: {}", error))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!(
                "Failed to post MR comment: {}",
                error_text
            )));
        }

        #[derive(Deserialize)]
        struct GitLabNoteItem {
            id: u64,
            body: String,
            author: GitLabMergeRequestAuthor,
            created_at: String,
            updated_at: String,
        }

        let item: GitLabNoteItem = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse created comment: {}", error))
        })?;

        let created_timestamp = chrono::DateTime::parse_from_rfc3339(&item.created_at)
            .map(|datetime| datetime.timestamp())
            .unwrap_or_else(|_| Utc::now().timestamp());
        let updated_timestamp = chrono::DateTime::parse_from_rfc3339(&item.updated_at)
            .map(|datetime| datetime.timestamp())
            .unwrap_or_else(|_| Utc::now().timestamp());

        Ok(RemoteReviewComment {
            id: format!("gitlab:{}", item.id),
            author_username: item.author.username,
            author_avatar_url: item.author.avatar_url,
            body: item.body,
            path: None,
            line_number: None,
            created_at_timestamp: created_timestamp,
            updated_at_timestamp: updated_timestamp,
        })
    }

    pub async fn get_merge_request_files(
        &self,
        token: &str,
        project_full_name: &str,
        mr_iid: u64,
    ) -> Result<Vec<PullRequestChangedFile>, ApplicationError> {
        let encoded_project = urlencoding_simple(project_full_name);
        let url = format!(
            "{}/projects/{}/merge_requests/{}/diffs",
            self.api_base_url, encoded_project, mr_iid
        );
        let headers = self.build_headers(token)?;

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| {
                ApplicationError::ProviderError(format!("GitLab network error: {}", error))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!(
                "Failed to fetch MR diffs: {}",
                error_text
            )));
        }

        #[derive(Deserialize)]
        struct GitLabDiffItem {
            old_path: String,
            new_path: String,
            new_file: bool,
            renamed_file: bool,
            deleted_file: bool,
            diff: String,
        }

        let items: Vec<GitLabDiffItem> = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse MR diffs: {}", error))
        })?;

        let files = items
            .into_iter()
            .map(|item| {
                let additions = item
                    .diff
                    .lines()
                    .filter(|line| line.starts_with('+') && !line.starts_with("+++"))
                    .count() as u32;
                let deletions = item
                    .diff
                    .lines()
                    .filter(|line| line.starts_with('-') && !line.starts_with("---"))
                    .count() as u32;
                let status = if item.new_file {
                    "added".to_string()
                } else if item.deleted_file {
                    "deleted".to_string()
                } else if item.renamed_file {
                    "renamed".to_string()
                } else {
                    "modified".to_string()
                };

                PullRequestChangedFile {
                    file_path: item.new_path,
                    previous_file_path: if item.renamed_file {
                        Some(item.old_path)
                    } else {
                        None
                    },
                    status,
                    additions_count: additions,
                    deletions_count: deletions,
                    changes_count: additions + deletions,
                    patch: Some(item.diff),
                }
            })
            .collect();

        Ok(files)
    }

    pub async fn submit_merge_request_review(
        &self,
        token: &str,
        project_full_name: &str,
        mr_iid: u64,
        action: &str,
        body: Option<&str>,
    ) -> Result<bool, ApplicationError> {
        let encoded_project = urlencoding_simple(project_full_name);
        let headers = self.build_headers(token)?;

        match action.to_uppercase().as_str() {
            "APPROVE" => {
                let url = format!(
                    "{}/projects/{}/merge_requests/{}/approve",
                    self.api_base_url, encoded_project, mr_iid
                );
                let response = self
                    .client
                    .post(&url)
                    .headers(headers.clone())
                    .send()
                    .await
                    .map_err(|error| {
                        ApplicationError::ProviderError(format!("GitLab network error: {}", error))
                    })?;

                if !response.status().is_success() {
                    let error_text = response.text().await.unwrap_or_default();
                    return Err(ApplicationError::ProviderError(format!(
                        "Failed to approve MR: {}",
                        error_text
                    )));
                }
            }
            "REQUEST_CHANGES" => {
                let url = format!(
                    "{}/projects/{}/merge_requests/{}/unapprove",
                    self.api_base_url, encoded_project, mr_iid
                );
                let _ = self.client.post(&url).headers(headers.clone()).send().await;
            }
            _ => {}
        }

        if let Some(comment_text) = body.filter(|text| !text.trim().is_empty()) {
            self.create_merge_request_comment(token, project_full_name, mr_iid, comment_text)
                .await?;
        }

        Ok(true)
    }

    pub async fn merge_merge_request(
        &self,
        token: &str,
        project_full_name: &str,
        mr_iid: u64,
        commit_message: Option<&str>,
        squash: Option<bool>,
        delete_source_branch: Option<bool>,
    ) -> Result<PullRequestMergeResult, ApplicationError> {
        let encoded_project = urlencoding_simple(project_full_name);
        let url = format!(
            "{}/projects/{}/merge_requests/{}/merge",
            self.api_base_url, encoded_project, mr_iid
        );
        let headers = self.build_headers(token)?;

        #[derive(Serialize)]
        struct MergePayload<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            merge_commit_message: Option<&'a str>,
            squash: bool,
            should_remove_source_branch: bool,
        }

        let response = self
            .client
            .put(&url)
            .headers(headers)
            .json(&MergePayload {
                merge_commit_message: commit_message.filter(|text| !text.trim().is_empty()),
                squash: squash.unwrap_or(false),
                should_remove_source_branch: delete_source_branch.unwrap_or(false),
            })
            .send()
            .await
            .map_err(|error| {
                ApplicationError::ProviderError(format!("GitLab network error: {}", error))
            })?;

        #[derive(Deserialize)]
        struct GitLabMergeApiResponse {
            state: Option<String>,
            merge_commit_sha: Option<String>,
            message: Option<String>,
        }

        let status = response.status();
        let payload: GitLabMergeApiResponse =
            response.json().await.unwrap_or(GitLabMergeApiResponse {
                state: None,
                merge_commit_sha: None,
                message: None,
            });

        if !status.is_success() && payload.state.as_deref() != Some("merged") {
            return Err(ApplicationError::ProviderError(
                payload
                    .message
                    .unwrap_or_else(|| "Failed to merge merge request".into()),
            ));
        }

        Ok(PullRequestMergeResult {
            is_merged: true,
            message: payload
                .message
                .unwrap_or_else(|| "Merge request merged successfully".into()),
            commit_sha: payload.merge_commit_sha,
        })
    }

    pub async fn update_merge_request_state(
        &self,
        token: &str,
        project_full_name: &str,
        mr_iid: u64,
        new_state: &str,
    ) -> Result<bool, ApplicationError> {
        let encoded_project = urlencoding_simple(project_full_name);
        let url = format!(
            "{}/projects/{}/merge_requests/{}",
            self.api_base_url, encoded_project, mr_iid
        );
        let headers = self.build_headers(token)?;

        #[derive(Serialize)]
        struct StatePayload<'a> {
            state_event: &'a str,
        }

        let state_event = if new_state == "open" {
            "reopen"
        } else {
            "close"
        };

        let response = self
            .client
            .put(&url)
            .headers(headers)
            .json(&StatePayload { state_event })
            .send()
            .await
            .map_err(|error| {
                ApplicationError::ProviderError(format!("GitLab network error: {}", error))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!(
                "Failed to update MR state: {}",
                error_text
            )));
        }

        Ok(true)
    }

    pub async fn list_pipelines(
        &self,
        token: &str,
        project_full_name: &str,
        status_filter: Option<&str>,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<WorkflowRun>, ApplicationError> {
        let encoded_project = urlencoding_simple(project_full_name);
        let mut url = format!(
            "{}/projects/{}/pipelines?page={}&per_page={}",
            self.api_base_url,
            encoded_project,
            page.max(1),
            per_page.clamp(1, 100)
        );
        if let Some(status) = status_filter
            && !status.is_empty() && status != "all" {
                url.push_str(&format!("&status={}", status));
            }

        let headers = self.build_headers(token)?;
        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| {
                ApplicationError::ProviderError(format!("GitLab network error: {}", error))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!(
                "Failed to list GitLab pipelines: {}",
                error_text
            )));
        }

        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct GitLabPipelineItem {
            id: u64,
            iid: Option<u64>,
            project_id: Option<u64>,
            sha: String,
            #[serde(rename = "ref")]
            ref_name: Option<String>,
            status: String,
            source: Option<String>,
            created_at: Option<String>,
            updated_at: Option<String>,
            web_url: String,
            name: Option<String>,
        }

        let pipelines: Vec<GitLabPipelineItem> = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse GitLab pipelines: {}", error))
        })?;

        let runs = pipelines
            .into_iter()
            .map(|pipeline| {
                let (status, conclusion) = match pipeline.status.as_str() {
                    "success" => ("completed".to_string(), Some("success".to_string())),
                    "failed" => ("completed".to_string(), Some("failure".to_string())),
                    "running" => ("in_progress".to_string(), None),
                    "pending" | "waiting_for_resource" | "preparing" => {
                        ("queued".to_string(), None)
                    }
                    "canceled" => ("completed".to_string(), Some("cancelled".to_string())),
                    "skipped" => ("completed".to_string(), Some("skipped".to_string())),
                    other => (other.to_string(), None),
                };

                let name = pipeline
                    .name
                    .unwrap_or_else(|| format!("Pipeline #{}", pipeline.id));

                let created_timestamp = pipeline
                    .created_at
                    .as_deref()
                    .and_then(|iso| chrono::DateTime::parse_from_rfc3339(iso).ok())
                    .map(|datetime| datetime.timestamp())
                    .unwrap_or(0);
                let updated_timestamp = pipeline
                    .updated_at
                    .as_deref()
                    .and_then(|iso| chrono::DateTime::parse_from_rfc3339(iso).ok())
                    .map(|datetime| datetime.timestamp())
                    .unwrap_or(0);

                WorkflowRun {
                    id: pipeline.id.to_string(),
                    workflow_name: name,
                    display_title: format!(
                        "Pipeline #{} on {}",
                        pipeline.id,
                        pipeline.ref_name.as_deref().unwrap_or("unknown")
                    ),
                    head_branch: pipeline.ref_name.unwrap_or_default(),
                    head_commit_sha: pipeline.sha,
                    status,
                    conclusion,
                    event: pipeline.source.unwrap_or_else(|| "push".to_string()),
                    run_number: pipeline.iid.unwrap_or(pipeline.id) as u32,
                    author_username: String::new(),
                    author_avatar_url: None,
                    created_at_timestamp: created_timestamp,
                    updated_at_timestamp: updated_timestamp,
                    html_url: Some(pipeline.web_url),
                }
            })
            .collect();

        Ok(runs)
    }

    pub async fn get_pipeline_jobs(
        &self,
        token: &str,
        project_full_name: &str,
        pipeline_id: u64,
    ) -> Result<Vec<WorkflowJob>, ApplicationError> {
        let encoded_project = urlencoding_simple(project_full_name);
        let url = format!(
            "{}/projects/{}/pipelines/{}/jobs?per_page=100",
            self.api_base_url, encoded_project, pipeline_id
        );

        let headers = self.build_headers(token)?;
        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| {
                ApplicationError::ProviderError(format!("GitLab network error: {}", error))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!(
                "Failed to get GitLab pipeline jobs: {}",
                error_text
            )));
        }

        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct GitLabJobItem {
            id: u64,
            name: String,
            stage: Option<String>,
            status: String,
            created_at: Option<String>,
            started_at: Option<String>,
            finished_at: Option<String>,
            web_url: String,
        }

        let jobs: Vec<GitLabJobItem> = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!(
                "Failed to parse GitLab pipeline jobs: {}",
                error
            ))
        })?;

        let result_jobs = jobs
            .into_iter()
            .map(|job| {
                let (status, conclusion) = match job.status.as_str() {
                    "success" => ("completed".to_string(), Some("success".to_string())),
                    "failed" => ("completed".to_string(), Some("failure".to_string())),
                    "running" => ("in_progress".to_string(), None),
                    "pending" | "waiting_for_resource" | "preparing" => {
                        ("queued".to_string(), None)
                    }
                    "canceled" => ("completed".to_string(), Some("cancelled".to_string())),
                    "skipped" => ("completed".to_string(), Some("skipped".to_string())),
                    other => (other.to_string(), None),
                };

                let started_timestamp = job
                    .started_at
                    .as_deref()
                    .and_then(|iso| chrono::DateTime::parse_from_rfc3339(iso).ok())
                    .map(|datetime| datetime.timestamp());
                let completed_timestamp = job
                    .finished_at
                    .as_deref()
                    .and_then(|iso| chrono::DateTime::parse_from_rfc3339(iso).ok())
                    .map(|datetime| datetime.timestamp());

                WorkflowJob {
                    id: job.id.to_string(),
                    run_id: pipeline_id.to_string(),
                    name: job.name,
                    status,
                    conclusion,
                    started_at_timestamp: started_timestamp,
                    completed_at_timestamp: completed_timestamp,
                    steps: Vec::new(),
                    html_url: Some(job.web_url),
                }
            })
            .collect();

        Ok(result_jobs)
    }

    pub async fn retry_pipeline(
        &self,
        token: &str,
        project_full_name: &str,
        pipeline_id: u64,
    ) -> Result<bool, ApplicationError> {
        let encoded_project = urlencoding_simple(project_full_name);
        let url = format!(
            "{}/projects/{}/pipelines/{}/retry",
            self.api_base_url, encoded_project, pipeline_id
        );

        let headers = self.build_headers(token)?;
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| {
                ApplicationError::ProviderError(format!("GitLab network error: {}", error))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!(
                "Failed to retry pipeline: {}",
                error_text
            )));
        }

        Ok(true)
    }

    pub async fn cancel_pipeline(
        &self,
        token: &str,
        project_full_name: &str,
        pipeline_id: u64,
    ) -> Result<bool, ApplicationError> {
        let encoded_project = urlencoding_simple(project_full_name);
        let url = format!(
            "{}/projects/{}/pipelines/{}/cancel",
            self.api_base_url, encoded_project, pipeline_id
        );

        let headers = self.build_headers(token)?;
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| {
                ApplicationError::ProviderError(format!("GitLab network error: {}", error))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!(
                "Failed to cancel pipeline: {}",
                error_text
            )));
        }

        Ok(true)
    }

    pub async fn get_job_trace(
        &self,
        token: &str,
        project_full_name: &str,
        job_id: u64,
    ) -> Result<String, ApplicationError> {
        let encoded_project = urlencoding_simple(project_full_name);
        let url = format!(
            "{}/projects/{}/jobs/{}/trace",
            self.api_base_url, encoded_project, job_id
        );

        let headers = self.build_headers(token)?;
        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| {
                ApplicationError::ProviderError(format!("GitLab network error: {}", error))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!(
                "Failed to get GitLab job trace: {}",
                error_text
            )));
        }

        let trace = response.text().await.unwrap_or_default();
        Ok(trace)
    }

    /// Creates a new merge request on GitLab.
    pub async fn create_merge_request(
        &self,
        token: &str,
        project_full_name: &str,
        title: &str,
        description: Option<&str>,
        source_branch: &str,
        target_branch: &str,
        is_draft: bool,
    ) -> Result<RemotePullRequest, ApplicationError> {
        let encoded_project = urlencoding_simple(project_full_name);
        let url = format!(
            "{}/projects/{}/merge_requests",
            self.api_base_url, encoded_project
        );
        let headers = self.build_headers(token)?;

        let formatted_title =
            if is_draft && !title.starts_with("Draft:") && !title.starts_with("WIP:") {
                format!("Draft: {}", title)
            } else {
                title.to_string()
            };

        #[derive(Serialize)]
        struct CreateMrPayload<'a> {
            title: &'a str,
            source_branch: &'a str,
            target_branch: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<&'a str>,
        }

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&CreateMrPayload {
                title: &formatted_title,
                source_branch,
                target_branch,
                description,
            })
            .send()
            .await
            .map_err(|error| {
                ApplicationError::ProviderError(format!("GitLab network error: {}", error))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApplicationError::ProviderError(format!(
                "Failed to create GitLab merge request: {}",
                error_text
            )));
        }

        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct GitLabCreateMrResponse {
            id: u64,
            iid: u64,
            title: String,
            description: Option<String>,
            state: String,
            web_url: String,
            author: GitLabUserResponse,
            source_branch: String,
            target_branch: String,
            work_in_progress: Option<bool>,
            draft: Option<bool>,
            user_notes_count: Option<u32>,
            created_at: String,
            updated_at: String,
        }

        let raw: GitLabCreateMrResponse = response.json().await.map_err(|error| {
            ApplicationError::ProviderError(format!("Failed to parse created MR: {}", error))
        })?;

        let created_timestamp = chrono::DateTime::parse_from_rfc3339(&raw.created_at)
            .map(|dt| dt.timestamp())
            .unwrap_or_else(|_| Utc::now().timestamp());

        let updated_timestamp = chrono::DateTime::parse_from_rfc3339(&raw.updated_at)
            .map(|dt| dt.timestamp())
            .unwrap_or_else(|_| Utc::now().timestamp());

        let is_mr_draft = raw.draft.unwrap_or(false)
            || raw.work_in_progress.unwrap_or(false)
            || raw.title.starts_with("Draft:")
            || raw.title.starts_with("WIP:");

        Ok(RemotePullRequest {
            id: format!("gitlab:{}", raw.id),
            number: raw.iid,
            title: raw.title,
            description: raw.description,
            state: PullRequestState::Open,
            author_username: raw.author.username,
            author_avatar_url: raw.author.avatar_url,
            source_branch: raw.source_branch,
            target_branch: raw.target_branch,
            is_draft: is_mr_draft,
            comments_count: raw.user_notes_count.unwrap_or(0),
            html_url: raw.web_url,
            created_at_timestamp: created_timestamp,
            updated_at_timestamp: updated_timestamp,
            merged_at_timestamp: None,
            pipeline_status: None,
        })
    }
}

fn urlencoding_simple(raw: &str) -> String {
    raw.replace('/', "%2F")
}
