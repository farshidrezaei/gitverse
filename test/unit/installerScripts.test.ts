import { execSync } from 'node:child_process'
import { existsSync, readFileSync, statSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

describe('GitVerse Installer Scripts', () => {
  const rootDirectory = resolve(__dirname, '../..')
  const bashScriptPath = resolve(rootDirectory, 'scripts/install.sh')
  const powerShellScriptPath = resolve(rootDirectory, 'scripts/install.ps1')

  it('verifies Linux/macOS install.sh exists, is executable, and passes syntax check', () => {
    expect(existsSync(bashScriptPath)).toBe(true)

    const stats = statSync(bashScriptPath)
    // Check executable bit for user
    const isExecutable = (stats.mode & 0o111) !== 0
    expect(isExecutable).toBe(true)

    // Run bash -n syntax validation
    expect(() => {
      execSync(`bash -n "${bashScriptPath}"`, { stdio: 'pipe' })
    }).not.toThrow()
  })

  it('verifies install.sh contains all required command-line options and functions', () => {
    const bashScriptContent = readFileSync(bashScriptPath, 'utf8')

    expect(bashScriptContent).toContain('--version')
    expect(bashScriptContent).toContain('--force')
    expect(bashScriptContent).toContain('--reinstall')
    expect(bashScriptContent).toContain('--uninstall')
    expect(bashScriptContent).toContain('--help')

    expect(bashScriptContent).toContain('detect_operating_system')
    expect(bashScriptContent).toContain('detect_architecture')
    expect(bashScriptContent).toContain('compare_versions')
    expect(bashScriptContent).toContain('get_installed_version')
    expect(bashScriptContent).toContain('install_linux')
    expect(bashScriptContent).toContain('install_macos')
    expect(bashScriptContent).toContain('uninstall_gitverse')
  })

  it('verifies Windows install.ps1 exists and defines expected parameters and functions', () => {
    expect(existsSync(powerShellScriptPath)).toBe(true)

    const powerShellScriptContent = readFileSync(powerShellScriptPath, 'utf8')

    expect(powerShellScriptContent).toContain('[string]$Version')
    expect(powerShellScriptContent).toContain('[switch]$Force')
    expect(powerShellScriptContent).toContain('[switch]$Reinstall')
    expect(powerShellScriptContent).toContain('[switch]$Uninstall')

    expect(powerShellScriptContent).toContain('Get-InstalledGitVerseVersion')
    expect(powerShellScriptContent).toContain('Compare-ApplicationVersions')
    expect(powerShellScriptContent).toContain('Fetch-GitHubReleaseData')
    expect(powerShellScriptContent).toContain('Uninstall-GitVerseInstallation')
  })

  it('ensures installer scripts do not contain forbidden abbreviations', () => {
    const bashContent = readFileSync(bashScriptPath, 'utf8')
    const powerShellContent = readFileSync(powerShellScriptPath, 'utf8')

    const combinedContent = bashContent + '\n' + powerShellContent

    // Forbidden variable/argument names from AGENTS.md rule 2.1
    // repo, req, res, msg, auth, env (as variable names)
    expect(combinedContent).not.toMatch(/\b(\$|local\s+)repo\b/)
    expect(combinedContent).not.toMatch(/\b(\$|local\s+)req\b/)
    expect(combinedContent).not.toMatch(/\b(\$|local\s+)res\b/)
    expect(combinedContent).not.toMatch(/\b(\$|local\s+)msg\b/)
    expect(combinedContent).not.toMatch(/\b(\$|local\s+)auth\b/)
  })

  it('verifies jq release asset filtering logic parses patterns correctly without escape errors', () => {
    const mockReleaseJson = JSON.stringify({
      tag_name: 'v1.0.1',
      assets: [
        {
          name: 'GitVerse_1.0.1_amd64.AppImage.sig',
          browser_download_url: 'https://github.com/farshidrezaei/gitverse/releases/download/v1.0.1/GitVerse_1.0.1_amd64.AppImage.sig'
        },
        {
          name: 'GitVerse_1.0.1_amd64.AppImage',
          browser_download_url: 'https://github.com/farshidrezaei/gitverse/releases/download/v1.0.1/GitVerse_1.0.1_amd64.AppImage'
        },
        {
          name: 'GitVerse_1.0.1_universal.dmg',
          browser_download_url: 'https://github.com/farshidrezaei/gitverse/releases/download/v1.0.1/GitVerse_1.0.1_universal.dmg'
        }
      ]
    })

    const bashScriptContent = readFileSync(bashScriptPath, 'utf8')
    expect(bashScriptContent).toContain('downloaded_dmg')
    expect(bashScriptContent).not.toMatch(/curl[^\n]+-o\s+"\$\{download_dmg\}"/)

    // Execute the jq pattern filter command directly to ensure jq does not fail on escape characters
    const testCommand = `printf '%s' '${mockReleaseJson}' | jq -r --arg pattern '\\.AppImage$' '.assets[] | select((.name | test($pattern; "i")) and (.name | test("\\\\.sig$") | not)) | .browser_download_url'`
    const extractedUrl = execSync(testCommand, { shell: '/bin/bash' }).toString().trim()

    expect(extractedUrl).toBe('https://github.com/farshidrezaei/gitverse/releases/download/v1.0.1/GitVerse_1.0.1_amd64.AppImage')
  })
})
