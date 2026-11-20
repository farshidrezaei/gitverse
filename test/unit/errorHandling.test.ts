import { describe, expect, it } from 'vitest'
import { formatApplicationError } from '~/utils/tauriClient'

describe('Application Error Formatting & Crash Prevention', () => {
  it('formats standard Error instance', () => {
    const error = new Error('File not found')
    expect(formatApplicationError(error)).toBe('File not found')
  })

  it('formats raw string error', () => {
    expect(formatApplicationError('Direct string failure')).toBe('Direct string failure')
  })

  it('unpacks Tauri serialized IPC error object without returning [object Object]', () => {
    const tauriError = {
      type: 'GitExecutionFailed',
      message: 'fatal: destination path already exists and is not an empty directory.',
    }

    const result = formatApplicationError(tauriError)
    expect(result).toBe('fatal: destination path already exists and is not an empty directory.')
    expect(result).not.toContain('[object Object]')
  })

  it('unpacks typed error with no message by falling back to error type identifier', () => {
    const tauriError = {
      type: 'RateLimitExceeded',
    }

    const result = formatApplicationError(tauriError)
    expect(result).toBe('RateLimitExceeded')
    expect(result).not.toContain('[object Object]')
  })

  it('unpacks error object carrying stderr or error fields', () => {
    const stderrError = {
      stderr: 'error: failed to push some refs to remote',
    }
    expect(formatApplicationError(stderrError)).toBe('error: failed to push some refs to remote')

    const nestedError = {
      error: 'Connection refused',
    }
    expect(formatApplicationError(nestedError)).toBe('Connection refused')
  })

  it('provides safe fallback on null, undefined or empty input', () => {
    expect(formatApplicationError(null, 'Custom fallback')).toBe('Custom fallback')
    expect(formatApplicationError(undefined, 'Custom fallback')).toBe('Custom fallback')
    expect(formatApplicationError('')).toBe('An unexpected error occurred')
  })
})
