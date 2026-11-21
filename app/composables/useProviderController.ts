import type {
  ProviderAccount,
  ProviderKind,
  RemoteProjectRepository,
  RemotePullRequest,
  PullRequestState,
  PipelineSummary,
  CiCheckRun,
  PullRequestChangedFile,
  PullRequestMergeResult,
  PullRequestReviewAction,
  PullRequestMergeMethod,
  RemoteReviewComment,
  WorkflowRun,
  WorkflowJob,
} from '~/types/provider'
import { invokeTauriCommand } from '~/utils/tauriClient'

export function useProviderController() {
  const accounts = useState<ProviderAccount[]>('provider-accounts', () => [])
  const isLoading = useState<boolean>('provider-loading', () => false)
  const isConnecting = useState<boolean>('provider-connecting', () => false)
  const errorMessage = useState<string | null>('provider-error', () => null)
  const remoteRepositories = useState<RemoteProjectRepository[]>('provider-repositories', () => [])
  const pullRequests = useState<RemotePullRequest[]>('provider-pull-requests', () => [])
  const cachedPullRequestFiles = useState<Record<string, PullRequestChangedFile[]>>('provider-cached-pr-files', () => ({}))
  const cachedCheckRuns = useState<Record<string, CiCheckRun[]>>('provider-cached-check-runs', () => ({}))

  async function loadAccounts(): Promise<ProviderAccount[]> {
    isLoading.value = true
    errorMessage.value = null
    try {
      const result = await invokeTauriCommand<ProviderAccount[]>('list_provider_accounts')
      accounts.value = result || []
      return accounts.value
    } catch (error) {
      errorMessage.value = String(error)
      return []
    } finally {
      isLoading.value = false
    }
  }

  async function connectGitHub(personalAccessToken: string, instanceUrl?: string): Promise<ProviderAccount | null> {
    isConnecting.value = true
    errorMessage.value = null
    try {
      const account = await invokeTauriCommand<ProviderAccount>('connect_github_account', {
        personalAccessToken,
        instanceUrl: instanceUrl?.trim() || null,
      })
      await loadAccounts()
      return account
    } catch (error) {
      errorMessage.value = String(error)
      throw error
    } finally {
      isConnecting.value = false
    }
  }

  async function connectGitLab(personalAccessToken: string, instanceUrl?: string): Promise<ProviderAccount | null> {
    isConnecting.value = true
    errorMessage.value = null
    try {
      const account = await invokeTauriCommand<ProviderAccount>('connect_gitlab_account', {
        personalAccessToken,
        instanceUrl: instanceUrl?.trim() || null,
      })
      await loadAccounts()
      return account
    } catch (error) {
      errorMessage.value = String(error)
      throw error
    } finally {
      isConnecting.value = false
    }
  }

  async function disconnectAccount(accountIdentifier: string): Promise<boolean> {
    isLoading.value = true
    errorMessage.value = null
    try {
      const isSuccess = await invokeTauriCommand<boolean>('disconnect_provider_account', {
        accountIdentifier,
      })
      if (isSuccess) {
        accounts.value = accounts.value.filter((item) => item.id !== accountIdentifier)
      }
      return isSuccess
    } catch (error) {
      errorMessage.value = String(error)
      return false
    } finally {
      isLoading.value = false
    }
  }

  async function fetchRepositories(accountIdentifier: string, page = 1, perPage = 30): Promise<RemoteProjectRepository[]> {
    isLoading.value = true
    errorMessage.value = null
    try {
      const repositories = await invokeTauriCommand<RemoteProjectRepository[]>('get_provider_repositories', {
        accountIdentifier,
        page,
        perPage,
      })
      remoteRepositories.value = repositories || []
      return remoteRepositories.value
    } catch (error) {
      errorMessage.value = String(error)
      return []
    } finally {
      isLoading.value = false
    }
  }

  async function fetchPullRequests(
    accountIdentifier: string,
    repositoryFullName: string,
    stateFilter?: PullRequestState,
  ): Promise<RemotePullRequest[]> {
    isLoading.value = true
    pullRequests.value = []
    errorMessage.value = null
    try {
      const result = await invokeTauriCommand<RemotePullRequest[]>('get_provider_pull_requests', {
        accountIdentifier,
        repositoryFullName,
        stateFilter: stateFilter || null,
      })
      pullRequests.value = result || []
      return pullRequests.value
    } catch (error) {
      errorMessage.value = String(error)
      return []
    } finally {
      isLoading.value = false
    }
  }

  async function fetchCommitPipelineSummary(
    accountIdentifier: string,
    repositoryFullName: string,
    referenceOrSha: string,
  ): Promise<PipelineSummary | null> {
    try {
      return await invokeTauriCommand<PipelineSummary | null>('get_commit_pipeline_summary', {
        accountIdentifier,
        repositoryFullName,
        referenceOrSha,
      })
    } catch (error) {
      return null
    }
  }

  async function fetchCheckRuns(
    accountIdentifier: string,
    repositoryFullName: string,
    referenceOrPipelineId: string,
  ): Promise<CiCheckRun[]> {
    const cacheKey = `${accountIdentifier}:${repositoryFullName}:${referenceOrPipelineId}`
    if (cachedCheckRuns.value[cacheKey]) {
      return cachedCheckRuns.value[cacheKey]
    }
    try {
      const result = await invokeTauriCommand<CiCheckRun[]>('list_check_runs', {
        accountIdentifier,
        repositoryFullName,
        referenceOrPipelineId,
      })
      const runs = result || []
      cachedCheckRuns.value[cacheKey] = runs
      return runs
    } catch (error) {
      return []
    }
  }

  async function fetchPullRequestComments(
    accountIdentifier: string,
    repositoryFullName: string,
    pullRequestNumber: number,
  ): Promise<RemoteReviewComment[]> {
    try {
      const result = await invokeTauriCommand<RemoteReviewComment[]>('get_pull_request_comments', {
        accountIdentifier,
        repositoryFullName,
        pullRequestNumber,
      })
      return result || []
    } catch (error) {
      errorMessage.value = String(error)
      return []
    }
  }

  async function createPullRequestComment(
    accountIdentifier: string,
    repositoryFullName: string,
    pullRequestNumber: number,
    body: string,
  ): Promise<RemoteReviewComment | null> {
    try {
      return await invokeTauriCommand<RemoteReviewComment>('create_pull_request_comment', {
        accountIdentifier,
        repositoryFullName,
        pullRequestNumber,
        body,
      })
    } catch (error) {
      errorMessage.value = String(error)
      return null
    }
  }

  async function fetchPullRequestFiles(
    accountIdentifier: string,
    repositoryFullName: string,
    pullRequestNumber: number,
  ): Promise<PullRequestChangedFile[]> {
    const cacheKey = `${accountIdentifier}:${repositoryFullName}:${pullRequestNumber}`
    if (cachedPullRequestFiles.value[cacheKey]) {
      return cachedPullRequestFiles.value[cacheKey]
    }
    try {
      const result = await invokeTauriCommand<PullRequestChangedFile[]>('get_pull_request_files', {
        accountIdentifier,
        repositoryFullName,
        pullRequestNumber,
      })
      const files = result || []
      cachedPullRequestFiles.value[cacheKey] = files
      return files
    } catch (error) {
      errorMessage.value = String(error)
      return []
    }
  }

  async function submitPullRequestReview(
    accountIdentifier: string,
    repositoryFullName: string,
    pullRequestNumber: number,
    action: PullRequestReviewAction,
    body?: string,
  ): Promise<boolean> {
    try {
      return await invokeTauriCommand<boolean>('submit_pull_request_review', {
        accountIdentifier,
        repositoryFullName,
        pullRequestNumber,
        action,
        body: body || null,
      })
    } catch (error) {
      errorMessage.value = String(error)
      return false
    }
  }

  async function mergePullRequest(
    accountIdentifier: string,
    repositoryFullName: string,
    pullRequestNumber: number,
    mergeMethod?: PullRequestMergeMethod,
    commitTitle?: string,
    commitMessage?: string,
    deleteSourceBranch?: boolean,
  ): Promise<PullRequestMergeResult | null> {
    try {
      return await invokeTauriCommand<PullRequestMergeResult>('merge_provider_pull_request', {
        accountIdentifier,
        repositoryFullName,
        pullRequestNumber,
        mergeMethod: mergeMethod || null,
        commitTitle: commitTitle || null,
        commitMessage: commitMessage || null,
        deleteSourceBranch: deleteSourceBranch ?? null,
      })
    } catch (error) {
      errorMessage.value = String(error)
      return null
    }
  }

  async function updatePullRequestState(
    accountIdentifier: string,
    repositoryFullName: string,
    pullRequestNumber: number,
    newState: 'open' | 'closed',
  ): Promise<boolean> {
    try {
      return await invokeTauriCommand<boolean>('update_pull_request_state', {
        accountIdentifier,
        repositoryFullName,
        pullRequestNumber,
        newState,
      })
    } catch (error) {
      errorMessage.value = String(error)
      return false
    }
  }

  async function fetchWorkflowRuns(
    accountIdentifier: string,
    repositoryFullName: string,
    statusFilter?: string,
    page: number = 1,
    perPage: number = 30,
  ): Promise<WorkflowRun[]> {
    try {
      const runs = await invokeTauriCommand<WorkflowRun[]>('list_workflow_runs', {
        accountIdentifier,
        repositoryFullName,
        statusFilter: statusFilter || null,
        page,
        perPage,
      })
      return runs || []
    } catch (error) {
      errorMessage.value = String(error)
      return []
    }
  }

  async function fetchWorkflowRunJobs(
    accountIdentifier: string,
    repositoryFullName: string,
    runId: string | number,
  ): Promise<WorkflowJob[]> {
    try {
      const jobs = await invokeTauriCommand<WorkflowJob[]>('get_workflow_run_jobs', {
        accountIdentifier,
        repositoryFullName,
        runId: Number(runId),
      })
      return jobs || []
    } catch (error) {
      errorMessage.value = String(error)
      return []
    }
  }

  async function rerunWorkflowRun(
    accountIdentifier: string,
    repositoryFullName: string,
    runId: string | number,
    failedJobsOnly: boolean = false,
  ): Promise<boolean> {
    try {
      return await invokeTauriCommand<boolean>('rerun_workflow_run', {
        accountIdentifier,
        repositoryFullName,
        runId: Number(runId),
        failedJobsOnly,
      })
    } catch (error) {
      errorMessage.value = String(error)
      return false
    }
  }

  async function cancelWorkflowRun(
    accountIdentifier: string,
    repositoryFullName: string,
    runId: string | number,
  ): Promise<boolean> {
    try {
      return await invokeTauriCommand<boolean>('cancel_workflow_run', {
        accountIdentifier,
        repositoryFullName,
        runId: Number(runId),
      })
    } catch (error) {
      errorMessage.value = String(error)
      return false
    }
  }

  async function fetchWorkflowJobLogs(
    accountIdentifier: string,
    repositoryFullName: string,
    jobId: string | number,
  ): Promise<string> {
    try {
      const logs = await invokeTauriCommand<string>('get_workflow_job_logs', {
        accountIdentifier,
        repositoryFullName,
        jobId: Number(jobId),
      })
      return logs || ''
    } catch (error) {
      errorMessage.value = String(error)
      return ''
    }
  }

  return {
    accounts,
    isLoading,
    isConnecting,
    errorMessage,
    remoteRepositories,
    pullRequests,
    loadAccounts,
    connectGitHub,
    connectGitLab,
    disconnectAccount,
    fetchRepositories,
    fetchPullRequests,
    fetchCommitPipelineSummary,
    fetchCheckRuns,
    fetchPullRequestComments,
    createPullRequestComment,
    fetchPullRequestFiles,
    submitPullRequestReview,
    mergePullRequest,
    updatePullRequestState,
    fetchWorkflowRuns,
    fetchWorkflowRunJobs,
    fetchWorkflowJobLogs,
    rerunWorkflowRun,
    cancelWorkflowRun,
  }
}
