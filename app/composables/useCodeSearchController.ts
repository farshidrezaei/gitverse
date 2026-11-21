import type { RepositorySearchResult } from '~/types/repository'
import { formatApplicationError } from '~/utils/tauriClient'

export interface CodeSearchOptions {
  reference?: string | null
  pathPattern?: string | null
  isCaseSensitive?: boolean
  isWordMatch?: boolean
  isRegex?: boolean
}

export function useCodeSearchController() {
  const { currentRepository } = useRepositoryWorkspace()
  const { notifyError } = useNotificationService()

  const searchResults = useState<RepositorySearchResult | null>('workspace-code-search-results', () => null)
  const isSearching = useState<boolean>('workspace-is-searching-code', () => false)

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function searchRepository(
    query: string,
    options: CodeSearchOptions = {},
  ): Promise<RepositorySearchResult | null> {
    if (!currentRepository.value || !query.trim()) {
      searchResults.value = null
      return null
    }

    try {
      isSearching.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const result = await invoke<RepositorySearchResult>('search_repository', {
          directoryPath: currentRepository.value.path,
          query: query.trim(),
          reference: options.reference || null,
          pathPattern: options.pathPattern || null,
          isCaseSensitive: options.isCaseSensitive ?? false,
          isWordMatch: options.isWordMatch ?? false,
          isRegex: options.isRegex ?? false,
        })
        searchResults.value = result
        return result
      }

      // Mock search result in browser/mock environment
      const mockResult: RepositorySearchResult = {
        query: query.trim(),
        total_matches: 2,
        file_count: 2,
        matches: [
          {
            file_path: 'src/main.rs',
            line_number: 42,
            line_content: `    // match for ${query}`,
          },
          {
            file_path: 'app/app.vue',
            line_number: 15,
            line_content: `<template><div>${query}</div></template>`,
          },
        ],
      }
      searchResults.value = mockResult
      return mockResult
    } catch (error) {
      console.error('Failed to execute code search:', error)
      notifyError(formatApplicationError(error))
      return null
    } finally {
      isSearching.value = false
    }
  }

  function clearSearchResults(): void {
    searchResults.value = null
  }

  return {
    searchResults,
    isSearching,
    searchRepository,
    clearSearchResults,
  }
}
