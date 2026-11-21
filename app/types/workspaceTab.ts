import type { FileChangeStatus } from '~/types/repository'
import type { RemotePullRequest, ProviderAccount, WorkflowRun } from '~/types/provider'

export interface WorkspaceRepositoryTab {
  id: string
  repositoryPath: string
  repositoryName: string
  isClosable: boolean
}

export interface WorkspaceDiffTab {
  id: string
  title: string
  filePath: string
  repositoryPath: string
  isStaged?: boolean
  commitHash?: string
  comparisonBaseHash?: string
  comparisonTargetHash?: string
  changeStatus?: FileChangeStatus
  isClosable: boolean
  isPinned: boolean
}

export interface WorkspacePullRequestTab {
  id: string
  title: string
  pullRequest: RemotePullRequest
  account: ProviderAccount
  repositoryFullName: string
  repositoryPath: string
  isClosable: boolean
  isPinned: boolean
  activeSubView?: 'conversation' | 'files' | 'checks'
}

export interface WorkspaceWorkflowRunTab {
  id: string
  title: string
  workflowRun: WorkflowRun
  account: ProviderAccount
  repositoryFullName: string
  repositoryPath: string
  isClosable: boolean
  isPinned: boolean
}

export interface OpenDiffTabOptions {
  filePath: string
  repositoryPath?: string
  isStaged?: boolean
  commitHash?: string
  comparisonBaseHash?: string
  comparisonTargetHash?: string
  changeStatus?: FileChangeStatus
  isPinned?: boolean
}

