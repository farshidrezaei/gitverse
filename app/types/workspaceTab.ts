import type { FileChangeStatus } from '~/types/repository'

export type WorkspaceTabType = 'workspace' | 'diff'

export interface BaseWorkspaceTab {
  id: string
  title: string
  type: WorkspaceTabType
  isClosable: boolean
  isPinned: boolean
}

export interface WorkspaceMainTab extends BaseWorkspaceTab {
  type: 'workspace'
  repositoryPath: string
  repositoryName: string
}

export interface WorkspaceDiffTab extends BaseWorkspaceTab {
  type: 'diff'
  filePath: string
  isStaged?: boolean
  commitHash?: string
  changeStatus?: FileChangeStatus
}

export type WorkspaceTab = WorkspaceMainTab | WorkspaceDiffTab

export interface OpenDiffTabOptions {
  filePath: string
  isStaged?: boolean
  commitHash?: string
  changeStatus?: FileChangeStatus
  isPinned?: boolean
}
