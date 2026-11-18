function safeState<T>(key: string, initialValue: () => T) {
  if (tryUseNuxtApp()) {
    return useState<T>(key, initialValue)
  }
  return ref(initialValue())
}

export function useCommandPalette() {
  const isCommandPaletteOpen = safeState<boolean>('gitverse_command_palette_open', () => false)

  function openCommandPalette(): void {
    isCommandPaletteOpen.value = true
  }

  function closeCommandPalette(): void {
    isCommandPaletteOpen.value = false
  }

  function toggleCommandPalette(): void {
    isCommandPaletteOpen.value = !isCommandPaletteOpen.value
  }

  return {
    isCommandPaletteOpen,
    openCommandPalette,
    closeCommandPalette,
    toggleCommandPalette,
  }
}
