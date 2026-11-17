import { describe, expect, it, vi } from 'vitest'
import { useActionHistory } from '~/composables/useActionHistory'

describe('useActionHistory', () => {
  it('initializes with empty undo and redo stacks', () => {
    const { undoStack, redoStack, canUndo, canRedo } = useActionHistory()
    expect(undoStack.value.length).toBe(0)
    expect(redoStack.value.length).toBe(0)
    expect(canUndo.value).toBe(false)
    expect(canRedo.value).toBe(false)
  })

  it('pushes actions and executes undo and redo correctly', async () => {
    const { pushAction, undo, redo, canUndo, canRedo, clearHistory } = useActionHistory()
    clearHistory()

    let value = 0
    const undoMock = vi.fn(async () => {
      value -= 10
    })
    const redoMock = vi.fn(async () => {
      value += 10
    })

    pushAction({
      title: 'Increment',
      undo: undoMock,
      redo: redoMock,
    })

    expect(canUndo.value).toBe(true)
    expect(canRedo.value).toBe(false)

    // Execute Undo
    const undoSuccess = await undo()
    expect(undoSuccess).toBe(true)
    expect(undoMock).toHaveBeenCalledTimes(1)
    expect(value).toBe(-10)
    expect(canUndo.value).toBe(false)
    expect(canRedo.value).toBe(true)

    // Execute Redo
    const redoSuccess = await redo()
    expect(redoSuccess).toBe(true)
    expect(redoMock).toHaveBeenCalledTimes(1)
    expect(value).toBe(0)
    expect(canUndo.value).toBe(true)
    expect(canRedo.value).toBe(false)
  })

  it('clears redo stack when a new action is pushed after an undo', async () => {
    const { pushAction, undo, canRedo, clearHistory } = useActionHistory()
    clearHistory()

    pushAction({
      title: 'Action 1',
      undo: async () => {},
      redo: async () => {},
    })

    await undo()
    expect(canRedo.value).toBe(true)

    // Push new action should wipe out redo stack
    pushAction({
      title: 'Action 2',
      undo: async () => {},
    })

    expect(canRedo.value).toBe(false)
  })
})
