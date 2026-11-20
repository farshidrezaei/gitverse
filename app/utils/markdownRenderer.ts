import { marked } from 'marked'
import hljs from 'highlight.js'

/**
 * Configure marked renderer for code syntax highlighting.
 */
const customRenderer = new marked.Renderer()

customRenderer.code = function ({ text, lang }: { text: string; lang?: string }): string {
  const language = lang && hljs.getLanguage(lang) ? lang : undefined
  let highlighted = ''

  if (language) {
    try {
      highlighted = hljs.highlight(text, { language, ignoreIllegals: true }).value
    } catch {
      highlighted = escapeHtml(text)
    }
  } else {
    try {
      highlighted = hljs.highlightAuto(text).value
    } catch {
      highlighted = escapeHtml(text)
    }
  }

  const languageClass = language ? ` hljs language-${escapeHtml(language)}` : ' hljs'
  return `<pre><code class="${languageClass}">${highlighted}</code></pre>`
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

/**
 * Strips script tags, iframe tags, and inline event handlers from HTML.
 */
function sanitizeHtmlSimple(html: string): string {
  return html
    // Remove script and iframe tags completely
    .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '')
    .replace(/<iframe\b[^<]*(?:(?!<\/iframe>)<[^<]*)*<\/iframe>/gi, '')
    .replace(/<object\b[^<]*(?:(?!<\/object>)<[^<]*)*<\/object>/gi, '')
    .replace(/<embed\b[^<]*(?:(?!<\/embed>)<[^<]*)*<\/embed>/gi, '')
    // Remove inline event handlers (onerror, onload, onclick, etc.)
    .replace(/\s+on[a-z]+=(["']).*?\1/gi, '')
    .replace(/\s+on[a-z]+=\S+/gi, '')
    // Neutralize javascript: pseudo-protocols
    .replace(/href=(["'])javascript:[^"']*\1/gi, 'href="#"')
    .replace(/src=(["'])javascript:[^"']*\1/gi, 'src=""')
}

/**
 * Parses markdown text into sanitized, safe HTML for display.
 */
export function renderSafeMarkdown(markdownContent: string | null | undefined): string {
  if (!markdownContent || !markdownContent.trim()) {
    return ''
  }

  try {
    const rawHtml = marked.parse(markdownContent, {
      gfm: true,
      breaks: true,
      async: false,
      renderer: customRenderer,
    }) as string

    return sanitizeHtmlSimple(rawHtml)
  } catch {
    return markdownContent
  }
}

