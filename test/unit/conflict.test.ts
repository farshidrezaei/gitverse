import { describe, expect, it } from 'vitest'
import {
  combineOursAndTheirs,
  computeConflictLineComparison,
  countConflictMarkers,
  hasConflictMarkers,
} from '~/utils/conflict'

describe('conflict utilities', () => {
  it('correctly counts conflict markers in content', () => {
    const content = `
const a = 1;
<<<<<<< HEAD
const b = 2;
=======
const b = 3;
>>>>>>> feature
console.log(a + b);
<<<<<<< HEAD
const c = 4;
=======
const c = 5;
>>>>>>> feature
`
    expect(countConflictMarkers(content)).toBe(2)
    expect(hasConflictMarkers(content)).toBe(true)
  })

  it('detects clean content without conflict markers', () => {
    const cleanContent = `
const a = 1;
const b = 2;
console.log(a + b);
`
    expect(countConflictMarkers(cleanContent)).toBe(0)
    expect(hasConflictMarkers(cleanContent)).toBe(false)
  })

  it('correctly combines ours and theirs content', () => {
    const ours = 'line 1\nline 2'
    const theirs = 'line 3\nline 4'
    const combined = combineOursAndTheirs(ours, theirs)
    expect(combined).toBe('line 1\nline 2\nline 3\nline 4')
  })

  it('correctly compares branch lines against base with 3-color categorization', () => {
    const base = 'const a = 1;\nconst b = 2;\nconst c = 3;'
    const branch = 'const a = 1;\nconst b = 20;\nconst c = 3;\nconst d = 4;'

    const comparisons = computeConflictLineComparison(branch, base)

    expect(comparisons).toHaveLength(4)

    // Line 1: unchanged context
    expect(comparisons[0]?.status).toBe('context')
    expect(comparisons[0]?.content).toBe('const a = 1;')

    // Line 2: modified (b changed from 2 to 20) -> Blue modification
    expect(comparisons[1]?.status).toBe('modification')
    expect(comparisons[1]?.content).toBe('const b = 20;')
    expect(comparisons[1]?.spans).toEqual([
      { text: 'const b = 2', isChanged: false },
      { text: '0', isChanged: true },
      { text: ';', isChanged: false },
    ])

    // Line 3: unchanged context
    expect(comparisons[2]?.status).toBe('context')
    expect(comparisons[2]?.content).toBe('const c = 3;')

    // Line 4: added line -> Green addition
    expect(comparisons[3]?.status).toBe('addition')
    expect(comparisons[3]?.content).toBe('const d = 4;')
  })
})
