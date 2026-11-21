export type ProviderKind = 'github' | 'gitlab'

export type PullRequestState = 'open' | 'closed' | 'merged'

export type PipelineState = 'success' | 'failed' | 'running' | 'pending' | 'canceled' | 'skipped'

export interface ProviderCapabilities {
  can_create_pull_request: boolean
  can_review_pull_request: boolean
  can_view_pipelines: boolean
  can_list_issues: boolean
  can_manage_releases: boolean
}

export interface RateLimitStatus {
  limit: number
  remaining: number
  resets_at_timestamp: number
}

export interface ProviderAccount {
  id: string
  provider: ProviderKind
  username: string
  display_name: string | null
  email: string | null
  avatar_url: string | null
  instance_url: string
  created_at_timestamp: number
}

export interface PipelineSummary {
  state: PipelineState
  web_url: string | null
  commit_sha: string | null
  total_checks_count: number
  successful_checks_count: number
  failed_checks_count: number
}

export interface RemotePullRequest {
  id: string
  number: number
  title: string
  description: string | null
  state: PullRequestState
  author_username: string
  author_avatar_url: string | null
  source_branch: string
  target_branch: string
  is_draft: boolean
  comments_count: number
  html_url: string
  created_at_timestamp: number
  updated_at_timestamp: number
  merged_at_timestamp: number | null
  pipeline_status: PipelineSummary | null
}

export interface CiCheckRun {
  id: string
  name: string
  status: string
  conclusion: string | null
  html_url: string | null
  started_at_timestamp: number | null
  completed_at_timestamp: number | null
}

export interface RemoteReviewComment {
  id: string
  author_username: string
  author_avatar_url: string | null
  body: string
  path: string | null
  line_number: number | null
  created_at_timestamp: number
  updated_at_timestamp: number
}

export interface PullRequestChangedFile {
  file_path: string
  previous_file_path: string | null
  status: string
  additions_count: number
  deletions_count: number
  changes_count: number
  patch: string | null
}

export type PullRequestReviewAction = 'APPROVE' | 'REQUEST_CHANGES' | 'COMMENT'

export type PullRequestMergeMethod = 'merge' | 'squash' | 'rebase'

export interface PullRequestMergeResult {
  is_merged: boolean
  message: string
  commit_sha: string | null
}

export interface RemoteProjectRepository {
  id: string
  name: string
  full_name: string
  description: string | null
  is_private: boolean
  is_fork: boolean
  default_branch: string
  clone_url_http: string
  clone_url_ssh: string
  html_url: string
  open_issues_count: number
  stars_count: number
  forks_count: number
  updated_at_timestamp: number
}

export interface WorkflowJobStep {
  name: string
  status: string
  conclusion: string | null
  number: number
  started_at_timestamp: number | null
  completed_at_timestamp: number | null
}

export interface WorkflowJob {
  id: string
  run_id: string
  name: string
  status: string
  conclusion: string | null
  started_at_timestamp: number | null
  completed_at_timestamp: number | null
  steps: WorkflowJobStep[]
  html_url: string | null
}

export interface WorkflowRun {
  id: string
  workflow_name: string
  display_title: string
  head_branch: string
  head_commit_sha: string
  status: string
  conclusion: string | null
  event: string
  run_number: number
  author_username: string
  author_avatar_url: string | null
  created_at_timestamp: number
  updated_at_timestamp: number
  html_url: string | null
}
