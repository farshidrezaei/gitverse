import { describe, expect, it } from 'vitest'
import { computeIntraLineDiff, buildSideBySideHunk } from '../../app/utils/diffLayout'
import { buildHunkPatch, buildSelectedLinesPatch } from '../../app/utils/patchGenerator'
import type { DiffHunk, FileDiff } from '../../app/types/repository'

describe('Table-Driven Diff Engine & Layout Matrix', () => {
  describe('Intra-line Character Highlighting Matrix', () => {
    interface IntraLineTestCase {
      description: string
      oldLine: string
      newLine: string
      expectedOldChangedText: string
      expectedNewChangedText: string
    }

    const intraLineTestCases: IntraLineTestCase[] = [
      {
        description: 'Identical lines produce zero changed spans',
        oldLine: 'const name = "gitverse"',
        newLine: 'const name = "gitverse"',
        expectedOldChangedText: '',
        expectedNewChangedText: '',
      },
      {
        description: 'Middle word change with common prefix and suffix',
        oldLine: 'const status = "inactive";',
        newLine: 'const status = "active";',
        expectedOldChangedText: 'in',
        expectedNewChangedText: '',
      },
      {
        description: 'Value replacement inside parentheses sharing trailing zero',
        oldLine: 'calculateTotal(100, tax)',
        newLine: 'calculateTotal(250, tax)',
        expectedOldChangedText: '10',
        expectedNewChangedText: '25',
      },
      {
        description: 'Prefix change with identical suffix',
        oldLine: 'oldPrefix_variableName',
        newLine: 'newPrefix_variableName',
        expectedOldChangedText: 'old',
        expectedNewChangedText: 'new',
      },
      {
        description: 'Suffix change with identical prefix',
        oldLine: 'handler_v1()',
        newLine: 'handler_v2()',
        expectedOldChangedText: '1',
        expectedNewChangedText: '2',
      },
      {
        description: 'Entirely different lines sharing trailing bracket',
        oldLine: 'foo()',
        newLine: 'bar(x, y)',
        expectedOldChangedText: 'foo(',
        expectedNewChangedText: 'bar(x, y',
      },
      {
        description: 'Addition to empty line',
        oldLine: '',
        newLine: 'return result;',
        expectedOldChangedText: '',
        expectedNewChangedText: 'return result;',
      },
      {
        description: 'Persian unicode token change',
        oldLine: 'وضعیت: در حال پردازش',
        newLine: 'وضعیت: تکمیل شده',
        expectedOldChangedText: 'در حال پردازش',
        expectedNewChangedText: 'تکمیل شده',
      },
    ]

    it.each(intraLineTestCases)(
      'Matrix: $description',
      ({ oldLine, newLine, expectedOldChangedText, expectedNewChangedText }) => {
        const { oldSpans, newSpans } = computeIntraLineDiff(oldLine, newLine)

        const oldChanged = oldSpans
          .filter(span => span.isChanged)
          .map(span => span.text)
          .join('')
        const newChanged = newSpans
          .filter(span => span.isChanged)
          .map(span => span.text)
          .join('')

        expect(oldChanged).toBe(expectedOldChangedText)
        expect(newChanged).toBe(expectedNewChangedText)
      }
    )
  })

  describe('Side-by-Side Hunk Alignment Matrix', () => {
    it('aligns pure additions, pure deletions, and modified pairs', () => {
      const hunk: DiffHunk = {
        header: '@@ -10,3 +10,4 @@',
        old_start: 10,
        old_lines_count: 3,
        new_start: 10,
        new_lines_count: 4,
        lines: [
          { line_type: 'context', old_line_number: 10, new_line_number: 10, content: 'function init() {' },
          { line_type: 'deletion', old_line_number: 11, new_line_number: null, content: '  const version = "1.0";' },
          { line_type: 'addition', old_line_number: null, new_line_number: 11, content: '  const version = "2.0";' },
          { line_type: 'addition', old_line_number: null, new_line_number: 12, content: '  const isReady = true;' },
          { line_type: 'context', old_line_number: 12, new_line_number: 13, content: '}' },
        ],
      }

      const sideBySide = buildSideBySideHunk(hunk)

      expect(sideBySide.rows.length).toBe(4)
      // Row 0: Context
      expect(sideBySide.rows[0]?.left.lineType).toBe('context')
      expect(sideBySide.rows[0]?.right.lineType).toBe('context')

      // Row 1: Modified pair (left and right marked as modification)
      expect(sideBySide.rows[1]?.left.lineType).toBe('modification')
      expect(sideBySide.rows[1]?.right.lineType).toBe('modification')
      expect(sideBySide.rows[1]?.left.spans?.some(span => span.isChanged)).toBe(true)

      // Row 2: Pure addition (left empty, right addition)
      expect(sideBySide.rows[2]?.left.lineType).toBe('empty')
      expect(sideBySide.rows[2]?.right.lineType).toBe('addition')

      // Row 3: Context
      expect(sideBySide.rows[3]?.left.lineType).toBe('context')
      expect(sideBySide.rows[3]?.right.lineType).toBe('context')
    })
  })

  describe('Patch Generation Matrix across File Lifecycle States', () => {
    const sampleHunk: DiffHunk = {
      header: '@@ -1,3 +1,3 @@',
      old_start: 1,
      old_lines_count: 3,
      new_start: 1,
      new_lines_count: 3,
      lines: [
        { line_type: 'context', old_line_number: 1, new_line_number: 1, content: 'line 1' },
        { line_type: 'deletion', old_line_number: 2, new_line_number: null, content: 'old line 2' },
        { line_type: 'addition', old_line_number: null, new_line_number: 2, content: 'new line 2' },
        { line_type: 'context', old_line_number: 3, new_line_number: 3, content: 'line 3' },
      ],
    }

    interface PatchLifecycleTestCase {
      description: string
      fileDiff: FileDiff
      expectedHeaderStart: string
    }

    const lifecycleCases: PatchLifecycleTestCase[] = [
      {
        description: 'Modified file patch headers',
        fileDiff: {
          old_path: 'src/app.ts',
          new_path: 'src/app.ts',
          change_status: 'modified',
          is_binary: false,
          is_new_file: false,
          is_deleted_file: false,
          hunks: [sampleHunk],
          stats: { additions: 1, deletions: 1 },
        },
        expectedHeaderStart: '--- a/src/app.ts\n+++ b/src/app.ts\n',
      },
      {
        description: 'New untracked/added file patch headers (/dev/null source)',
        fileDiff: {
          old_path: null,
          new_path: 'src/new-feature.ts',
          change_status: 'added',
          is_binary: false,
          is_new_file: true,
          is_deleted_file: false,
          hunks: [sampleHunk],
          stats: { additions: 3, deletions: 0 },
        },
        expectedHeaderStart: '--- /dev/null\n+++ b/src/new-feature.ts\n',
      },
      {
        description: 'Deleted file patch headers (/dev/null destination)',
        fileDiff: {
          old_path: 'obsolete.txt',
          new_path: 'obsolete.txt',
          change_status: 'deleted',
          is_binary: false,
          is_new_file: false,
          is_deleted_file: true,
          hunks: [sampleHunk],
          stats: { additions: 0, deletions: 3 },
        },
        expectedHeaderStart: '--- a/obsolete.txt\n+++ /dev/null\n',
      },
    ]

    it.each(lifecycleCases)(
      'Generates correct headers for: $description',
      ({ fileDiff, expectedHeaderStart }) => {
        const patch = buildHunkPatch(fileDiff, 0)
        expect(patch).not.toBeNull()
        expect(patch?.startsWith(expectedHeaderStart)).toBe(true)
        expect(patch).toContain('@@ -1,3 +1,3 @@')
      }
    )

    it('generates partial patch for selective line staging and discarding', () => {
      const fileDiff: FileDiff = {
        old_path: 'test.txt',
        new_path: 'test.txt',
        change_status: 'modified',
        is_binary: false,
        is_new_file: false,
        is_deleted_file: false,
        hunks: [sampleHunk],
        stats: { additions: 1, deletions: 1 },
      }

      // Stage only line index 2 (addition: 'new line 2')
      const stagedPatch = buildSelectedLinesPatch(fileDiff, 0, [2], 'stage')
      expect(stagedPatch).not.toBeNull()
      expect(stagedPatch).toContain('+new line 2')

      // Discard only line index 2
      const discardPatch = buildSelectedLinesPatch(fileDiff, 0, [2], 'discard')
      expect(discardPatch).not.toBeNull()
      expect(discardPatch).toContain('+new line 2')
    })
  })
})
