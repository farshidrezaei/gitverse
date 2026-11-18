import { describe, expect, it } from 'vitest'
import { useRebaseController } from '~/composables/useRebaseController'

describe('useRebaseController', () => {
  it('initializes with non-rebasing state', () => {
    const { isRebasing } = useRebaseController()
    expect(isRebasing.value).toBe(false)
  })

  it('returns empty list when no current repository', async () => {
    const { fetchRebaseCommits } = useRebaseController()
    const commits = await fetchRebaseCommits('dummy-hash')
    expect(Array.isArray(commits)).toBe(true)
  })
})
