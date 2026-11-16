<script setup lang="ts">
const localeHead = useLocaleHead()
const { restorePersistedLocale } = useLocaleController()
const { restorePersistedTheme } = useThemeController()

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
