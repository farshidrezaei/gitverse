import { describe, expect, it } from 'vitest'
import fs from 'node:fs'
import path from 'node:path'

describe('WorkspaceUnifiedExplorer Architectural Standard', () => {
  const explorerDirectory = path.resolve(__dirname, '../../app/components/workspace/explorer')

  it('contains all required modular section components', () => {
    const expectedComponents = [
      'WorkspaceUnifiedExplorer.vue',
      'ExplorerChangesSection.vue',
      'ExplorerBranchesSection.vue',
      'ExplorerStashesSection.vue',
      'ExplorerRemotesSection.vue',
      'ExplorerTagsSection.vue',
      'ExplorerWorktreesSection.vue',
    ]

    for (const componentName of expectedComponents) {
      const componentPath = path.join(explorerDirectory, componentName)
      expect(fs.existsSync(componentPath)).toBe(true)
    }
  })

  it('guarantees zero generic loading spinners or text in explorer components', () => {
    const entries = fs.readdirSync(explorerDirectory)
    for (const entry of entries) {
      if (entry.endsWith('.vue')) {
        const fileContent = fs.readFileSync(path.join(explorerDirectory, entry), 'utf8')
        expect(fileContent).not.toContain("t('common.loading')")
      }
    }
  })

  it('WorkspaceUnifiedExplorer uses proper props and emits without banned abbreviations', () => {
    const fileContent = fs.readFileSync(
      path.join(explorerDirectory, 'WorkspaceUnifiedExplorer.vue'),
      'utf8',
    )
    expect(fileContent).toContain('repositoryStatus')
    expect(fileContent).toContain('references')
    expect(fileContent).toContain('favoriteBranches')
    expect(fileContent).toContain('selectedBranchName')
    expect(fileContent).toContain('stagingViewMode')
    // No banned abbreviations
    expect(fileContent).not.toMatch(/\bcfg\b/)
    expect(fileContent).not.toMatch(/\bctx\b/)
    expect(fileContent).not.toMatch(/\bmgr\b/)
    expect(fileContent).not.toMatch(/\bsvc\b/)
    expect(fileContent).not.toMatch(/\brepo\b/)
  })
})
