import { describe, expect, it } from 'vitest'
import type { ReflogRecord } from '~/types/repository'

describe('git reflog filtering and classification', () => {
  const sampleReflogRecords: ReflogRecord[] = [
    {
      selector: 'HEAD@{0}',
      commit_hash: '4ba8dfc',
      full_hash: '4ba8dfc1234567890abcdef',
      action: 'commit: feat(commit): implement conventional commit builder',
      timestamp_seconds: 1700000000,
      author_name: 'Farshid Rezaei',
    },
    {
      selector: 'HEAD@{1}',
      commit_hash: 'c751dff',
      full_hash: 'c751dff1234567890abcdef',
      action: 'checkout: moving from main to dev',
      timestamp_seconds: 1699999000,
      author_name: 'Farshid Rezaei',
    },
    {
      selector: 'HEAD@{2}',
      commit_hash: '91f8b6e',
      full_hash: '91f8b6e1234567890abcdef',
      action: 'rebase (finish): returning to refs/heads/main',
      timestamp_seconds: 1699998000,
      author_name: 'Alice Developer',
    },
  ]

  function filterReflog(records: ReflogRecord[], query: string): ReflogRecord[] {
    const normalized = query.trim().toLowerCase()
    if (!normalized) return records
    return records.filter(
      (record) =>
        record.action.toLowerCase().includes(normalized) ||
        record.commit_hash.toLowerCase().includes(normalized) ||
        record.selector.toLowerCase().includes(normalized) ||
        record.author_name.toLowerCase().includes(normalized)
    )
  }

  it('returns all records when query is empty', () => {
    expect(filterReflog(sampleReflogRecords, '')).toHaveLength(3)
  })

  it('filters by action keyword', () => {
    const commits = filterReflog(sampleReflogRecords, 'commit')
    expect(commits).toHaveLength(1)
    expect(commits[0]?.selector).toBe('HEAD@{0}')

    const checkouts = filterReflog(sampleReflogRecords, 'checkout')
    expect(checkouts).toHaveLength(1)
    expect(checkouts[0]?.selector).toBe('HEAD@{1}')
  })

  it('filters by author name', () => {
    const aliceRecords = filterReflog(sampleReflogRecords, 'Alice')
    expect(aliceRecords).toHaveLength(1)
    expect(aliceRecords[0]?.commit_hash).toBe('91f8b6e')
  })

  it('filters by commit hash', () => {
    const hashMatch = filterReflog(sampleReflogRecords, 'c751dff')
    expect(hashMatch).toHaveLength(1)
    expect(hashMatch[0]?.selector).toBe('HEAD@{1}')
  })
})
