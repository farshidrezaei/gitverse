import { describe, expect, it } from 'vitest'
import {
  buildTelegramReleaseMessage,
  extractChangelogForVersion,
  formatMarkdownToTelegramHtml,
} from '../../scripts/send-telegram-release.mjs'

describe('Telegram Release Notifier', () => {
  const sampleChangelog = `
# Changelog

## [1.1.0] - 2026-11-23

### Added
- **Feature A**: Description with \`<tag>\` and & symbol and [link](https://gitverse.dev).
- **Feature B**: Another item.

### Fixed
- **Bug 1**: Fixed issue in \`domain/mod.rs\`.

## [1.0.0] - 2026-11-22

### Added
- **Old Feature**: Initial baseline.
`

  it('extracts changelog specifically for target version', () => {
    const extracted = extractChangelogForVersion(sampleChangelog, '1.1.0')
    expect(extracted).toContain('### Added')
    expect(extracted).toContain('Feature A')
    expect(extracted).toContain('Bug 1')
    expect(extracted).not.toContain('Old Feature')
    expect(extracted).not.toContain('## [1.0.0]')
  })

  it('formats markdown to telegram-safe HTML with escaping', () => {
    const formatted = formatMarkdownToTelegramHtml(`### Added
- **Feature A**: Description with \`<tag>\` and & symbol and [doc link](https://gitverse.dev).
### Fixed
- **Bug 1**: Fixed issue in \`domain/mod.rs\`.`)

    expect(formatted).toContain('✨ <b>قابلیت‌های جدید (Added):</b>')
    expect(formatted).toContain('🐛 <b>رفع باگ‌ها (Fixed):</b>')
    expect(formatted).toContain(
      '• <b>Feature A</b>: Description with <code>&lt;tag&gt;</code> and &amp; symbol and <a href="https://gitverse.dev">doc link</a>.',
    )
    expect(formatted).toContain('• <b>Bug 1</b>: Fixed issue in <code>domain/mod.rs</code>.')
  })

  it('builds complete message within telegram character limit', () => {
    const message = buildTelegramReleaseMessage({
      version: '1.1.0',
      releaseUrl: 'https://github.com/farshidrezaei/gitverse/releases/tag/v1.1.0',
      changelogMarkdown: `- **Feature**: Test item`,
    })

    expect(message).toContain('🚀 <b>GitVerse v1.1.0 منتشر شد!</b>')
    expect(message).toContain('https://github.com/farshidrezaei/gitverse/releases/tag/v1.1.0')
    expect(message).toContain('.AppImage')
    expect(message).toContain('.dmg')
    expect(message).toContain('.exe')
    expect(message.length).toBeLessThan(4096)
  })

  it('truncates excessively long changelogs gracefully', () => {
    const hugeMarkdown = Array.from({ length: 200 })
      .map((_, index) => `- **Item ${index}**: Long descriptive changelog line that takes up space.`)
      .join('\n')

    const message = buildTelegramReleaseMessage({
      version: '1.1.0',
      releaseUrl: 'https://github.com/farshidrezaei/gitverse/releases/tag/v1.1.0',
      changelogMarkdown: hugeMarkdown,
    })

    expect(message.length).toBeLessThan(4096)
    expect(message).toContain('مشاهده متن کامل در گیت‌هاب')
  })
})
