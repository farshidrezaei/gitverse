import type { DiffHunk, DiffLine, DiffLineType } from '~/types/repository'

export interface DiffTextSpan {
  text: string
  isChanged: boolean
}

export interface SideBySideLineCell {
  lineNumber: number | null
  content: string
  lineType: DiffLineType | 'empty'
  spans?: DiffTextSpan[]
  originalLineIndex?: number
}

export interface SideBySideRow {
  id: string
  left: SideBySideLineCell
  right: SideBySideLineCell
}

export interface SideBySideHunk {
  header: string
  rows: SideBySideRow[]
}

const sideBySideHunkCache = new WeakMap<DiffHunk, SideBySideHunk>()

/**
 * Computes intra-line character/token differences between old and new line content.
 * Identifies common prefix and suffix to isolate changed substrings.
 */
export function computeIntraLineDiff(
  oldContent: string,
  newContent: string
): { oldSpans: DiffTextSpan[]; newSpans: DiffTextSpan[] } {
  if (oldContent === newContent) {
    return {
      oldSpans: [{ text: oldContent, isChanged: false }],
      newSpans: [{ text: newContent, isChanged: false }],
    }
  }

  let prefixLength = 0
  const maxPrefix = Math.min(oldContent.length, newContent.length)
  while (prefixLength < maxPrefix && oldContent[prefixLength] === newContent[prefixLength]) {
    prefixLength += 1
  }

  let suffixLength = 0
  const maxSuffix = Math.min(oldContent.length - prefixLength, newContent.length - prefixLength)
  while (
    suffixLength < maxSuffix &&
    oldContent[oldContent.length - 1 - suffixLength] === newContent[newContent.length - 1 - suffixLength]
  ) {
    suffixLength += 1
  }

  const buildSpans = (text: string): DiffTextSpan[] => {
    const spans: DiffTextSpan[] = []
    const prefix = text.slice(0, prefixLength)
    const middle = text.slice(prefixLength, text.length - suffixLength)
    const suffix = suffixLength > 0 ? text.slice(text.length - suffixLength) : ''

    if (prefix) {
      spans.push({ text: prefix, isChanged: false })
    }
    if (middle) {
      spans.push({ text: middle, isChanged: true })
    }
    if (suffix) {
      spans.push({ text: suffix, isChanged: false })
    }
    return spans
  }

  return {
    oldSpans: buildSpans(oldContent),
    newSpans: buildSpans(newContent),
  }
}

/**
 * Transforms unified hunk lines into aligned side-by-side rows.
 * Aligns deletion lines on the left with replacement addition lines on the right.
 * Distinguishes modified lines (blue with word diff) from pure additions (green) and deletions (red).
 * Uses WeakMap memoization to avoid redundant computations across renders.
 */
export function buildSideBySideHunk(hunk: DiffHunk): SideBySideHunk {
  const cached = sideBySideHunkCache.get(hunk)
  if (cached) {
    return cached
  }

  const rows: SideBySideRow[] = []
  let rowIndex = 0

  const lines = hunk.lines
  let lineIndex = 0

  while (lineIndex < lines.length) {
    const currentLine = lines[lineIndex]
    if (!currentLine) {
      lineIndex += 1
      continue
    }

    if (currentLine.line_type === 'context' || currentLine.line_type === 'header') {
      const cleanContent = currentLine.content.startsWith(' ')
        ? currentLine.content.slice(1)
        : currentLine.content

      rows.push({
        id: `row-${rowIndex++}`,
        left: {
          lineNumber: currentLine.old_line_number,
          content: cleanContent,
          lineType: 'context',
          spans: [{ text: cleanContent, isChanged: false }],
          originalLineIndex: lineIndex,
        },
        right: {
          lineNumber: currentLine.new_line_number,
          content: cleanContent,
          lineType: 'context',
          spans: [{ text: cleanContent, isChanged: false }],
          originalLineIndex: lineIndex,
        },
      })
      lineIndex += 1
      continue
    }

    // Accumulate consecutive deletions and additions with their original hunk line indices
    interface IndexedLine {
      line: DiffLine
      originalLineIndex: number
    }
    const deletions: IndexedLine[] = []
    const additions: IndexedLine[] = []

    while (lineIndex < lines.length && lines[lineIndex]?.line_type === 'deletion') {
      const line = lines[lineIndex]
      if (line) deletions.push({ line, originalLineIndex: lineIndex })
      lineIndex += 1
    }

    while (lineIndex < lines.length && lines[lineIndex]?.line_type === 'addition') {
      const line = lines[lineIndex]
      if (line) additions.push({ line, originalLineIndex: lineIndex })
      lineIndex += 1
    }

    const maxCount = Math.max(deletions.length, additions.length)
    for (let index = 0; index < maxCount; index += 1) {
      const deletionEntry = deletions[index]
      const additionEntry = additions[index]

      if (deletionEntry && additionEntry) {
        // Paired line: this is an in-place modification (Blue 3-color mode)
        const deletion = deletionEntry.line
        const addition = additionEntry.line
        const cleanOldContent = deletion.content.startsWith('-')
          ? deletion.content.slice(1)
          : deletion.content
        const cleanNewContent = addition.content.startsWith('+')
          ? addition.content.slice(1)
          : addition.content

        const { oldSpans, newSpans } = computeIntraLineDiff(cleanOldContent, cleanNewContent)

        rows.push({
          id: `row-${rowIndex++}`,
          left: {
            lineNumber: deletion.old_line_number,
            content: cleanOldContent,
            lineType: 'modification',
            spans: oldSpans,
            originalLineIndex: deletionEntry.originalLineIndex,
          },
          right: {
            lineNumber: addition.new_line_number,
            content: cleanNewContent,
            lineType: 'modification',
            spans: newSpans,
            originalLineIndex: additionEntry.originalLineIndex,
          },
        })
      } else if (deletionEntry) {
        // Pure deletion (Red)
        const deletion = deletionEntry.line
        const cleanOldContent = deletion.content.startsWith('-')
          ? deletion.content.slice(1)
          : deletion.content

        rows.push({
          id: `row-${rowIndex++}`,
          left: {
            lineNumber: deletion.old_line_number,
            content: cleanOldContent,
            lineType: 'deletion',
            spans: [{ text: cleanOldContent, isChanged: true }],
            originalLineIndex: deletionEntry.originalLineIndex,
          },
          right: {
            lineNumber: null,
            content: '',
            lineType: 'empty',
          },
        })
      } else if (additionEntry) {
        // Pure addition (Green)
        const addition = additionEntry.line
        const cleanNewContent = addition.content.startsWith('+')
          ? addition.content.slice(1)
          : addition.content

        rows.push({
          id: `row-${rowIndex++}`,
          left: {
            lineNumber: null,
            content: '',
            lineType: 'empty',
          },
          right: {
            lineNumber: addition.new_line_number,
            content: cleanNewContent,
            lineType: 'addition',
            spans: [{ text: cleanNewContent, isChanged: true }],
            originalLineIndex: additionEntry.originalLineIndex,
          },
        })
      }
    }
  }

  const result: SideBySideHunk = {
    header: hunk.header,
    rows,
  }
  sideBySideHunkCache.set(hunk, result)
  return result
}
