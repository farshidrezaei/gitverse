export interface GitBinaryInformation {
  executable_path: string
  version_string: string
  is_valid: boolean
  has_porcelain_v2_support: boolean
}

export interface RepositoryOverview {
  path: string
  name: string
  current_branch_name: string | null
  head_commit_hash: string | null
  is_empty: boolean
  is_detached_head: boolean
  working_tree_is_clean: boolean
  ahead_count: number
  behind_count: number
}

export interface RecentRepositoryRecord {
  id: string
  path: string
  name: string
  last_opened_timestamp: number
  is_pinned: boolean
  is_missing?: boolean
}

export type FileChangeStatus =
  | 'modified'
  | 'added'
  | 'deleted'
  | 'renamed'
  | 'copied'
  | 'typechanged'
  | 'untracked'
  | 'conflicted'

export interface FileStatusEntry {
  path: string
  original_path: string | null
  status: FileChangeStatus
  is_staged: boolean
}

export type RepositoryOperationState =
  | 'none'
  | 'merging'
  | 'rebasing'
  | 'cherrypicking'
  | 'reverting'

export interface ConflictFileDetails {
  file_path: string
  base_content: string | null
  ours_content: string | null
  theirs_content: string | null
  working_file_content: string
}

export interface DetailedRepositoryStatus {
  branch_name: string | null
  head_commit_hash: string | null
  upstream_branch_name: string | null
  ahead_count: number
  behind_count: number
  is_detached_head: boolean
  staged_files: FileStatusEntry[]
  unstaged_files: FileStatusEntry[]
  conflicted_files: FileStatusEntry[]
  operation_state: RepositoryOperationState
}

export type DiffLineType = 'context' | 'addition' | 'deletion' | 'header' | 'modification'

export interface DiffLine {
  line_type: DiffLineType
  old_line_number: number | null
  new_line_number: number | null
  content: string
}

export interface DiffHunk {
  header: string
  old_start: number
  old_lines: number
  new_start: number
  new_lines: number
  lines: DiffLine[]
}

export interface FileDiff {
  old_path: string | null
  new_path: string
  is_binary: boolean
  is_new_file: boolean
  is_deleted_file: boolean
  hunks: DiffHunk[]
  total_additions: number
  total_deletions: number
  old_content_base64?: string | null
  new_content_base64?: string | null
}

export type GitReferenceType = 'localbranch' | 'remotebranch' | 'tag' | 'head'

export interface GitCommitDecoration {
  name: string
  reference_type: GitReferenceType
  is_head: boolean
}

export interface GitCommit {
  hash: string
  parent_hashes: string[]
  author_name: string
  author_email: string
  author_timestamp: number
  committer_name: string
  committer_email: string
  committer_timestamp: number
  subject: string
  body: string
  decorations: GitCommitDecoration[]
}

export interface GraphEdge {
  parent_hash: string
  from_lane: number
  to_lane: number
  is_merge_edge: boolean
}

export interface CommitGraphNode {
  commit: GitCommit
  lane: number
  edges: GraphEdge[]
  is_head: boolean
  is_merge: boolean
}

export interface CommitGraphHistoryResponse {
  nodes: CommitGraphNode[]
  max_lane_index: number
  total_commits_count: number | null
  has_more_commits: boolean
}

export interface BranchSummary {
  name: string
  is_head: boolean
  upstream_name: string | null
  ahead_count: number
  behind_count: number
  head_commit_hash: string | null
}

export interface RemoteBranchSummary {
  name: string
  remote_name: string
  head_commit_hash: string | null
}

export interface TagSummary {
  name: string
  commit_hash: string
}

export interface RemoteSummary {
  name: string
  fetch_url: string | null
  push_url: string | null
}

export interface RepositoryReferences {
  branches: BranchSummary[]
  remote_branches: RemoteBranchSummary[]
  tags: TagSummary[]
  remotes: RemoteSummary[]
}

export interface CommitChangedFile {
  path: string
  original_path: string | null
  status: FileChangeStatus
  additions: number
  deletions: number
}

export type CommitSignatureStatus = 'verified' | 'unverified' | 'none'

export interface CommitSignature {
  status: CommitSignatureStatus
  signer_name: string | null
  key_id: string | null
}

