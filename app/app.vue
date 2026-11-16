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
})
</script>

<template>
  <UApp>
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
  </UApp>
</template>
