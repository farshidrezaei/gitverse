import { computeIntraLineDiff, type DiffTextSpan } from '~/utils/diffLayout'

export interface ConflictLineComparison {
  lineNumber: number | null
  content: string
  status: 'context' | 'addition' | 'deletion' | 'modification'
  spans?: DiffTextSpan[]
}

export interface ParsedConflictChunk {
  id: string
  startIndex: number
  endIndex: number
  startLine: number
  endLine: number
  oursBranchName: string
  oursContent: string
  theirsBranchName: string
  theirsContent: string
  rawBlock: string
}

/**
 * Parses all conflict marker blocks (<<<<<<<, =======, >>>>>>>) into structured chunks.
 */
export function parseConflictChunks(content: string): ParsedConflictChunk[] {
  if (!content) return []

  const chunks: ParsedConflictChunk[] = []
  const lines = content.split(/\r?\n/)
  let inConflict = false
  let currentStartLine = 0
  let currentOursBranchName = ''
  let currentTheirsBranchName = ''
  let currentOursLines: string[] = []
  let currentTheirsLines: string[] = []
  let inTheirs = false
  let rawBlockLines: string[] = []

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index]!
    const lineNum = index + 1

    if (line.startsWith('<<<<<<<')) {
      inConflict = true
      inTheirs = false
      currentStartLine = lineNum
      currentOursBranchName = line.slice(7).trim() || 'HEAD'
      currentOursLines = []
      currentTheirsLines = []
      rawBlockLines = [line]
    } else if (inConflict && line.startsWith('=======')) {
      inTheirs = true
      rawBlockLines.push(line)
    } else if (inConflict && line.startsWith('>>>>>>>')) {
      currentTheirsBranchName = line.slice(7).trim() || 'Incoming'
      rawBlockLines.push(line)
      chunks.push({
        id: `conflict-chunk-${chunks.length + 1}`,
        startIndex: currentStartLine - 1,
        endIndex: lineNum - 1,
        startLine: currentStartLine,
        endLine: lineNum,
        oursBranchName: currentOursBranchName,
        oursContent: currentOursLines.join('\n'),
        theirsBranchName: currentTheirsBranchName,
        theirsContent: currentTheirsLines.join('\n'),
        rawBlock: rawBlockLines.join('\n'),
      })
      inConflict = false
      inTheirs = false
      rawBlockLines = []
    } else if (inConflict) {
      rawBlockLines.push(line)
      if (inTheirs) {
        currentTheirsLines.push(line)
      } else {
        currentOursLines.push(line)
      }
    }
  }

  return chunks
}

/**
 * Resolves a single conflict chunk within the full document with a chosen resolution strategy.
 */
export function resolveConflictChunk(
  documentContent: string,
  chunk: ParsedConflictChunk,
  strategy: 'ours' | 'theirs' | 'both_ours_first' | 'both_theirs_first'
): string {
  let resolvedText = ''
  switch (strategy) {
    case 'ours':
      resolvedText = chunk.oursContent
      break
    case 'theirs':
      resolvedText = chunk.theirsContent
      break
    case 'both_ours_first':
      resolvedText = combineOursAndTheirs(chunk.oursContent, chunk.theirsContent)
      break
    case 'both_theirs_first':
      resolvedText = combineOursAndTheirs(chunk.theirsContent, chunk.oursContent)
      break
  }

  const lines = documentContent.split(/\r?\n/)
  const beforeLines = lines.slice(0, chunk.startIndex)
  const afterLines = lines.slice(chunk.endIndex + 1)
  const middleLines = resolvedText.length > 0 ? resolvedText.split(/\r?\n/) : []

  return [...beforeLines, ...middleLines, ...afterLines].join('\n')
}

/**
 * Counts number of unresolved conflict sections within a file.
 */
export function countConflictMarkers(content: string): number {
  if (!content) {
    return 0
  }
  const matches = content.match(/^<{7}\s/gm)
  return matches ? matches.length : 0
}

/**
 * Checks if a string still contains Git conflict markers.
 */
export function hasConflictMarkers(content: string): boolean {
  if (!content) {
    return false
  }
  return /^<{7}\s/m.test(content) && /^={7}$/m.test(content) && /^>{7}\s/m.test(content)
}

/**
 * Combines both Ours and Theirs content sequentially with a newline separation.
 */
export function combineOursAndTheirs(oursContent: string, theirsContent: string): string {
  const normalizedOurs = oursContent.trimEnd()
  const normalizedTheirs = theirsContent.trimStart()
  if (!normalizedOurs) {
    return normalizedTheirs
  }
  if (!normalizedTheirs) {
    return normalizedOurs
  }
  return `${normalizedOurs}\n${normalizedTheirs}`
}

/**
 * Computes line-by-line differences of a branch version against the base ancestor.
 * Produces 3-color status (addition/green, deletion/red, modification/blue with intra-line diff).
 */
