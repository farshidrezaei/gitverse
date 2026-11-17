import { describe, expect, it } from 'vitest'
import {
  combineOursAndTheirs,
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
})
