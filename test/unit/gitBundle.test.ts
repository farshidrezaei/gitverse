import { describe, expect, it } from 'vitest'
import type { GitBundleVerificationResult } from '~/types/repository'

describe('git bundle verification model', () => {
  const validBundle: GitBundleVerificationResult = {
    is_valid: true,
    bundle_file_path: '/tmp/repo.bundle',
    references: ['refs/heads/main', 'refs/tags/v1.0.0'],
    message: 'Bundle is valid and compatible',
  }

  const invalidBundle: GitBundleVerificationResult = {
    is_valid: false,
    bundle_file_path: '/tmp/corrupt.bundle',
    references: [],
    message: 'fatal: error reading bundle',
  }

  it('correctly models valid bundle with extracted references', () => {
    expect(validBundle.is_valid).toBe(true)
    expect(validBundle.references.length).toBe(2)
    expect(validBundle.references).toContain('refs/heads/main')
  })

  it('correctly models invalid bundle with zero references and error message', () => {
    expect(invalidBundle.is_valid).toBe(false)
    expect(invalidBundle.references).toEqual([])
    expect(invalidBundle.message).toContain('fatal')
  })
})
