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
}
