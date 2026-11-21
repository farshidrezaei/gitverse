import { describe, expect, it } from 'vitest'
import type { RepositorySearchResult } from '~/types/repository'

describe('repository code search data structure', () => {
  const sampleResult: RepositorySearchResult = {
    query: 'struct Commit',
    total_matches: 3,
    file_count: 2,
    matches: [
      {
        file_path: 'src-tauri/src/domain/graph.rs',
        line_number: 12,
        line_content: 'pub struct Commit {',
      },
      {
        file_path: 'src-tauri/src/domain/graph.rs',
        line_number: 45,
        line_content: '    pub fn new_commit() -> Commit {',
      },
      {
        file_path: 'app/types/repository.ts',
        line_number: 88,
        line_content: 'export interface CommitNode {',
      },
    ],
  }

  it('aggregates matches and groups by file correctly', () => {
    const map = new Map<string, typeof sampleResult.matches>()
    for (const match of sampleResult.matches) {
      const list = map.get(match.file_path) || []
      list.push(match)
      map.set(match.file_path, list)
    }

    expect(map.size).toBe(2)
    expect(map.get('src-tauri/src/domain/graph.rs')?.length).toBe(2)
    expect(map.get('app/types/repository.ts')?.length).toBe(1)
  })

  it('preserves query and match counts', () => {
    expect(sampleResult.query).toBe('struct Commit')
    expect(sampleResult.total_matches).toBe(3)
    expect(sampleResult.file_count).toBe(2)
  })
})
