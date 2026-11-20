import { describe, expect, it } from 'vitest'

describe('conventional commit builder', () => {
  function formatConventionalCommit(options: {
    type: string
    scope?: string
    isBreakingChange?: boolean
    summary: string
    body?: string
    issuesClosed?: string
  }): string {
    const cleanScope = options.scope?.trim()
    const breakingMarker = options.isBreakingChange ? '!' : ''
    const scopeSegment = cleanScope ? `(${cleanScope})` : ''
    const header = `${options.type}${scopeSegment}${breakingMarker}: ${options.summary.trim()}`

    const segments: string[] = [header]

    if (options.body?.trim()) {
      segments.push('')
      segments.push(options.body.trim())
    }

    const footers: string[] = []
    if (options.isBreakingChange) {
      footers.push('BREAKING CHANGE: This commit introduces breaking changes.')
    }
    if (options.issuesClosed?.trim()) {
      footers.push(options.issuesClosed.trim())
    }

    if (footers.length > 0) {
      segments.push('')
      segments.push(footers.join('\n'))
    }

    return segments.join('\n')
  }

  it('formats standard commit without scope or body', () => {
    const result = formatConventionalCommit({
      type: 'feat',
      summary: 'add conventional commit builder',
    })
    expect(result).toBe('feat: add conventional commit builder')
  })

  it('formats commit with scope', () => {
    const result = formatConventionalCommit({
      type: 'fix',
      scope: 'auth',
      summary: 'resolve token expiration race condition',
    })
    expect(result).toBe('fix(auth): resolve token expiration race condition')
  })

  it('formats breaking change with exclamation mark and footer', () => {
    const result = formatConventionalCommit({
      type: 'feat',
      scope: 'api',
      isBreakingChange: true,
      summary: 'drop deprecated legacy endpoint',
    })
    expect(result).toBe(
      'feat(api)!: drop deprecated legacy endpoint\n\nBREAKING CHANGE: This commit introduces breaking changes.'
    )
  })

  it('includes detailed body and closed issues footer', () => {
    const result = formatConventionalCommit({
      type: 'refactor',
      scope: 'graph',
      summary: 'optimize DAG lane assignment',
      body: 'Pre-sort commit parents to avoid topological inversions in large repositories.',
      issuesClosed: 'Closes #42',
    })
    expect(result).toBe(
      'refactor(graph): optimize DAG lane assignment\n\nPre-sort commit parents to avoid topological inversions in large repositories.\n\nCloses #42'
    )
  })
})
