/**
 * Counts number of unresolved conflict sections within a file.
 */
export function countConflictMarkers(content: string): number {
  if (!content) return 0
  const matches = content.match(/^<{7}\s/gm)
  return matches ? matches.length : 0
}

/**
 * Checks if a string still contains Git conflict markers.
 */
export function hasConflictMarkers(content: string): boolean {
  if (!content) return false
  return /^<{7}\s/m.test(content) && /^={7}$/m.test(content) && /^>{7}\s/m.test(content)
}

/**
 * Combines both Ours and Theirs content sequentially with a newline separation.
 */
export function combineOursAndTheirs(oursContent: string, theirsContent: string): string {
  const normalizedOurs = oursContent.trimEnd()
  const normalizedTheirs = theirsContent.trimStart()
  if (!normalizedOurs) return normalizedTheirs
  if (!normalizedTheirs) return normalizedOurs
  return `${normalizedOurs}\n${normalizedTheirs}`
}
