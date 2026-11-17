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
}

export interface StashRecord {
  index: number
  stash_identifier: string
  message: string
  commit_hash: string
  timestamp_seconds: number
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





