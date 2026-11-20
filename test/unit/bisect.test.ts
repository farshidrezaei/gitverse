import { describe, expect, it } from 'vitest'
import type { BisectState } from '~/types/repository'

describe('git bisect assistant logic', () => {
  it('determines whether bisect session can be initiated', () => {
    function canInitiateBisect(badRef: string, goodRef: string): boolean {
      return badRef.trim().length > 0 && goodRef.trim().length > 0
    }

    expect(canInitiateBisect('', 'v1.0.0')).toBe(false)
    expect(canInitiateBisect('HEAD', '')).toBe(false)
    expect(canInitiateBisect('   ', 'main')).toBe(false)
    expect(canInitiateBisect('HEAD', 'v1.0.0')).toBe(true)
    expect(canInitiateBisect('main', '7a2b3c4')).toBe(true)
  })

  it('correctly detects culprit commit presence', () => {
    const inProgressState: BisectState = {
      is_active: true,
      current_commit_hash: 'c2381b329e528c4ee96f237ef319e27be8313e54',
      current_commit_summary: 'commit 2',
      revisions_remaining: 3,
      estimated_steps_remaining: 2,
      culprit_commit_hash: null,
      culprit_commit_summary: null,
      log_output: 'git bisect start',
    }

    const completedState: BisectState = {
      is_active: true,
      current_commit_hash: 'c86466e99565eefe9891747df843197c13c43e57',
      current_commit_summary: 'commit 3',
      revisions_remaining: 0,
      estimated_steps_remaining: 0,
      culprit_commit_hash: 'c86466e99565eefe9891747df843197c13c43e57',
      culprit_commit_summary: 'commit 3 introducing regression',
      log_output: 'c86466e is the first bad commit',
    }

    function hasCulprit(state: BisectState | null): boolean {
      return Boolean(state?.is_active && state.culprit_commit_hash)
    }

    expect(hasCulprit(null)).toBe(false)
    expect(hasCulprit(inProgressState)).toBe(false)
    expect(hasCulprit(completedState)).toBe(true)
  })

  it('validates bisect step decisions', () => {
    const allowedDecisions = ['good', 'bad', 'skip'] as const

    function isValidDecision(decision: string): boolean {
      return allowedDecisions.includes(decision.toLowerCase() as (typeof allowedDecisions)[number])
    }

    expect(isValidDecision('good')).toBe(true)
    expect(isValidDecision('bad')).toBe(true)
    expect(isValidDecision('skip')).toBe(true)
    expect(isValidDecision('GOOD')).toBe(true)
    expect(isValidDecision('revert')).toBe(false)
    expect(isValidDecision('')).toBe(false)
  })
})
