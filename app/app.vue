<script setup lang="ts">
const localeHead = useLocaleHead()
const { restorePersistedLocale } = useLocaleController()
const { restorePersistedTheme } = useThemeController()

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

  // Prevent default browser context menu in desktop shell, preserving input editing
  window.addEventListener('contextmenu', (event: MouseEvent) => {
    const target = event.target as HTMLElement | null
    if (target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA') {
      return
    }
    event.preventDefault()
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
