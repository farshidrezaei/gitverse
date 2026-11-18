export interface BranchValidationResult {
  isValid: boolean
  errorKey?: string
}

/**
 * Validates a Git branch name according to git-check-ref-format specification
 * and uniqueness constraints against existing branches in the repository.
 */
export function validateGitBranchName(
  candidateName: string,
  options?: {
    currentBranchName?: string
    existingBranchNames?: string[]
  }
): BranchValidationResult {
  const trimmed = candidateName.trim()

  if (!trimmed) {
    return { isValid: false, errorKey: 'branch.validationEmpty' }
  }

  if (options?.currentBranchName && trimmed === options.currentBranchName) {
    return { isValid: false, errorKey: 'branch.validationSameAsCurrent' }
  }

  if (options?.existingBranchNames) {
    const isDuplicate = options.existingBranchNames.some(
      (existing) =>
        existing.toLowerCase() === trimmed.toLowerCase() &&
        existing !== options.currentBranchName
    )
    if (isDuplicate) {
      return { isValid: false, errorKey: 'branch.validationAlreadyExists' }
    }
  }

  // 1. Cannot contain whitespace or control characters
  if (/\s/.test(trimmed)) {
    return { isValid: false, errorKey: 'branch.validationNoSpaces' }
  }

  // 2. Cannot start or end with a slash, dash, or dot
  if (trimmed.startsWith('/') || trimmed.endsWith('/')) {
    return { isValid: false, errorKey: 'branch.validationNoLeadingTrailingSlash' }
  }
  if (trimmed.startsWith('-') || trimmed.endsWith('-')) {
    return { isValid: false, errorKey: 'branch.validationNoLeadingTrailingDash' }
  }
  if (trimmed.startsWith('.') || trimmed.endsWith('.')) {
    return { isValid: false, errorKey: 'branch.validationNoLeadingTrailingDot' }
  }

  // 3. Cannot contain consecutive slashes '//' or consecutive dots '..'
  if (trimmed.includes('//') || trimmed.includes('..')) {
    return { isValid: false, errorKey: 'branch.validationNoConsecutiveSeparators' }
  }

  // 4. Cannot end with .lock (Git lock file reservation)
  if (trimmed.endsWith('.lock')) {
    return { isValid: false, errorKey: 'branch.validationCannotEndWithLock' }
  }

  // 5. Cannot contain '@{' sequence or special ref characters
  if (trimmed.includes('@{')) {
    return { isValid: false, errorKey: 'branch.validationInvalidCharacters' }
  }

  // 6. Only letters, numbers, hyphens, underscores, slashes, and dots allowed
  if (!/^[a-zA-Z0-9._\-/]+$/.test(trimmed)) {
    return { isValid: false, errorKey: 'branch.validationInvalidCharacters' }
  }

  return { isValid: true }
}

/**
 * Automatically sanitizes raw input or pasted text into a compliant Git branch slug.
 * Transforms whitespace and forbidden characters into hyphens, collapses consecutive separators,
 * and preserves valid branch hierarchy slashes (JetBrains-style smart branch input).
 */
export function sanitizeBranchNameInput(rawInput: string): string {
  if (!rawInput) {
    return ''
  }

  const hasTrailingWhitespace = /\s$/.test(rawInput)

  // 1. Convert whitespace and any non-alphanumeric/non-allowed punctuation into hyphens
  let sanitized = rawInput.replace(/[^a-zA-Z0-9._\-/]+/g, '-')

  // 2. Collapse consecutive slashes into single slash
  sanitized = sanitized.replace(/\/+/g, '/')

  // 3. Collapse consecutive dots into single dot
  sanitized = sanitized.replace(/\.+/g, '.')

  // 4. Collapse consecutive hyphens into single hyphen
  sanitized = sanitized.replace(/-+/g, '-')

  // 5. Clean up hyphens around slashes (e.g. "feat-/name" -> "feat/name")
  sanitized = sanitized.replace(/-+\//g, '/')
  sanitized = sanitized.replace(/\/-+/g, '/')

  // 6. Remove leading dash, slash, or dot
  sanitized = sanitized.replace(/^[-/.]+/, '')

  // 7. Remove trailing dash unless the user specifically typed a trailing space to separate words
  if (!hasTrailingWhitespace) {
    sanitized = sanitized.replace(/-+$/, '')
  }

  return sanitized
}
