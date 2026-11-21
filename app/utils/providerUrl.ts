export interface ParsedRepositoryLocation {
  host: string
  fullName: string
}

/**
 * Parses Git remote URL (SCP SSH or HTTPS syntax) into host and owner/repository name.
 */
export function parseRepositoryFromUrl(url: string): ParsedRepositoryLocation | null {
  const trimmed = url.trim()

  // Match git@host:owner/repo.git
  const scpMatch = trimmed.match(/^[a-zA-Z0-9._-]+@([^:]+):([^\s/]+)\/([^\s/]+?)(?:\.git)?$/)
  if (scpMatch && scpMatch[1] && scpMatch[2] && scpMatch[3]) {
    return {
      host: scpMatch[1].toLowerCase(),
      fullName: `${scpMatch[2]}/${scpMatch[3]}`,
    }
  }

  // Match https://host/owner/repo.git
  const urlMatch = trimmed.match(/^https?:\/\/([^/]+)\/([^\s/]+)\/([^\s/]+?)(?:\.git)?$/)
  if (urlMatch && urlMatch[1] && urlMatch[2] && urlMatch[3]) {
    return {
      host: urlMatch[1].toLowerCase(),
      fullName: `${urlMatch[2]}/${urlMatch[3]}`,
    }
  }

  return null
}
