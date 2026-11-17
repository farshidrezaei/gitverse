import type { ConfirmationTier } from '~/types/confirmation'

/**
 * Validates whether user-entered input strictly matches the required confirmation keyword.
 */
export function validateConfirmationKeyword(
  userInput: string,
  requiredKeyword: string,
  isCaseSensitive = true
): boolean {
  const sanitizedInput = userInput.trim()
  const sanitizedKeyword = requiredKeyword.trim()

  if (!sanitizedKeyword) {
    return true
  }

  if (isCaseSensitive) {
    return sanitizedInput === sanitizedKeyword
  }

  return sanitizedInput.toLowerCase() === sanitizedKeyword.toLowerCase()
}

/**
 * Resolves remaining whole seconds from progress and duration.
 */
export function calculateRemainingSeconds(
  elapsedMilliseconds: number,
  totalDurationMilliseconds: number
): number {
  const remainingMilliseconds = Math.max(0, totalDurationMilliseconds - elapsedMilliseconds)
  return Math.max(1, Math.ceil(remainingMilliseconds / 1000))
}

/**
 * Checks whether a confirmation tier introduces non-trivial friction.
 */
export function hasElevatedConfirmationRisk(tier: ConfirmationTier): boolean {
  return tier === 'hold' || tier === 'keyword'
}
