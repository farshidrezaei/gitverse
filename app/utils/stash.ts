/**
 * Extracts numeric index from a stash identifier string (e.g. "stash@{2}" -> 2).
 */
export function parseStashIndex(stashIdentifier: string): number {
  const match = stashIdentifier.match(/stash@\{(\d+)\}/)
  return match && match[1] ? parseInt(match[1], 10) : 0
}

/**
 * Formats a stash subject by stripping internal Git prefixes like "On master: " or "WIP on master: ".
 */
export function formatStashDisplayMessage(rawMessage: string): string {
  const trimmed = rawMessage.trim()
  const wipPrefixMatch = trimmed.match(/^(?:WIP on [^:]+:\s*(?:[0-9a-f]+\s+)?|On [^:]+:\s*)(.*)$/i)
  if (wipPrefixMatch && wipPrefixMatch[1]) {
    const clean = wipPrefixMatch[1].trim()
    return clean || trimmed
  }
  return trimmed
}

/**
 * Formats relative time from a timestamp in seconds.
 */
export function formatStashRelativeTime(
  timestampSeconds: number,
  currentTimestampSeconds = Math.floor(Date.now() / 1000)
): string {
  const differenceSeconds = Math.max(0, currentTimestampSeconds - timestampSeconds)
  if (differenceSeconds < 60) {
    return `${differenceSeconds}s ago`
  }
  const minutes = Math.floor(differenceSeconds / 60)
  if (minutes < 60) {
    return `${minutes}m ago`
  }
  const hours = Math.floor(minutes / 60)
  if (hours < 24) {
    return `${hours}h ago`
  }
  const days = Math.floor(hours / 24)
  return `${days}d ago`
}
