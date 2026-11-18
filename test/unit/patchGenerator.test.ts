import { describe, it, expect } from 'vitest'
import { buildHunkPatch, buildSelectedLinesPatch } from '~/utils/patchGenerator'
import type { FileDiff, DiffHunk } from '~/types/repository'

describe('patchGenerator', () => {
  const sampleHunk: DiffHunk = {
    header: '@@ -1,5 +1,5 @@',
    old_start: 1,
    old_lines: 5,
    new_start: 1,
    new_lines: 5,
    lines: [
      { line_type: 'context', old_line_number: 1, new_line_number: 1, content: 'line 1' },
      { line_type: 'deletion', old_line_number: 2, new_line_number: null, content: 'line 2' },
      { line_type: 'addition', old_line_number: null, new_line_number: 2, content: 'line 2 modified' },
      { line_type: 'deletion', old_line_number: 3, new_line_number: null, content: 'line 3' },
      { line_type: 'addition', old_line_number: null, new_line_number: 3, content: 'line 3 modified' },
      { line_type: 'context', old_line_number: 4, new_line_number: 4, content: 'line 4' },
    ],
  }

  const sampleFileDiff: FileDiff = {
    old_path: 'src/main.ts',
    new_path: 'src/main.ts',
    is_binary: false,
    is_new_file: false,
    is_deleted_file: false,
    hunks: [sampleHunk],
    total_additions: 2,
    total_deletions: 2,
  }

  it('builds whole hunk patch correctly', () => {
    const patch = buildHunkPatch(sampleFileDiff, 0)
    expect(patch).not.toBeNull()
    expect(patch).toContain('--- a/src/main.ts\n+++ b/src/main.ts\n')
    expect(patch).toContain('@@ -1,5 +1,5 @@\n')
    expect(patch).toContain(' line 1\n-line 2\n+line 2 modified\n-line 3\n+line 3 modified\n line 4\n')
  })

  it('builds new file patch with /dev/null header', () => {
    const newFileDiff: FileDiff = {
      ...sampleFileDiff,
      is_new_file: true,
      old_path: null,
    }
    const patch = buildHunkPatch(newFileDiff, 0)
    expect(patch).toContain('--- /dev/null\n+++ b/src/main.ts\n')
  })

  it('builds line-level staging patch with unselected changes converted to context or omitted', () => {
    // Select only index 1 (-line 2) and index 2 (+line 2 modified) for staging
    const patch = buildSelectedLinesPatch(sampleFileDiff, 0, [1, 2], 'stage')
    expect(patch).not.toBeNull()
    expect(patch).toContain('--- a/src/main.ts\n+++ b/src/main.ts\n')
    // line 3 deletion was unselected, so it should stay as context line: ' line 3'
    expect(patch).toContain(' line 3\n')
    // line 3 addition was unselected, so it should be omitted
    expect(patch).not.toContain('+line 3 modified')
    // line 2 addition and deletion were selected
    expect(patch).toContain('-line 2\n+line 2 modified\n')
  })

  it('builds line-level unstage/discard patch with unselected additions kept as context and deletions omitted', () => {
    // Unstage only index 2 (+line 2 modified)
    const patch = buildSelectedLinesPatch(sampleFileDiff, 0, [2], 'unstage')
    expect(patch).not.toBeNull()
    // Unselected addition (+line 3 modified) stays as context
    expect(patch).toContain(' line 3 modified\n')
    // Selected addition (+line 2 modified) remains addition to be reversed
    expect(patch).toContain('+line 2 modified\n')
    // Unselected deletions (-line 2, -line 3) omitted because they are already deleted
    expect(patch).not.toContain('-line 2')
    expect(patch).not.toContain('-line 3')
  })

  it('returns null for non-existent hunk index or empty selected indices', () => {
    expect(buildHunkPatch(sampleFileDiff, 99)).toBeNull()
    expect(buildSelectedLinesPatch(sampleFileDiff, 0, [], 'stage')).toBeNull()
  })
})
