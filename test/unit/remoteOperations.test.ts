import { describe, expect, it } from 'vitest'
import type { RemoteSummary } from '~/types/repository'

describe('remoteOperations', () => {
  it('correctly models remote with fetch and push URLs', () => {
    const remote: RemoteSummary = {
      name: 'origin',
      fetch_url: 'https://github.com/gitverse/core.git',
      push_url: 'git@github.com:gitverse/core.git',
    }

    expect(remote.name).toBe('origin')
    expect(remote.fetch_url).toBe('https://github.com/gitverse/core.git')
    expect(remote.push_url).toBe('git@github.com:gitverse/core.git')
  })

  it('validates remote name rules: disallows spaces and empty strings', () => {
    function isValidRemoteName(name: string, existing: string[] = []): boolean {
      const trimmed = name.trim()
      if (!trimmed) return false
      if (/\s/.test(trimmed)) return false
      if (existing.includes(trimmed)) return false
      return true
    }

    expect(isValidRemoteName('origin')).toBe(true)
    expect(isValidRemoteName('upstream')).toBe(true)
    expect(isValidRemoteName('my-remote_1')).toBe(true)
    expect(isValidRemoteName('')).toBe(false)
    expect(isValidRemoteName('   ')).toBe(false)
    expect(isValidRemoteName('origin with space')).toBe(false)
    expect(isValidRemoteName('origin', ['origin'])).toBe(false)
  })

  it('validates remote URL rules: allows https and ssh formats', () => {
    function isValidRemoteUrl(url: string): boolean {
      const trimmed = url.trim()
      if (!trimmed) return false
      return trimmed.startsWith('https://') || trimmed.startsWith('http://') || trimmed.startsWith('git@') || trimmed.startsWith('ssh://')
    }

    expect(isValidRemoteUrl('https://github.com/user/repo.git')).toBe(true)
    expect(isValidRemoteUrl('git@github.com:user/repo.git')).toBe(true)
    expect(isValidRemoteUrl('ssh://git@gitlab.com/user/repo.git')).toBe(true)
    expect(isValidRemoteUrl('')).toBe(false)
    expect(isValidRemoteUrl('ftp://invalid.com')).toBe(false)
  })
})
