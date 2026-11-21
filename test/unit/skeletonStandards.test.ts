import { describe, expect, it } from 'vitest'
import fs from 'node:fs'
import path from 'node:path'

describe('Zero-CLS Skeleton Loading Standard', () => {
  const componentsDirectory = path.resolve(__dirname, '../../app/components')

  it('prohibits generic t("common.loading") text in component templates', () => {
    function scanDirectory(directory: string): string[] {
      const violations: string[] = []
      const entries = fs.readdirSync(directory, { withFileTypes: true })

      for (const entry of entries) {
        const fullPath = path.join(directory, entry.name)
        if (entry.isDirectory()) {
          violations.push(...scanDirectory(fullPath))
        } else if (entry.isFile() && entry.name.endsWith('.vue')) {
          const content = fs.readFileSync(fullPath, 'utf8')
          if (content.includes("t('common.loading')")) {
            violations.push(path.relative(componentsDirectory, fullPath))
          }
        }
      }
      return violations
    }

    const violations = scanDirectory(componentsDirectory)
    expect(violations).toEqual([])
  })

  it('enforces USkeleton in ExplorerPullRequestsSection for loading state', () => {
    const fileContent = fs.readFileSync(
      path.join(componentsDirectory, 'workspace/explorer/ExplorerPullRequestsSection.vue'),
      'utf8',
    )
    expect(fileContent).toContain('USkeleton')
    expect(fileContent).not.toContain("t('common.loading')")
  })

  it('enforces USkeleton in ProviderAccountsCard for loading state', () => {
    const fileContent = fs.readFileSync(
      path.join(componentsDirectory, 'settings/ProviderAccountsCard.vue'),
      'utf8',
    )
    expect(fileContent).toContain('USkeleton')
    expect(fileContent).toContain('provider-skeleton')
    expect(fileContent).not.toContain("t('common.loading')")
  })

  it('enforces USkeleton in FolderPickerModal for folders list', () => {
    const fileContent = fs.readFileSync(
      path.join(componentsDirectory, 'workspace/FolderPickerModal.vue'),
      'utf8',
    )
    expect(fileContent).toContain('USkeleton')
    expect(fileContent).toContain('folder-skeleton')
  })

  it('enforces USkeleton in InteractiveRebaseModal for commits list', () => {
    const fileContent = fs.readFileSync(
      path.join(componentsDirectory, 'workspace/commit/InteractiveRebaseModal.vue'),
      'utf8',
    )
    expect(fileContent).toContain('USkeleton')
    expect(fileContent).toContain('rebase-skeleton')
  })
})