export function computeConflictLineComparison(
  branchContent: string,
  baseContent: string
): ConflictLineComparison[] {
  if (!branchContent && !baseContent) {
    return []
  }

  const branchLines = branchContent ? branchContent.split(/\r?\n/) : []
  const baseLines = baseContent ? baseContent.split(/\r?\n/) : []

  if (baseLines.length === 0) {
    return branchLines.map((line, index) => ({
      lineNumber: index + 1,
      content: line,
      status: 'addition',
      spans: [{ text: line, isChanged: true }],
    }))
  }

  if (branchLines.length === 0) {
    return baseLines.map((line, index) => ({
      lineNumber: index + 1,
      content: line,
      status: 'deletion',
      spans: [{ text: line, isChanged: true }],
    }))
  }

  // Guard against extreme file sizes to maintain fast UI responsiveness
  const totalMatrixSize = baseLines.length * branchLines.length
  if (totalMatrixSize > 2_000_000) {
    return branchLines.map((line, index) => ({
      lineNumber: index + 1,
      content: line,
      status: 'context',
      spans: [{ text: line, isChanged: false }],
    }))
  }

  // Longest Common Subsequence matrix
  const baseCount = baseLines.length
  const branchCount = branchLines.length
  const matrix: number[][] = Array.from({ length: baseCount + 1 }, () =>
    new Array(branchCount + 1).fill(0)
  )

  for (let baseIndex = 0; baseIndex < baseCount; baseIndex += 1) {
    for (let branchIndex = 0; branchIndex < branchCount; branchIndex += 1) {
      if (baseLines[baseIndex] === branchLines[branchIndex]) {
        matrix[baseIndex + 1]![branchIndex + 1] = (matrix[baseIndex]![branchIndex] || 0) + 1
      } else {
        matrix[baseIndex + 1]![branchIndex + 1] = Math.max(
          matrix[baseIndex]![branchIndex + 1] || 0,
          matrix[baseIndex + 1]![branchIndex] || 0
        )
      }
    }
  }

  // Backtrack to assemble edit operations
  let currentBaseIndex = baseCount
  let currentBranchIndex = branchCount
  const editOperations: Array<{
    type: 'context' | 'deletion' | 'addition'
    baseLineIndex?: number
    branchLineIndex?: number
  }> = []

  while (currentBaseIndex > 0 || currentBranchIndex > 0) {
    if (
      currentBaseIndex > 0 &&
      currentBranchIndex > 0 &&
      baseLines[currentBaseIndex - 1] === branchLines[currentBranchIndex - 1]
    ) {
      editOperations.push({
        type: 'context',
        baseLineIndex: currentBaseIndex - 1,
        branchLineIndex: currentBranchIndex - 1,
      })
      currentBaseIndex -= 1
      currentBranchIndex -= 1
    } else if (
      currentBranchIndex > 0 &&
      (currentBaseIndex === 0 ||
        (matrix[currentBaseIndex]![currentBranchIndex - 1] || 0) >=
          (matrix[currentBaseIndex - 1]![currentBranchIndex] || 0))
    ) {
      editOperations.push({
        type: 'addition',
        branchLineIndex: currentBranchIndex - 1,
      })
      currentBranchIndex -= 1
    } else if (
      currentBaseIndex > 0 &&
      (currentBranchIndex === 0 ||
        (matrix[currentBaseIndex]![currentBranchIndex - 1] || 0) <
          (matrix[currentBaseIndex - 1]![currentBranchIndex] || 0))
    ) {
      editOperations.push({
        type: 'deletion',
        baseLineIndex: currentBaseIndex - 1,
      })
      currentBaseIndex -= 1
    }
  }

  editOperations.reverse()

  const comparisonResults: ConflictLineComparison[] = []
  let operationIndex = 0

  while (operationIndex < editOperations.length) {
    const currentOperation = editOperations[operationIndex]
    if (!currentOperation) {
      operationIndex += 1
      continue
    }

    if (currentOperation.type === 'context') {
      const lineText = branchLines[currentOperation.branchLineIndex!] || ''
      comparisonResults.push({
        lineNumber: currentOperation.branchLineIndex! + 1,
        content: lineText,
        status: 'context',
        spans: [{ text: lineText, isChanged: false }],
      })
      operationIndex += 1
      continue
    }

    // Collect contiguous deletions and additions
    const contiguousDeletions: number[] = []
    const contiguousAdditions: number[] = []

    while (
      operationIndex < editOperations.length &&
      editOperations[operationIndex]?.type === 'deletion'
    ) {
      contiguousDeletions.push(editOperations[operationIndex]!.baseLineIndex!)
      operationIndex += 1
    }

    while (
      operationIndex < editOperations.length &&
      editOperations[operationIndex]?.type === 'addition'
    ) {
      contiguousAdditions.push(editOperations[operationIndex]!.branchLineIndex!)
      operationIndex += 1
    }

    const pairedCount = Math.min(contiguousDeletions.length, contiguousAdditions.length)

    // Paired lines become In-place Modifications (Blue)
    for (let pairIndex = 0; pairIndex < pairedCount; pairIndex += 1) {
      const baseLineText = baseLines[contiguousDeletions[pairIndex]!] || ''
      const branchLineText = branchLines[contiguousAdditions[pairIndex]!] || ''
      const { newSpans } = computeIntraLineDiff(baseLineText, branchLineText)

      comparisonResults.push({
        lineNumber: contiguousAdditions[pairIndex]! + 1,
        content: branchLineText,
        status: 'modification',
        spans: newSpans,
      })
    }

    // Unpaired additions remain Additions (Green)
    for (let addIndex = pairedCount; addIndex < contiguousAdditions.length; addIndex += 1) {
      const branchLineText = branchLines[contiguousAdditions[addIndex]!] || ''
      comparisonResults.push({
        lineNumber: contiguousAdditions[addIndex]! + 1,
        content: branchLineText,
        status: 'addition',
        spans: [{ text: branchLineText, isChanged: true }],
      })
    }

    // Unpaired deletions indicate lines in Base that are omitted in this branch (Red placeholder)
    for (let deleteIndex = pairedCount; deleteIndex < contiguousDeletions.length; deleteIndex += 1) {
      const baseLineText = baseLines[contiguousDeletions[deleteIndex]!] || ''
      comparisonResults.push({
        lineNumber: null,
        content: baseLineText,
        status: 'deletion',
        spans: [{ text: baseLineText, isChanged: true }],
      })
    }
  }

  return comparisonResults
}
