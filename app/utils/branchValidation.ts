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

/**
 * Determines whether a keyboard event should be blocked to prevent invalid Git branch characters
 * or disallowed sequences (such as consecutive dots, leading dashes/slashes, or illegal symbols).
 */
export function shouldBlockBranchKeydown(
  event: KeyboardEvent,
  currentValue: string,
  selectionStart: number | null,
  selectionEnd: number | null = null
): boolean {
  // Allow browser/system shortcut keys (Ctrl+A, Ctrl+C, Ctrl+V, Cmd+V, etc.)
  if (event.ctrlKey || event.metaKey || event.altKey) {
    return false
  }

  // Allow navigation, deletion, and submission keys (Backspace, Delete, Arrows, Tab, Enter, etc.)
  if (event.key.length !== 1) {
    return false
  }

  const key = event.key
  const isReplacingSelection = selectionStart !== null && selectionEnd !== null && selectionStart !== selectionEnd
  const cursorIndex = selectionStart ?? currentValue.length

  // Disallow leading hyphen, slash, dot, or space at the beginning of the branch name
  if (cursorIndex === 0 && !isReplacingSelection && (key === '-' || key === '/' || key === '.' || key === ' ')) {
    return true
  }

  // Check character immediately before cursor (if not replacing selection from beginning)
  const previousCharacter = cursorIndex > 0 ? (currentValue[cursorIndex - 1] ?? '') : ''

  // Check character immediately after cursor (if caret is collapsed)
  const nextCharacter = !isReplacingSelection && cursorIndex < currentValue.length
    ? (currentValue[cursorIndex] ?? '')
    : ''

  // 1. Disallow consecutive dots (..), slashes (//), or hyphens (--)
  if (key === '.' && (previousCharacter === '.' || nextCharacter === '.')) {
    return true
  }
  if (key === '/' && (previousCharacter === '/' || nextCharacter === '/')) {
    return true
  }
  if (key === '-' && (previousCharacter === '-' || nextCharacter === '-')) {
    return true
  }

  // 2. Disallow illegal combinations around slashes (/- or -/)
  if (key === '-' && (previousCharacter === '/' || nextCharacter === '/')) {
    return true
  }
  if (key === '/' && (previousCharacter === '-' || nextCharacter === '-')) {
    return true
  }
  if (key === '.' && (previousCharacter === '/' || nextCharacter === '/')) {
    return true
  }
  if (key === '/' && (previousCharacter === '.' || nextCharacter === '.')) {
    return true
  }

  // 3. Space transforms into hyphen, so disallow space immediately adjacent to hyphen, slash, or dot
  if (key === ' ' && (previousCharacter === '-' || previousCharacter === '/' || previousCharacter === '.')) {
    return true
  }

  // 4. Disallow characters outside ASCII alphanumerics, dot, underscore, hyphen, slash, and space
  const isAllowedCharacter = /^[a-zA-Z0-9._\-/ ]$/.test(key)
  if (!isAllowedCharacter) {
    return true
  }

  return false
}

/**
 * Keydown event handler that prevents illegal character entry at the hardware/keyboard level.
 */
export function handleBranchKeydownFilter(event: KeyboardEvent, currentValue: string): void {
  const target = event.target as HTMLInputElement | null
  const selectionStart = target ? target.selectionStart : null
  const selectionEnd = target ? target.selectionEnd : null

  if (shouldBlockBranchKeydown(event, currentValue, selectionStart, selectionEnd)) {
    event.preventDefault()
  }
}

/**
 * Synchronizes the native HTMLInputElement DOM value immediately with the sanitized slug,
 * preserving caret position and preventing desync when Vue skips reactive DOM re-renders.
 */
export function syncBranchInputElement(
  event: Event,
  onSanitized?: (sanitizedValue: string) => void
): string {
  const target = event.target as HTMLInputElement | null
  if (!target) {
    return ''
  }

  const rawValue = target.value
  const sanitized = sanitizeBranchNameInput(rawValue)

  if (target.value !== sanitized) {
    const cursorPosition = target.selectionStart
    target.value = sanitized
    if (cursorPosition !== null) {
      const adjustedPosition = Math.min(cursorPosition, sanitized.length)
      target.setSelectionRange(adjustedPosition, adjustedPosition)
    }
  }

  if (onSanitized) {
    onSanitized(sanitized)
  }

  return sanitized
}
