/**
 * Formats a UNIX timestamp (in seconds) into a readable relative time string.
 */
export function formatRelativeTime(timestampSeconds: number): string {
  const now = Math.floor(Date.now() / 1000)
  const diffSeconds = Math.max(0, now - timestampSeconds)

  if (diffSeconds < 60) {
    return 'just now'
  }
  const minutes = Math.floor(diffSeconds / 60)
  if (minutes < 60) {
    return `${minutes}m ago`
  }
  const hours = Math.floor(minutes / 60)
  if (hours < 24) {
    return `${hours}h ago`
  }
  const days = Math.floor(hours / 24)
  if (days < 30) {
    return `${days}d ago`
  }
  const months = Math.floor(days / 30)
  if (months < 12) {
    return `${months}mo ago`
  }
  const years = Math.floor(days / 365)
  return `${years}y ago`
}

/**
 * Formats a UNIX timestamp (in milliseconds) into a localized time string (HH:MM:SS).
 */
export function formatTime(timestampMilliseconds: number): string {
  const date = new Date(timestampMilliseconds)
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
}
