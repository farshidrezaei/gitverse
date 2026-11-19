import type {
  ProviderAccount,
  ProviderKind,
  RemoteProjectRepository,
  RemotePullRequest,
  PullRequestState,
} from '~/types/provider'
import { invokeTauriIpc } from '~/utils/tauriClient'

export function useProviderController() {
  const accounts = useState<ProviderAccount[]>('provider-accounts', () => [])
  const isLoading = useState<boolean>('provider-loading', () => false)
  const isConnecting = useState<boolean>('provider-connecting', () => false)
  const errorMessage = useState<string | null>('provider-error', () => null)
  const remoteRepositories = useState<RemoteProjectRepository[]>('provider-repositories', () => [])
  const pullRequests = useState<RemotePullRequest[]>('provider-pull-requests', () => [])

  async function loadAccounts(): Promise<ProviderAccount[]> {
    isLoading.value = true
    errorMessage.value = null
    try {
      const result = await invokeTauriIpc<ProviderAccount[]>('list_provider_accounts')
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
      const account = await invokeTauriIpc<ProviderAccount>('connect_github_account', {
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
      const account = await invokeTauriIpc<ProviderAccount>('connect_gitlab_account', {
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
      const isSuccess = await invokeTauriIpc<boolean>('disconnect_provider_account', {
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
      const repositories = await invokeTauriIpc<RemoteProjectRepository[]>('get_provider_repositories', {
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
    errorMessage.value = null
    try {
      const result = await invokeTauriIpc<RemotePullRequest[]>('get_provider_pull_requests', {
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
  }
}
