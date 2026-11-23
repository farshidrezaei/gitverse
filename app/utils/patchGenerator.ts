import type { FileDiff, DiffHunk } from '~/types/repository'

export type PatchActionType = 'stage' | 'unstage' | 'discard'

function getFilePatchHeader(fileDiff: FileDiff): string {
  const oldPath = fileDiff.old_path || fileDiff.new_path
  const newPath = fileDiff.new_path

  if (fileDiff.is_new_file) {
    return `--- /dev/null\n+++ b/${newPath}\n`
  }
  if (fileDiff.is_deleted_file) {
    return `--- a/${oldPath}\n+++ /dev/null\n`
  }
  return `--- a/${oldPath}\n+++ b/${newPath}\n`
}

/**
 * Builds a valid unified diff patch for an entire hunk within a file diff.
 */
export function buildHunkPatch(fileDiff: FileDiff, hunkIndex: number): string | null {
  const targetHunk: DiffHunk | undefined = fileDiff.hunks[hunkIndex]
  if (!targetHunk) {
    return null
  }

  const fileHeader = getFilePatchHeader(fileDiff)
  let hunkBody = `${targetHunk.header}\n`

  for (const line of targetHunk.lines) {
    if (line.line_type === 'addition') {
      hunkBody += `+${line.content}\n`
    } else if (line.line_type === 'deletion') {
      hunkBody += `-${line.content}\n`
    } else if (line.line_type === 'context') {
      hunkBody += ` ${line.content}\n`
    }
    if (line.has_no_newline) {
      hunkBody += '\\ No newline at end of file\n'
    }
  }

  return `${fileHeader}${hunkBody}`
}

/**
 * Builds a valid unified diff patch for specific selected line indices within a hunk.
 * Supports staging unstaged lines, unstaging staged lines, or discarding unstaged lines.
 */
export function buildSelectedLinesPatch(
  fileDiff: FileDiff,
  hunkIndex: number,
  selectedLineIndices: number[],
  actionType: PatchActionType,
): string | null {
  const targetHunk: DiffHunk | undefined = fileDiff.hunks[hunkIndex]
  if (!targetHunk || selectedLineIndices.length === 0) {
    return null
  }

  const selectedSet = new Set(selectedLineIndices)
  const fileHeader = getFilePatchHeader(fileDiff)

  const outputLines: string[] = []
  let oldCount = 0
  let newCount = 0

  for (let index = 0; index < targetHunk.lines.length; index += 1) {
    const line = targetHunk.lines[index]
    if (!line) continue

    const isSelected = selectedSet.has(index)

    if (line.line_type === 'context') {
      outputLines.push(` ${line.content}`)
      if (line.has_no_newline) {
        outputLines.push('\\ No newline at end of file')
      }
      oldCount += 1
      newCount += 1
    } else if (actionType === 'stage') {
      // Staging: we are applying patch to index
      if (line.line_type === 'addition') {
        if (isSelected) {
          outputLines.push(`+${line.content}`)
          if (line.has_no_newline) {
            outputLines.push('\\ No newline at end of file')
          }
          newCount += 1
        }
        // If not selected, omitted from patch
      } else if (line.line_type === 'deletion') {
        if (isSelected) {
          outputLines.push(`-${line.content}`)
          if (line.has_no_newline) {
            outputLines.push('\\ No newline at end of file')
          }
          oldCount += 1
        } else {
          // Unselected deletion stays in index as context
          outputLines.push(` ${line.content}`)
          if (line.has_no_newline) {
            outputLines.push('\\ No newline at end of file')
          }
          oldCount += 1
          newCount += 1
        }
      }
    } else {
      // Unstaging (reverse from index) or Discarding (reverse from working tree)
      if (line.line_type === 'addition') {
        if (isSelected) {
          outputLines.push(`+${line.content}`)
          if (line.has_no_newline) {
            outputLines.push('\\ No newline at end of file')
          }
          newCount += 1
        } else {
          // Unselected addition stays in file/index as context
          outputLines.push(` ${line.content}`)
          if (line.has_no_newline) {
            outputLines.push('\\ No newline at end of file')
          }
          oldCount += 1
          newCount += 1
        }
      } else if (line.line_type === 'deletion') {
        if (isSelected) {
          outputLines.push(`-${line.content}`)
          if (line.has_no_newline) {
            outputLines.push('\\ No newline at end of file')
          }
          oldCount += 1
        }
        // If not selected, omitted from patch so it remains deleted
      }
    }
  }

  if (outputLines.length === 0) {
    return null
  }

  const reconstructedHeader = `@@ -${targetHunk.old_start},${oldCount} +${targetHunk.new_start},${newCount} @@`
  return `${fileHeader}${reconstructedHeader}\n${outputLines.join('\n')}\n`
}
