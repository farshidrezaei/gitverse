import { describe, it, expect } from 'vitest'
import { parseRepositoryFromUrl } from '~/utils/providerUrl'

describe('providerUrl Utility', () => {
  it('parses GitHub SSH SCP syntax', () => {
    const result = parseRepositoryFromUrl('git@github.com:facebook/react.git')
    expect(result).toEqual({
      host: 'github.com',
      fullName: 'facebook/react',
    })
  })

  it('parses GitLab SSH SCP syntax without .git suffix', () => {
    const result = parseRepositoryFromUrl('git@gitlab.com:gitlab-org/gitlab')
    expect(result).toEqual({
      host: 'gitlab.com',
      fullName: 'gitlab-org/gitlab',
    })
  })

  it('parses HTTPS URL with .git suffix', () => {
    const result = parseRepositoryFromUrl('https://github.com/vuejs/core.git')
    expect(result).toEqual({
      host: 'github.com',
      fullName: 'vuejs/core',
    })
  })

  it('parses HTTPS URL without .git suffix', () => {
    const result = parseRepositoryFromUrl('https://gitlab.com/group/subproject')
    expect(result).toEqual({
      host: 'gitlab.com',
      fullName: 'group/subproject',
    })
  })

  it('returns null for invalid or non-remote URLs', () => {
    expect(parseRepositoryFromUrl('')).toBeNull()
    expect(parseRepositoryFromUrl('/home/user/repo')).toBeNull()
    expect(parseRepositoryFromUrl('invalid-url')).toBeNull()
  })
})
