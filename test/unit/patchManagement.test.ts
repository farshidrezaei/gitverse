import { describe, expect, it } from 'vitest'
import type { PatchCheckResult, PatchExportScope } from '~/types/repository'

describe('patch management and validation', () => {
  it('extracts affected file paths from a unified diff patch string', () => {
    const samplePatch = `diff --git a/app/components/Header.vue b/app/components/Header.vue
index 1234567..abcdef0 100644
--- a/app/components/Header.vue
+++ b/app/components/Header.vue
@@ -10,3 +10,4 @@
+<script setup lang="ts">
diff --git a/src-tauri/src/main.rs b/src-tauri/src/main.rs
index abcdef0..1234567 100644
--- a/src-tauri/src/main.rs
+++ b/src-tauri/src/main.rs
@@ -1,2 +1,3 @@
+// modified
`

    function extractAffectedFiles(patchContent: string): string[] {
      const files: string[] = []
      for (const line of patchContent.split('\n')) {
        if (line.startsWith('diff --git a/')) {
          const rest = line.slice('diff --git a/'.length)
          const spaceIndex = rest.indexOf(' b/')
          if (spaceIndex !== -1) {
            const filePath = rest.slice(0, spaceIndex)
            if (!files.includes(filePath)) {
              files.push(filePath)
            }
          }
        }
      }
      return files
    }

    const files = extractAffectedFiles(samplePatch)
    expect(files).toEqual(['app/components/Header.vue', 'src-tauri/src/main.rs'])
  })

  it('determines patch export filename based on scope and commit hash', () => {
    function computePatchFilename(scope: PatchExportScope, commitHash?: string | null): string {
      if (scope === 'commit' && commitHash) {
        return `commit-${commitHash.slice(0, 8)}.patch`
      }
      return `${scope}-changes.patch`
    }

    expect(computePatchFilename('unstaged')).toBe('unstaged-changes.patch')
    expect(computePatchFilename('staged')).toBe('staged-changes.patch')
    expect(computePatchFilename('working_tree')).toBe('working_tree-changes.patch')
    expect(computePatchFilename('commit', '4ba8dfc1234567890abcdef')).toBe('commit-4ba8dfc1.patch')
  })

  it('evaluates patch apply safety from pre-check results', () => {
    function isPatchSafeToApply(result: PatchCheckResult | null): boolean {
      return Boolean(result && result.can_apply && !result.error_message)
    }

    expect(isPatchSafeToApply(null)).toBe(false)
    expect(
      isPatchSafeToApply({
        can_apply: false,
        error_message: 'error: patch failed: file.txt:1',
        affected_files: ['file.txt'],
      })
    ).toBe(false)
    expect(
      isPatchSafeToApply({
        can_apply: true,
        error_message: null,
        affected_files: ['file.txt', 'config.json'],
      })
    ).toBe(true)
  })
})
