<script setup lang="ts">
const { t } = useI18n()
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
      <NuxtErrorBoundary>
        <NuxtPage />
        <template #error="{ error, clearError }">
          <div class="flex h-full w-full items-center justify-center p-8 bg-[var(--gv-background)]">
            <div class="max-w-md w-full rounded-2xl border border-default/70 bg-[var(--gv-surface)] p-6 shadow-2xl text-center space-y-4">
              <div class="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-error/10 text-error">
                <UIcon name="i-lucide-alert-triangle" class="h-6 w-6" />
              </div>
              <h2 class="text-base font-bold text-highlighted">
                {{ t('common.error') }}
              </h2>
              <p class="text-xs text-muted font-mono bg-default/20 p-3 rounded-lg text-start break-words overflow-auto max-h-32">
                {{ error }}
              </p>
              <div class="flex items-center justify-center gap-3 pt-2">
                <UButton
                  color="primary"
                  variant="solid"
                  icon="i-lucide-rotate-ccw"
                  :label="t('common.refresh')"
                  @click="clearError"
                />
              </div>
            </div>
          </div>
        </template>
      </NuxtErrorBoundary>
    </NuxtLayout>
  </UApp>
</template>
