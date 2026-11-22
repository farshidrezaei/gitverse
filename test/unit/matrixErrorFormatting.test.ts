import { describe, expect, it } from 'vitest'
import { formatApplicationError } from '../../app/utils/tauriClient'

describe('Table-Driven Error Formatting Matrix', () => {
  interface ErrorMatrixTestCase {
    description: string
    input: unknown
    fallback?: string
    expectedOutput: string
  }

  const errorCases: ErrorMatrixTestCase[] = [
    // Standard JS Error instances
    {
      description: 'Standard Error instance',
      input: new Error('Network request failed'),
      expectedOutput: 'Network request failed',
    },
    {
      description: 'Custom TypeError',
      input: new TypeError('Cannot read property of undefined'),
      expectedOutput: 'Cannot read property of undefined',
    },

    // Raw strings
    {
      description: 'Plain string error',
      input: 'fatal: repository not found',
      expectedOutput: 'fatal: repository not found',
    },
    {
      description: 'Multi-line string error',
      input: 'error: failed to push some refs\nhint: Updates were rejected',
      expectedOutput: 'error: failed to push some refs\nhint: Updates were rejected',
    },

    // Tauri serialized structured errors
    {
      description: 'Tauri GitExecutionFailed with message',
      input: {
        type: 'GitExecutionFailed',
        message: 'fatal: Not a git repository (or any of the parent directories)',
      },
      expectedOutput: 'fatal: Not a git repository (or any of the parent directories)',
    },
    {
      description: 'Tauri error with message and exit code',
      input: {
        exit_code: 128,
        message: 'fatal: destination path already exists',
      },
      expectedOutput: 'fatal: destination path already exists',
    },
    {
      description: 'Tauri typed error without message (falls back to type key)',
      input: {
        type: 'AuthenticationRequired',
      },
      expectedOutput: 'AuthenticationRequired',
    },
    {
      description: 'Error payload carrying stderr',
      input: {
        stderr: 'fatal: unable to access: Could not resolve host: github.com',
      },
      expectedOutput: 'fatal: unable to access: Could not resolve host: github.com',
    },
    {
      description: 'Nested error object inside error field',
      input: {
        error: 'Permission denied (publickey)',
      },
      expectedOutput: 'Permission denied (publickey)',
    },
    {
      description: 'Nested JSON error object with message inside error',
      input: {
        error: {
          message: 'API rate limit exceeded for user',
        },
      },
      expectedOutput: 'API rate limit exceeded for user',
    },

    // Edge cases, nullish and primitives
    {
      description: 'null with custom fallback',
      input: null,
      fallback: 'Action failed',
      expectedOutput: 'Action failed',
    },
    {
      description: 'undefined with custom fallback',
      input: undefined,
      fallback: 'Something went wrong',
      expectedOutput: 'Something went wrong',
    },
    {
      description: 'null without fallback defaults to generic message',
      input: null,
      expectedOutput: 'An unexpected error occurred',
    },
    {
      description: 'empty string defaults to generic message',
      input: '',
      expectedOutput: 'An unexpected error occurred',
    },
    {
      description: 'whitespace-only string defaults to generic message',
      input: '   \n\t ',
      expectedOutput: 'An unexpected error occurred',
    },
    {
      description: 'Numeric error code',
      input: 500,
      expectedOutput: '500',
    },
  ]

  it.each(errorCases)(
    'Formats: $description',
    ({ input, fallback, expectedOutput }) => {
      const result = formatApplicationError(input, fallback)
      expect(result).toBe(expectedOutput)
      expect(result).not.toContain('[object Object]')
    }
  )

  it('safely handles circular reference objects without crashing', () => {
    const circular: Record<string, unknown> = { message: 'Circular error' }
    circular.self = circular
    const formatted = formatApplicationError(circular)
    expect(formatted).toBe('Circular error')
    expect(formatted).not.toContain('[object Object]')
  })
})
