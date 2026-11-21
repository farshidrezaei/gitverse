import { describe, expect, it } from 'vitest'
import fs from 'node:fs'
import path from 'node:path'

describe('DesktopAppBarRepositoryTabs Architectural Standard', () => {
  const componentPath = path.resolve(
    __dirname,
    '../../app/components/layout/DesktopAppBarRepositoryTabs.vue',
  )

  it('DesktopAppBarRepositoryTabs component exists', () => {
    expect(fs.existsSync(componentPath)).toBe(true)
  })

  it('guarantees zero generic loading spinners or text in DesktopAppBarRepositoryTabs', () => {
    const fileContent = fs.readFileSync(componentPath, 'utf8')
    expect(fileContent).not.toContain("t('common.loading')")
  })

  it('DesktopAppBarRepositoryTabs uses strict naming without banned abbreviations', () => {
    const fileContent = fs.readFileSync(componentPath, 'utf8')
    expect(fileContent).toContain('repositoryTabs')
    expect(fileContent).toContain('activeRepositoryPath')
    expect(fileContent).toContain('handleSelectRepository')
    expect(fileContent).toContain('handleCloseRepository')

    // No banned abbreviations
    expect(fileContent).not.toMatch(/\bcfg\b/)
    expect(fileContent).not.toMatch(/\bctx\b/)
    expect(fileContent).not.toMatch(/\bmgr\b/)
    expect(fileContent).not.toMatch(/\bsvc\b/)
    expect(fileContent).not.toMatch(/\brepo\b/)
  })
})
