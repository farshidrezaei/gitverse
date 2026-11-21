import { describe, it, expect } from 'vitest'
import fs from 'node:fs'
import path from 'node:path'

describe('ExplorerPullRequestsSection Architectural & Quality Standards', () => {
  const componentPath = path.resolve(
    process.cwd(),
    'app/components/workspace/explorer/ExplorerPullRequestsSection.vue',
  )

  it('component exists and uses USkeleton for loading', () => {
    expect(fs.existsSync(componentPath)).toBe(true)
    const content = fs.readFileSync(componentPath, 'utf8')
    expect(content).toContain('USkeleton')
    expect(content).not.toContain('animate-spin')
  })

  it('adheres to strict naming with zero forbidden abbreviations', () => {
    const content = fs.readFileSync(componentPath, 'utf8')
    const forbidden = [
      /\bcfg\b/,
      /\bctx\b/,
      /\bmgr\b/,
      /\bsvc\b/,
      /\breq\b/,
      /\bres\b/,
      /\bmsg\b/,
      /\busr\b/,
      /\bval\b/,
    ]
    for (const pattern of forbidden) {
      expect(pattern.test(content)).toBe(false)
    }
  })

  it('defines openPullRequest and checkoutBranch emits', () => {
    const content = fs.readFileSync(componentPath, 'utf8')
    expect(content).toContain('openPullRequest:')
    expect(content).toContain('checkoutBranch:')
  })

  it('uses Nuxt UI components first', () => {
    const content = fs.readFileSync(componentPath, 'utf8')
    expect(content).toContain('<UButton')
    expect(content).toContain('<UInput')
    expect(content).toContain('<UIcon')
    expect(content).toContain('<USkeleton')
  })
})
