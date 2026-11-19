import { describe, it, expect } from 'vitest'
import { renderSafeMarkdown } from '~/utils/markdownRenderer'

describe('Markdown Renderer Unit Test Suite', () => {
  it('returns empty string for empty input', () => {
    expect(renderSafeMarkdown('')).toBe('')
    expect(renderSafeMarkdown(null)).toBe('')
    expect(renderSafeMarkdown(undefined)).toBe('')
    expect(renderSafeMarkdown('   ')).toBe('')
  })

  it('renders standard markdown elements correctly', () => {
    const markdown = '# Pull Request Title\n\nThis is a description with **bold** text and `code`.'
    const rendered = renderSafeMarkdown(markdown)

    expect(rendered).toContain('<h1>Pull Request Title</h1>')
    expect(rendered).toContain('<strong>bold</strong>')
    expect(rendered).toContain('<code>code</code>')
  })

  it('renders task lists and bullet points correctly', () => {
    const markdown = '- [x] Completed task\n- [ ] Pending task'
    const rendered = renderSafeMarkdown(markdown)

    expect(rendered).toContain('<li>')
    expect(rendered).toContain('Completed task')
  })

  it('sanitizes dangerous scripts and xss vectors', () => {
    const malicious = 'Description <script>alert("hack")</script><img src="x" onerror="alert(1)">'
    const sanitized = renderSafeMarkdown(malicious)

    expect(sanitized).not.toContain('<script>')
    expect(sanitized).not.toContain('onerror')
  })
})
