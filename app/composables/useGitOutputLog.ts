export interface GitLogEntry {
  id: string
  timestamp: number
  command: string
  status: 'running' | 'success' | 'error'
  output?: string
  error?: string
  durationMilliseconds?: number
}

const MAX_LOG_ENTRIES = 100

function safeState<T>(key: string, initialValue: () => T) {
  if (tryUseNuxtApp()) {
    return useState<T>(key, initialValue)
  }
  return ref(initialValue())
}

export function useGitOutputLog() {
  const logEntries = safeState<GitLogEntry[]>('git-output-log-entries', () => [
    {
      id: 'initial-runtime-log',
      timestamp: Date.now(),
      command: 'git --version',
      status: 'success',
      output: 'Git engine initialized and ready.',
      durationMilliseconds: 12,
    },
  ])

  const isBottomPanelOpen = safeState<boolean>('git-bottom-panel-open', () => false)
  const activeBottomPanelTab = safeState<'output' | 'terminal'>('git-bottom-panel-tab', () => 'output')

  function recordLog(
    command: string,
    status: 'running' | 'success' | 'error',
    output?: string,
    error?: string,
    durationMilliseconds?: number,
  ): string {
    const entryId = `log-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`
    const newEntry: GitLogEntry = {
      id: entryId,
      timestamp: Date.now(),
      command,
      status,
      output,
      error,
      durationMilliseconds,
    }

    logEntries.value = [newEntry, ...logEntries.value].slice(0, MAX_LOG_ENTRIES)
    return entryId
  }

  function updateLog(
    id: string,
    update: Partial<Omit<GitLogEntry, 'id' | 'timestamp'>>,
  ): void {
    const targetEntry = logEntries.value.find((entry) => entry.id === id)
    if (targetEntry) {
      Object.assign(targetEntry, update)
    }
  }

  function clearLogs(): void {
    logEntries.value = []
  }

  function toggleBottomPanel(): void {
    isBottomPanelOpen.value = !isBottomPanelOpen.value
  }

  function openBottomPanel(tab: 'output' | 'terminal' = 'output'): void {
    activeBottomPanelTab.value = tab
    isBottomPanelOpen.value = true
  }

  function closeBottomPanel(): void {
    isBottomPanelOpen.value = false
  }

  return {
    logEntries,
    isBottomPanelOpen,
    activeBottomPanelTab,
    recordLog,
    updateLog,
    clearLogs,
    toggleBottomPanel,
    openBottomPanel,
    closeBottomPanel,
  }
}
