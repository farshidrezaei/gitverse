import { describe, expect, it } from 'vitest'
import { useGitOutputLog } from '~/composables/useGitOutputLog'

describe('useGitOutputLog Composable', () => {
  it('initializes with default ready entry and panel closed', () => {
    const { logEntries, isBottomPanelOpen, activeBottomPanelTab } = useGitOutputLog()

    expect(logEntries.value.length).toBeGreaterThan(0)
    expect(isBottomPanelOpen.value).toBe(false)
    expect(activeBottomPanelTab.value).toBe('output')
  })

  it('records and updates log entries', () => {
    const { recordLog, updateLog, logEntries } = useGitOutputLog()

    const logId = recordLog('git status', 'running')
    expect(logEntries.value[0]?.id).toBe(logId)
    expect(logEntries.value[0]?.status).toBe('running')

    updateLog(logId, { status: 'success', output: 'clean' })
    expect(logEntries.value[0]?.status).toBe('success')
    expect(logEntries.value[0]?.output).toBe('clean')
  })

  it('toggles bottom panel state and switches tabs', () => {
    const {
      isBottomPanelOpen,
      activeBottomPanelTab,
      toggleBottomPanel,
      openBottomPanel,
      closeBottomPanel,
    } = useGitOutputLog()

    isBottomPanelOpen.value = false
    toggleBottomPanel()
    expect(isBottomPanelOpen.value).toBe(true)

    openBottomPanel('terminal')
    expect(isBottomPanelOpen.value).toBe(true)
    expect(activeBottomPanelTab.value).toBe('terminal')

    closeBottomPanel()
    expect(isBottomPanelOpen.value).toBe(false)
  })
})