export interface CommitDetails {
  hash: string
  parent_hashes: string[]
  author_name: string
  author_email: string
  author_timestamp: number
  committer_name: string
  committer_email: string
  committer_timestamp: number
  subject: string
  body: string
  decorations: GitCommitDecoration[]
  total_additions: number
  total_deletions: number
  changed_files: CommitChangedFile[]
  signature?: CommitSignature | null
}

export interface StashRecord {
  index: number
  stash_identifier: string
  message: string
  commit_hash: string
  timestamp_seconds: number
}

export interface StashDetails {
  index: number
  stash_identifier: string
  message: string
  commit_hash: string
  timestamp_seconds: number
  total_additions: number
  total_deletions: number
  changed_files: CommitChangedFile[]
}

export interface ReflogRecord {
  selector: string
  commit_hash: string
  full_hash: string
  action: string
  timestamp_seconds: number
  author_name: string
}

export interface BisectState {
  is_active: boolean
  current_commit_hash?: string | null
  current_commit_summary?: string | null
  revisions_remaining?: number | null
  estimated_steps_remaining?: number | null
  culprit_commit_hash?: string | null
  culprit_commit_summary?: string | null
  log_output: string
}

export type PatchExportScope = 'staged' | 'unstaged' | 'working_tree' | 'commit'

export interface PatchCheckResult {
  can_apply: boolean
  error_message?: string | null
  affected_files: string[]
}

export interface GitSubmoduleSummary {
  path: string
  name: string
  url?: string | null
  branch?: string | null
  current_commit_hash: string
  is_initialized: boolean
  has_uncommitted_changes: boolean
  has_revision_divergence: boolean
}

export interface ContributorInsight {
  name: string
  email: string
  commit_count: number
  percentage: number
}

export interface ActivityDay {
  day_of_week: number
  count: number
}

export interface RepositoryInsights {
  total_commits: number
  total_contributors: number
  top_contributors: ContributorInsight[]
  activity_by_weekday: ActivityDay[]
  lines_added: number
  lines_deleted: number
}

export interface FileRevisionEntry {
  commit_hash: string
  author_name: string
  author_email: string
  author_timestamp: number
  commit_summary: string
  change_status: string
}

export interface FileHistoryTimeline {
  file_path: string
  revisions: FileRevisionEntry[]
  total_revisions: number
}

export interface MergeBranchOutcome {
  is_successful: boolean
  has_conflicts: boolean
  message: string
}

export interface RebaseBranchOutcome {
  is_successful: boolean
  has_conflicts: boolean
  message: string
}

export type RebaseAction = 'pick' | 'reword' | 'edit' | 'squash' | 'fixup' | 'drop'

export interface RebasePlanItem {
  action: RebaseAction
  commit_hash: string
  short_hash: string
  original_subject: string
  new_message: string | null
  author_name: string
}

export interface InteractiveRebaseRequest {
  base_commit_hash: string
  plan_items: RebasePlanItem[]
}

export interface CommitComparison {
  base_hash: string
  target_hash: string
  base_subject: string | null
  target_subject: string | null
  total_additions: number
  total_deletions: number
  changed_files: CommitChangedFile[]
}

export interface FolderEntry {
  name: string
  path: string
  is_git_repository: boolean
}

export interface DirectoryListing {
  current_path: string
  parent_path: string | null
  folders: FolderEntry[]
  is_current_git_repository: boolean
}

export interface GitWorktreeSummary {
  directory_path: string
  head_commit_hash: string
  branch_name: string | null
  is_bare: boolean
  is_locked: boolean
  lock_reason: string | null
  is_prunable: boolean
  is_main: boolean
}

export interface ApplicationDiagnostics {
  app_version: string
  os_name: string
  os_arch: string
  git_binary_path: string
  git_version: string
  current_repository_path: string | null
}

export interface BlameHunk {
  commit_hash: string
  author_name: string
  author_email: string
  author_timestamp: number
  commit_summary: string
  original_line_number: number
  final_line_number: number
  line_content: string
}

export interface FileBlame {
  file_path: string
  lines: BlameHunk[]
}

export interface FileRevisionEntry {
  commit_hash: string
  author_name: string
  author_email: string
  author_timestamp: number
  commit_summary: string
  change_status: string
}

export interface FileHistoryTimeline {
  file_path: string
  revisions: FileRevisionEntry[]
  total_revisions: number
}

export interface SearchMatchEntry {
  file_path: string
  line_number: number
  line_content: string
}

export interface RepositorySearchResult {
  query: string
  total_matches: number
  file_count: number
  matches: SearchMatchEntry[]
}

