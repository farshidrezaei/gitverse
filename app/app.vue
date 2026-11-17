<script setup lang="ts">
const localeHead = useLocaleHead()
const { restorePersistedLocale } = useLocaleController()
const { restorePersistedTheme } = useThemeController()
const { registerKeyboardShortcuts } = useActionHistory()

// Initialize persisted theme attributes early
if (import.meta.client) {
  restorePersistedTheme()
}

useHead(() => ({
  htmlAttrs: localeHead.value.htmlAttrs,
}))

onMounted(async () => {
  await restorePersistedLocale()
  restorePersistedTheme()
  const cleanupKeyboardShortcuts = registerKeyboardShortcuts()

  // Prevent default browser context menu in desktop shell, preserving input editing
  const handleContextMenu = (event: MouseEvent) => {
    const target = event.target as HTMLElement | null
    if (target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA') {
      return
    }
    event.preventDefault()
  }

  window.addEventListener('contextmenu', handleContextMenu)

  onUnmounted(() => {
    cleanupKeyboardShortcuts()
    window.removeEventListener('contextmenu', handleContextMenu)
  })
})
</script>

<template>
  <UApp>
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
  </UApp>
</template>
