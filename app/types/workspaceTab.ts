import type { FileChangeStatus } from '~/types/repository'

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
