import { describe, expect, it } from 'vitest'
import { buildSideBySideHunk, computeIntraLineDiff } from '../../app/utils/diffLayout'
import type { DiffHunk } from '../../app/types/repository'

describe('diffLayout - buildSideBySideHunk', () => {
  it('correctly maps pure context lines to identical left and right columns', () => {
    const hunk: DiffHunk = {
      header: '@@ -1,2 +1,2 @@',
      old_start: 1,
      old_lines: 2,
      new_start: 1,
      new_lines: 2,
      lines: [
        {
          line_type: 'context',
          content: ' const a = 1;',
          old_line_number: 1,
          new_line_number: 1,
        },
        {
          line_type: 'context',
          content: ' const b = 2;',
          old_line_number: 2,
          new_line_number: 2,
        },
      ],
    }

    const result = buildSideBySideHunk(hunk)
    expect(result.header).toBe('@@ -1,2 +1,2 @@')
    expect(result.rows).toHaveLength(2)

    expect(result.rows[0]?.left).toEqual({
      lineNumber: 1,
      content: 'const a = 1;',
      lineType: 'context',
      spans: [{ text: 'const a = 1;', isChanged: false }],
    })
    expect(result.rows[0]?.right).toEqual({
      lineNumber: 1,
      content: 'const a = 1;',
      lineType: 'context',
      spans: [{ text: 'const a = 1;', isChanged: false }],
    })
  })

  it('correctly aligns consecutive deletion followed by addition into paired rows', () => {
    const hunk: DiffHunk = {
      header: '@@ -10,3 +10,4 @@',
      old_start: 10,
      old_lines: 3,
      new_start: 10,
      new_lines: 4,
      lines: [
        {
          line_type: 'deletion',
          content: '-let isOld = true;',
          old_line_number: 10,
          new_line_number: null,
        },
        {
          line_type: 'deletion',
          content: '-let isLegacy = false;',
          old_line_number: 11,
          new_line_number: null,
        },
        {
          line_type: 'addition',
          content: '+let isNew = true;',
          old_line_number: null,
          new_line_number: 10,
        },
      ],
    }

    const result = buildSideBySideHunk(hunk)
    expect(result.rows).toHaveLength(2)

    // Row 0: first deletion pairs with first addition -> becomes modification (Blue 3-color mode)
    expect(result.rows[0]?.left).toEqual({
      lineNumber: 10,
      content: 'let isOld = true;',
      lineType: 'modification',
      spans: [
        { text: 'let is', isChanged: false },
        { text: 'Old', isChanged: true },
        { text: ' = true;', isChanged: false },
      ],
    })
    expect(result.rows[0]?.right).toEqual({
      lineNumber: 10,
      content: 'let isNew = true;',
      lineType: 'modification',
      spans: [
        { text: 'let is', isChanged: false },
        { text: 'New', isChanged: true },
        { text: ' = true;', isChanged: false },
      ],
    })

    // Row 1: second deletion pairs with empty placeholder -> pure deletion (Red)
    expect(result.rows[1]?.left).toEqual({
      lineNumber: 11,
      content: 'let isLegacy = false;',
      lineType: 'deletion',
      spans: [{ text: 'let isLegacy = false;', isChanged: true }],
    })
    expect(result.rows[1]?.right).toEqual({
      lineNumber: null,
      content: '',
      lineType: 'empty',
    })
  })

  it('correctly computes intra-line diff for arbitrary text changes', () => {
    const diff = computeIntraLineDiff('const count = 10;', 'const count = 25;')

    expect(diff.oldSpans).toEqual([
      { text: 'const count = ', isChanged: false },
      { text: '10', isChanged: true },
      { text: ';', isChanged: false },
    ])
    expect(diff.newSpans).toEqual([
      { text: 'const count = ', isChanged: false },
      { text: '25', isChanged: true },
      { text: ';', isChanged: false },
    ])
  })
})
