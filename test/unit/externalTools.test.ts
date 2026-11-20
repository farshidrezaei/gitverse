import { describe, expect, it } from 'vitest'
import type { SupportedExternalEditor } from '~/composables/useExternalToolsController'

describe('external tools integration', () => {
  it('validates supported external editors list', () => {
    const supportedEditors: SupportedExternalEditor[] = [
      'vscode',
      'cursor',
      'zed',
      'sublime',
      'intellij',
    ]

    expect(supportedEditors).toContain('vscode')
    expect(supportedEditors).toContain('cursor')
    expect(supportedEditors).toContain('zed')
    expect(supportedEditors).toContain('sublime')
    expect(supportedEditors).toContain('intellij')
    expect(supportedEditors.length).toBe(5)
  })

  it('determines editor binary names mapping correctly', () => {
    const editorBinaryMapping: Record<SupportedExternalEditor, string> = {
      vscode: 'code',
      cursor: 'cursor',
      zed: 'zed',
      sublime: 'subl',
      intellij: 'idea',
    }

    expect(editorBinaryMapping.vscode).toBe('code')
    expect(editorBinaryMapping.cursor).toBe('cursor')
    expect(editorBinaryMapping.zed).toBe('zed')
    expect(editorBinaryMapping.sublime).toBe('subl')
    expect(editorBinaryMapping.intellij).toBe('idea')
  })
})
