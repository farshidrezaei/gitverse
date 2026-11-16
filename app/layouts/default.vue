<script setup lang="ts">
const { t } = useI18n()
const { locale, changeLocale } = useLocaleController()
const { toggleColorMode, colorMode } = useThemeController()
</script>

<template>
  <div class="flex h-screen w-screen flex-col overflow-hidden bg-(--gv-background)">
    <!-- Top Navigation App Bar -->
    <header class="flex h-12 shrink-0 items-center justify-between border-b border-default bg-(--gv-sidebar) px-4">
      <div class="flex items-center gap-3">
        <SharedBrandLogo size="sm" is-decorative />
        <NuxtLink to="/" class="flex items-center gap-1.5 font-bold text-sm text-highlighted">
          <span>GitVerse</span>
          <UBadge size="xs" color="primary" variant="subtle">v0.1.0</UBadge>
        </NuxtLink>
      </div>

      <nav class="flex items-center gap-1">
        <UButton
          to="/"
          size="sm"
          color="neutral"
          variant="ghost"
          icon="i-lucide-folder-git-2"
          :label="t('navigation.workspace')"
        />
        <UButton
          to="/design-system"
          size="sm"
          color="neutral"
          variant="ghost"
          icon="i-lucide-palette"
          :label="t('navigation.designSystem')"
        />
      </nav>

      <div class="flex items-center gap-2">
        <UButton
          size="xs"
          color="neutral"
          variant="outline"
          icon="i-lucide-languages"
          @click="changeLocale(locale === 'en' ? 'fa' : 'en')"
        >
          {{ locale === 'en' ? 'فارسی' : 'English' }}
        </UButton>
        <UButton
          size="xs"
          color="neutral"
          variant="outline"
          :icon="colorMode.value === 'dark' ? 'i-lucide-moon' : 'i-lucide-sun'"
          @click="toggleColorMode"
        />
      </div>
    </header>

    <!-- Main Content Viewport -->
    <main class="flex min-h-0 flex-1 flex-col overflow-y-auto">
      <slot />
    </main>
  </div>
</template>
