export interface ActionHistoryEntry {
  id: string
  title: string
  description?: string
  undo: () => Promise<void>
  redo?: () => Promise<void>
  timestamp: number
}

const MAX_HISTORY_STACK_SIZE = 50

function safeState<T>(key: string, init: () => T) {
  if (tryUseNuxtApp()) {
    return useState<T>(key, init)
  }
  return ref(init())
}

export function useActionHistory() {
  const undoStack = safeState<ActionHistoryEntry[]>('action-history-undo-stack', () => [])
  const redoStack = safeState<ActionHistoryEntry[]>('action-history-redo-stack', () => [])
  const isExecuting = safeState<boolean>('action-history-is-executing', () => false)

  const canUndo = computed(() => undoStack.value.length > 0 && !isExecuting.value)
  const canRedo = computed(() => redoStack.value.length > 0 && !isExecuting.value)

  function pushAction(entry: Omit<ActionHistoryEntry, 'id' | 'timestamp'>): ActionHistoryEntry {
    const fullEntry: ActionHistoryEntry = {
      ...entry,
      id: `action-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
      timestamp: Date.now(),
    }

    undoStack.value = [fullEntry, ...undoStack.value].slice(0, MAX_HISTORY_STACK_SIZE)
    redoStack.value = [] // Clear redo stack on new action
    return fullEntry
  }

  async function undo(): Promise<boolean> {
    if (undoStack.value.length === 0 || isExecuting.value) {
      return false
    }

    const [actionToUndo, ...remainingUndoStack] = undoStack.value
    if (!actionToUndo) return false

    isExecuting.value = true
    try {
      await actionToUndo.undo()
      undoStack.value = remainingUndoStack
      if (actionToUndo.redo) {
        redoStack.value = [actionToUndo, ...redoStack.value].slice(0, MAX_HISTORY_STACK_SIZE)
      }
      return true
    } catch (error) {
      console.error(`Failed to undo action "${actionToUndo.title}":`, error)
      return false
    } finally {
      isExecuting.value = false
    }
  }

  async function redo(): Promise<boolean> {
    if (redoStack.value.length === 0 || isExecuting.value) {
      return false
    }

    const [actionToRedo, ...remainingRedoStack] = redoStack.value
    if (!actionToRedo || !actionToRedo.redo) return false

    isExecuting.value = true
    try {
      await actionToRedo.redo()
      redoStack.value = remainingRedoStack
      undoStack.value = [actionToRedo, ...undoStack.value].slice(0, MAX_HISTORY_STACK_SIZE)
      return true
    } catch (error) {
      console.error(`Failed to redo action "${actionToRedo.title}":`, error)
      return false
    } finally {
      isExecuting.value = false
    }
  }

  function clearHistory(): void {
    undoStack.value = []
    redoStack.value = []
  }

  /**
   * Sets up window keyboard listeners for Ctrl+Z and Ctrl+Shift+Z / Ctrl+Y.
   * Protects text inputs so normal editing undo is not intercepted.
   */
  function registerKeyboardShortcuts(): () => void {
    if (!import.meta.client) {
      return () => {}
    }

    const handleKeyDown = (event: KeyboardEvent) => {
      // Do not intercept if focus is inside text input, textarea, or contentEditable element
      const activeElement = document.activeElement as HTMLElement | null
      if (
        activeElement &&
        (activeElement.tagName === 'INPUT' ||
          activeElement.tagName === 'TEXTAREA' ||
          activeElement.isContentEditable)
      ) {
        return
      }

      const isMac = navigator.platform.toUpperCase().includes('MAC')
      const modifierKey = isMac ? event.metaKey : event.ctrlKey

      if (!modifierKey) return

      // Redo: Ctrl+Shift+Z or Ctrl+Y (on Windows/Linux)
      if (
        (event.key === 'z' || event.key === 'Z') &&
        event.shiftKey
      ) {
        event.preventDefault()
        void redo()
      } else if (!isMac && (event.key === 'y' || event.key === 'Y')) {
        event.preventDefault()
        void redo()
      } else if (event.key === 'z' || event.key === 'Z') {
        event.preventDefault()
        void undo()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => {
      window.removeEventListener('keydown', handleKeyDown)
    }
  }

  return {
    undoStack: readonly(undoStack),
    redoStack: readonly(redoStack),
    canUndo,
    canRedo,
    isExecuting: readonly(isExecuting),
    pushAction,
    undo,
    redo,
    clearHistory,
    registerKeyboardShortcuts,
  }
}
