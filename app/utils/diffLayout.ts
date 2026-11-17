import type { DiffHunk, DiffLine, DiffLineType } from '~/types/repository'

export interface SideBySideLineCell {
  lineNumber: number | null
  content: string
  lineType: DiffLineType | 'empty'
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
 * Transforms unified hunk lines into aligned side-by-side rows.
 * Aligns deletion lines on the left with replacement addition lines on the right.
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
      rows.push({
        id: `row-${rowIndex++}`,
        left: {
          lineNumber: currentLine.old_line_number,
          content: currentLine.content.startsWith(' ') ? currentLine.content.slice(1) : currentLine.content,
          lineType: 'context',
        },
        right: {
          lineNumber: currentLine.new_line_number,
          content: currentLine.content.startsWith(' ') ? currentLine.content.slice(1) : currentLine.content,
          lineType: 'context',
        },
      })
      lineIndex += 1
      continue
    }

    // Accumulate consecutive deletions and additions
    const deletions: DiffLine[] = []
    const additions: DiffLine[] = []

    while (lineIndex < lines.length && lines[lineIndex]?.line_type === 'deletion') {
      const line = lines[lineIndex]
      if (line) deletions.push(line)
      lineIndex += 1
    }

    while (lineIndex < lines.length && lines[lineIndex]?.line_type === 'addition') {
      const line = lines[lineIndex]
      if (line) additions.push(line)
      lineIndex += 1
    }

    const maxCount = Math.max(deletions.length, additions.length)
    for (let index = 0; index < maxCount; index += 1) {
      const deletion = deletions[index]
      const addition = additions[index]

      rows.push({
        id: `row-${rowIndex++}`,
        left: deletion
          ? {
              lineNumber: deletion.old_line_number,
              content: deletion.content.startsWith('-') ? deletion.content.slice(1) : deletion.content,
              lineType: 'deletion',
            }
          : {
              lineNumber: null,
              content: '',
              lineType: 'empty',
            },
        right: addition
          ? {
              lineNumber: addition.new_line_number,
              content: addition.content.startsWith('+') ? addition.content.slice(1) : addition.content,
              lineType: 'addition',
            }
          : {
              lineNumber: null,
              content: '',
              lineType: 'empty',
            },
      })
    }
  }

  const result: SideBySideHunk = {
    header: hunk.header,
    rows,
  }
  sideBySideHunkCache.set(hunk, result)
  return result
}
