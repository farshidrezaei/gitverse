import { marked } from 'marked'

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
    }) as string

    return sanitizeHtmlSimple(rawHtml)
  } catch {
    return markdownContent
  }
}
