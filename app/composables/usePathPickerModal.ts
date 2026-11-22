export type PathPickerMode = 'folder' | 'file' | 'save_file'

export interface PathPickerOptions {
  mode: PathPickerMode
  title?: string
  subtitle?: string
  confirmLabel?: string
  mustBeGitRepository?: boolean
  fileExtensions?: string[]
  initialPath?: string
  defaultFileName?: string
}

let activeResolveCallback: ((selectedPath: string | null) => void) | null = null

function safeState<T>(key: string, initialValue: () => T) {
  if (tryUseNuxtApp()) {
    return useState<T>(key, initialValue)
  }
  return ref(initialValue())
}

export function usePathPickerModal() {
  const isPathPickerModalOpen = safeState<boolean>('modal-path-picker-open', () => false)
  const pathPickerOptions = safeState<PathPickerOptions>('modal-path-picker-options', () => ({
    mode: 'folder',
  }))

  function openPathPicker(options: PathPickerOptions): Promise<string | null> {
    pathPickerOptions.value = options
    isPathPickerModalOpen.value = true
    return new Promise((resolve) => {
      activeResolveCallback = resolve
    })
  }

  function handlePathConfirmed(selectedPath: string): void {
    isPathPickerModalOpen.value = false
    if (activeResolveCallback) {
      activeResolveCallback(selectedPath)
      activeResolveCallback = null
    }
  }

  function handlePathCancelled(): void {
    isPathPickerModalOpen.value = false
    if (activeResolveCallback) {
      activeResolveCallback(null)
      activeResolveCallback = null
    }
  }

  return {
    isPathPickerModalOpen,
    pathPickerOptions,
    openPathPicker,
    handlePathConfirmed,
    handlePathCancelled,
  }
}
