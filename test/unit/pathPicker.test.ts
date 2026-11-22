import { describe, expect, it } from 'vitest'
import { usePathPickerModal } from '~/composables/usePathPickerModal'

describe('usePathPickerModal Composable', () => {
  it('initializes with modal closed', () => {
    const { isPathPickerModalOpen, pathPickerOptions } = usePathPickerModal()
    expect(isPathPickerModalOpen.value).toBe(false)
    expect(pathPickerOptions.value.mode).toBe('folder')
  })

  it('opens modal with requested options and resolves when confirmed', async () => {
    const {
      isPathPickerModalOpen,
      pathPickerOptions,
      openPathPicker,
      handlePathConfirmed,
    } = usePathPickerModal()

    const promise = openPathPicker({
      mode: 'file',
      title: 'Select Patch File',
      fileExtensions: ['patch', 'diff'],
    })

    expect(isPathPickerModalOpen.value).toBe(true)
    expect(pathPickerOptions.value.mode).toBe('file')
    expect(pathPickerOptions.value.title).toBe('Select Patch File')
    expect(pathPickerOptions.value.fileExtensions).toEqual(['patch', 'diff'])

    handlePathConfirmed('/home/user/test.patch')

    const result = await promise
    expect(result).toBe('/home/user/test.patch')
    expect(isPathPickerModalOpen.value).toBe(false)
  })

  it('resolves null when user cancels selection', async () => {
    const {
      isPathPickerModalOpen,
      openPathPicker,
      handlePathCancelled,
    } = usePathPickerModal()

    const promise = openPathPicker({
      mode: 'folder',
      title: 'Select Worktree Folder',
    })

    expect(isPathPickerModalOpen.value).toBe(true)
    handlePathCancelled()

    const result = await promise
    expect(result).toBeNull()
    expect(isPathPickerModalOpen.value).toBe(false)
  })
})
