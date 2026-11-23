import { describe, expect, it } from 'vitest'
import {
  buildReleaseBody,
  extractChangelogSection,
} from '../../scripts/publish-github-release.mjs'

describe('GitHub Release Publisher', () => {
  const sampleChangelog = `
# Changelog

## [1.0.0] - 2026-11-21

### Added
- **Feature 1**: First feature description.

## [0.9.0] - 2026-11-10

### Added
- **Old Feature**: Old description.
`

  it('extracts changelog section for target version', () => {
    const section = extractChangelogSection(sampleChangelog, '1.0.0')
    expect(section).toContain('### Added')
    expect(section).toContain('Feature 1')
    expect(section).not.toContain('Old Feature')
    expect(section).not.toContain('## [0.9.0]')
  })

  it('builds release body with changelog and official installer table', () => {
    const body = buildReleaseBody('1.0.0', '### Added\n- **Feature 1**: First feature.')
    expect(body).toContain('## What is Changed in GitVerse v1.0.0')
    expect(body).toContain('Feature 1')
    expect(body).toContain('### Official Desktop Installers')
    expect(body).toContain('.AppImage')
    expect(body).toContain('.dmg')
    expect(body).toContain('.exe')
    expect(body).toContain('SHA256SUMS.txt')
  })
})
