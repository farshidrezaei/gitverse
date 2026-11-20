import { describe, expect, it } from 'vitest'
import type { GitSubmoduleSummary } from '~/types/repository'

describe('git submodule parsing and status classification', () => {
  it('identifies submodule initialization and divergence state', () => {
    const mockSubmodules: GitSubmoduleSummary[] = [
      {
        path: 'libs/engine',
        name: 'engine',
        url: 'https://github.com/org/engine.git',
        branch: 'main',
        current_commit_hash: '0afa6c2685f487180f6b074d106da34149f51f38',
        is_initialized: true,
        has_uncommitted_changes: false,
        has_revision_divergence: false,
      },
      {
        path: 'vendor/thirdparty',
        name: 'thirdparty',
        url: 'https://github.com/third/party.git',
        branch: null,
        current_commit_hash: '1111111111111111111111111111111111111111',
        is_initialized: false,
        has_uncommitted_changes: false,
        has_revision_divergence: false,
      },
      {
        path: 'plugins/diverged',
        name: 'diverged',
        url: null,
        branch: 'v2.0.0',
        current_commit_hash: '2222222222222222222222222222222222222222',
        is_initialized: true,
        has_uncommitted_changes: false,
        has_revision_divergence: true,
      },
    ]

    function getSubmoduleStatusBadgeColor(
      submodule: GitSubmoduleSummary
    ): 'success' | 'warning' | 'error' | 'neutral' {
      if (!submodule.is_initialized) return 'neutral'
      if (submodule.has_uncommitted_changes) return 'error'
      if (submodule.has_revision_divergence) return 'warning'
      return 'success'
    }

    expect(getSubmoduleStatusBadgeColor(mockSubmodules[0])).toBe('success')
    expect(getSubmoduleStatusBadgeColor(mockSubmodules[1])).toBe('neutral')
    expect(getSubmoduleStatusBadgeColor(mockSubmodules[2])).toBe('warning')
  })

  it('computes submodule target directory path correctly', () => {
    function getSubmoduleAbsolutePath(parentPath: string, submodulePath: string): string {
      const sanitizedParent = parentPath.endsWith('/') ? parentPath.slice(0, -1) : parentPath
      const sanitizedChild = submodulePath.startsWith('/') ? submodulePath.slice(1) : submodulePath
      return `${sanitizedParent}/${sanitizedChild}`
    }

    expect(
      getSubmoduleAbsolutePath('/home/user/project', 'libs/engine')
    ).toBe('/home/user/project/libs/engine')
    expect(
      getSubmoduleAbsolutePath('/home/user/project/', '/libs/engine')
    ).toBe('/home/user/project/libs/engine')
  })
})
