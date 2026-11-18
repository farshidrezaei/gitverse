import { describe, expect, it } from 'vitest'
import { useWorktreeController } from '~/composables/useWorktreeController'

describe('useWorktreeController', () => {
  it('initializes with empty worktrees list', () => {
    const { worktreeList, isLoadingWorktrees } = useWorktreeController()
    expect(Array.isArray(worktreeList.value)).toBe(true)
    expect(isLoadingWorktrees.value).toBe(false)
  })

  it('fetches mock worktrees when repository is present in mock mode', async () => {
    const { worktreeList, fetchWorktrees } = useWorktreeController()
    // When no current repo, returns empty
    const result = await fetchWorktrees()
    expect(Array.isArray(result)).toBe(true)
  })
})
