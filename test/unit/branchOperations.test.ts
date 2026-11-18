import { describe, expect, it } from 'vitest'
import type { MergeBranchOutcome, RebaseBranchOutcome } from '~/types/repository'
import {
  validateGitBranchName,
  sanitizeBranchNameInput,
  shouldBlockBranchKeydown,
  handleBranchKeydownFilter,
  syncBranchInputElement,
} from '~/utils/branchValidation'

describe('branch operations models and contracts', () => {
  it('correctly validates clean merge branch outcome', () => {
    const cleanMerge: MergeBranchOutcome = {
      is_successful: true,
      has_conflicts: false,
      message: 'Fast-forward',
    }

    expect(cleanMerge.is_successful).toBe(true)
    expect(cleanMerge.has_conflicts).toBe(false)
  })

  it('correctly validates conflicted merge branch outcome', () => {
    const conflictedMerge: MergeBranchOutcome = {
      is_successful: false,
      has_conflicts: true,
      message: 'CONFLICT (content): Merge conflict in app.vue',
    }

    expect(conflictedMerge.is_successful).toBe(false)
    expect(conflictedMerge.has_conflicts).toBe(true)
    expect(conflictedMerge.message).toContain('CONFLICT')
  })

  it('correctly validates rebase branch outcomes', () => {
    const cleanRebase: RebaseBranchOutcome = {
      is_successful: true,
      has_conflicts: false,
      message: 'Successfully rebased and updated refs/heads/feature.',
    }

    const conflictedRebase: RebaseBranchOutcome = {
      is_successful: false,
      has_conflicts: true,
      message: 'Failed to merge in the changes.\nPatch failed at 0001 add button',
    }

    expect(cleanRebase.is_successful).toBe(true)
    expect(conflictedRebase.is_successful).toBe(false)
    expect(conflictedRebase.has_conflicts).toBe(true)
  })

  it('correctly validates branch name format and uniqueness', () => {
    const existing = ['main', 'develop', 'feature/login']

    // Valid branch names
    expect(validateGitBranchName('feature/login-v2', { currentBranchName: 'feature/login', existingBranchNames: existing }).isValid).toBe(true)
    expect(validateGitBranchName('bugfix_123', { existingBranchNames: existing }).isValid).toBe(true)
    expect(validateGitBranchName('v1.0.0-release', { existingBranchNames: existing }).isValid).toBe(true)

    // Uniqueness: already exists
    const duplicate = validateGitBranchName('develop', { existingBranchNames: existing })
    expect(duplicate.isValid).toBe(false)
    expect(duplicate.errorKey).toBe('branch.validationAlreadyExists')

    // Same as current
    const same = validateGitBranchName('feature/login', { currentBranchName: 'feature/login', existingBranchNames: existing })
    expect(same.isValid).toBe(false)
    expect(same.errorKey).toBe('branch.validationSameAsCurrent')

    // Empty
    expect(validateGitBranchName('   ').isValid).toBe(false)

    // Spaces
    const spaces = validateGitBranchName('feature new')
    expect(spaces.isValid).toBe(false)
    expect(spaces.errorKey).toBe('branch.validationNoSpaces')

    // Leading / trailing slashes or dashes
    expect(validateGitBranchName('/feature').isValid).toBe(false)
    expect(validateGitBranchName('feature/').isValid).toBe(false)
    expect(validateGitBranchName('-feature').isValid).toBe(false)
    expect(validateGitBranchName('feature-').isValid).toBe(false)
    expect(validateGitBranchName('.feature').isValid).toBe(false)

    // Consecutive separators
    expect(validateGitBranchName('feature//test').isValid).toBe(false)
    expect(validateGitBranchName('feature..test').isValid).toBe(false)

    // Lock file
    expect(validateGitBranchName('feature.lock').isValid).toBe(false)

    // Invalid chars
    expect(validateGitBranchName('feature~1').isValid).toBe(false)
    expect(validateGitBranchName('feature^2').isValid).toBe(false)
    expect(validateGitBranchName('feature:test').isValid).toBe(false)
  })

  it('correctly sanitizes pasted and typed branch names in JetBrains style', () => {
    // Empty input
    expect(sanitizeBranchNameInput('')).toBe('')

    // Whitespace to hyphen
    expect(sanitizeBranchNameInput('feature my branch')).toBe('feature-my-branch')
    expect(sanitizeBranchNameInput('feature   multiple    spaces')).toBe('feature-multiple-spaces')

    // Punctuation and forbidden symbols to hyphens
    expect(sanitizeBranchNameInput('feat: add user login & auth!')).toBe('feat-add-user-login-auth')
    expect(sanitizeBranchNameInput('fix(core): [UI] #42 *urgent*')).toBe('fix-core-UI-42-urgent')

    // Slash hierarchy preserved
    expect(sanitizeBranchNameInput('feature/awesome-thing')).toBe('feature/awesome-thing')
    expect(sanitizeBranchNameInput('feature///awesome-thing')).toBe('feature/awesome-thing')

    // Hyphens around slashes cleaned
    expect(sanitizeBranchNameInput('feature/-awesome')).toBe('feature/awesome')
    expect(sanitizeBranchNameInput('feature-/awesome')).toBe('feature/awesome')

    // Consecutive dots collapsed
    expect(sanitizeBranchNameInput('v1..0..0')).toBe('v1.0.0')

    // Leading dash, slash, dot removed
    expect(sanitizeBranchNameInput('--leading-dash')).toBe('leading-dash')
    expect(sanitizeBranchNameInput('/leading-slash')).toBe('leading-slash')
    expect(sanitizeBranchNameInput('..leading-dot')).toBe('leading-dot')

    // Trailing slash allowed during active typing
    expect(sanitizeBranchNameInput('feature/')).toBe('feature/')

    // Trailing space converted to hyphen to allow continuous typing
    expect(sanitizeBranchNameInput('feature ')).toBe('feature-')
  })

  it('correctly intercepts and blocks illegal branch keystrokes in real time', () => {
    const makeKeydownEvent = (key: string, options?: Partial<KeyboardEvent>): KeyboardEvent => ({
      key,
      ctrlKey: false,
      metaKey: false,
      altKey: false,
      shiftKey: false,
      preventDefault: () => {},
      ...options,
    } as KeyboardEvent)

    // 1. Consecutive dots: typing second dot after 'asd.' is BLOCKED
    expect(shouldBlockBranchKeydown(makeKeydownEvent('.'), 'asd.', 4, 4)).toBe(true)

    // First dot after 'asd' is ALLOWED
    expect(shouldBlockBranchKeydown(makeKeydownEvent('.'), 'asd', 3, 3)).toBe(false)

    // 2. Consecutive slashes: typing second slash after 'feature/' is BLOCKED
    expect(shouldBlockBranchKeydown(makeKeydownEvent('/'), 'feature/', 8, 8)).toBe(true)
    expect(shouldBlockBranchKeydown(makeKeydownEvent('/'), 'feature', 7, 7)).toBe(false)

    // 3. Consecutive hyphens: typing second hyphen after 'feature-' is BLOCKED
    expect(shouldBlockBranchKeydown(makeKeydownEvent('-'), 'feature-', 8, 8)).toBe(true)
    expect(shouldBlockBranchKeydown(makeKeydownEvent('-'), 'feature', 7, 7)).toBe(false)

    // 4. Leading separator: typing '.', '/', '-', or ' ' at index 0 is BLOCKED
    expect(shouldBlockBranchKeydown(makeKeydownEvent('.'), '', 0, 0)).toBe(true)
    expect(shouldBlockBranchKeydown(makeKeydownEvent('/'), '', 0, 0)).toBe(true)
    expect(shouldBlockBranchKeydown(makeKeydownEvent('-'), '', 0, 0)).toBe(true)
    expect(shouldBlockBranchKeydown(makeKeydownEvent(' '), '', 0, 0)).toBe(true)

    // 5. Illegal ref characters: BLOCKED immediately
    expect(shouldBlockBranchKeydown(makeKeydownEvent('~'), 'test', 4, 4)).toBe(true)
    expect(shouldBlockBranchKeydown(makeKeydownEvent('^'), 'test', 4, 4)).toBe(true)
    expect(shouldBlockBranchKeydown(makeKeydownEvent(':'), 'test', 4, 4)).toBe(true)
    expect(shouldBlockBranchKeydown(makeKeydownEvent('*'), 'test', 4, 4)).toBe(true)
    expect(shouldBlockBranchKeydown(makeKeydownEvent('?'), 'test', 4, 4)).toBe(true)
    expect(shouldBlockBranchKeydown(makeKeydownEvent('@'), 'test', 4, 4)).toBe(true)
    expect(shouldBlockBranchKeydown(makeKeydownEvent('!'), 'test', 4, 4)).toBe(true)

    // 6. Navigation and shortcuts: ALLOWED
    expect(shouldBlockBranchKeydown(makeKeydownEvent('Backspace'), 'asd.', 4, 4)).toBe(false)
    expect(shouldBlockBranchKeydown(makeKeydownEvent('ArrowLeft'), 'asd.', 4, 4)).toBe(false)
    expect(shouldBlockBranchKeydown(makeKeydownEvent('v', { ctrlKey: true }), 'asd.', 4, 4)).toBe(false)
    expect(shouldBlockBranchKeydown(makeKeydownEvent('a', { metaKey: true }), 'asd.', 4, 4)).toBe(false)

    // 7. Middle insertion adjacent to dot/slash: BLOCKED
    // Text is 'a.b', cursor at 1 (before '.') -> typing '.' would create '..b'
    expect(shouldBlockBranchKeydown(makeKeydownEvent('.'), 'a.b', 1, 1)).toBe(true)
  })

  it('correctly synchronizes HTMLInputElement DOM value and cursor', () => {
    const inputElement = document.createElement('input')
    inputElement.value = 'asd......w'
    inputElement.selectionStart = 10
    inputElement.selectionEnd = 10

    let capturedSanitized = ''
    const result = syncBranchInputElement(
      { target: inputElement } as unknown as Event,
      (sanitized) => {
        capturedSanitized = sanitized
      }
    )

    expect(result).toBe('asd.w')
    expect(capturedSanitized).toBe('asd.w')
    expect(inputElement.value).toBe('asd.w')
    expect(inputElement.selectionStart).toBe(5)
  })
})
