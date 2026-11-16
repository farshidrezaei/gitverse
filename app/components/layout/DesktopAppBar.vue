<script setup lang="ts">
const { t } = useI18n()
const { locale, changeLocale } = useLocaleController()
const { colorMode, toggleColorMode } = useThemeController()
const {
  isMaximized,
  isWindowControlsAvailable,
  minimizeWindow,
  toggleMaximizeWindow,
  closeWindow,
} = useWindowControls()

function handleBarDoubleClick(event: MouseEvent): void {
  const target = event.target as HTMLElement | null
  if (target?.closest('button, a, input, select, [role="button"]')) {
    return
  }
  if (isWindowControlsAvailable.value) {
    void toggleMaximizeWindow()
  }
}
</script>

<template>
  <header
    data-tauri-drag-region
    class="flex h-10 shrink-0 select-none items-center justify-between border-b border-default bg-[var(--gv-sidebar)] px-2.5 text-xs"
    @dblclick="handleBarDoubleClick"
  >
    <!-- Leading: Brand & Active Context -->
    <div data-tauri-drag-region class="flex items-center gap-2.5 min-w-0">
      <div data-tauri-drag-region class="flex items-center gap-1.5">
        <SharedBrandLogo size="sm" is-decorative />
        <span data-tauri-drag-region class="font-bold text-highlighted tracking-tight">
          {{ t('app.name') }}
        </span>
      </div>

      <span class="text-muted/60" aria-hidden="true">/</span>

      <!-- Active Workspace / Repository Breadcrumb -->
      <div data-tauri-drag-region class="flex items-center gap-1.5 text-muted truncate">
        <span class="font-medium text-toned truncate">gitverse</span>
        <UBadge size="xs" color="primary" variant="subtle" icon="i-lucide-git-branch">
          main
        </UBadge>
      </div>

      <!-- Navigation Links -->
      <nav class="flex items-center gap-1 ms-2">
        <UButton
          to="/"
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-git-graph"
          :label="t('navigation.workspace')"
        />
        <UButton
          to="/design-system"
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-palette"
          :label="t('navigation.designSystem')"
        />
      </nav>
    </div>

    <!-- Center Drag Region -->
    <div data-tauri-drag-region class="flex-1 h-full flex items-center justify-center">
      <span
        v-if="!isWindowControlsAvailable"
        data-tauri-drag-region
        class="text-[10px] text-muted tracking-wider uppercase opacity-75"
      >
        {{ t('application.status.browserPreviewMode') }}
      </span>
    </div>

    <!-- Trailing: Language, Theme & Frameless Window Controls -->
    <div class="flex items-center gap-1 shrink-0">
      <UTooltip :text="t('designSystem.controls.languageToggle')">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-languages"
          @click="changeLocale(locale === 'en' ? 'fa' : 'en')"
        >
          {{ locale === 'en' ? 'FA' : 'EN' }}
        </UButton>
      </UTooltip>

      <UTooltip :text="t('common.theme')">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          :icon="colorMode.value === 'dark' ? 'i-lucide-moon' : 'i-lucide-sun'"
          @click="toggleColorMode"
        />
      </UTooltip>

      <div class="mx-1 h-3.5 w-px bg-default" aria-hidden="true" />

      <!-- Native Frameless Window Buttons (LTR order: Min, Max, Close) -->
      <div class="flex items-center gap-0.5" dir="ltr">
        <UTooltip :text="t('window.minimize')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            square
            icon="i-lucide-minus"
            :disabled="!isWindowControlsAvailable"
            :aria-label="t('window.minimize')"
            @click="minimizeWindow"
          />
        </UTooltip>

        <UTooltip :text="isMaximized ? t('window.restore') : t('window.maximize')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            square
            :icon="isMaximized ? 'i-lucide-copy' : 'i-lucide-square'"
            :disabled="!isWindowControlsAvailable"
            :aria-label="isMaximized ? t('window.restore') : t('window.maximize')"
            @click="toggleMaximizeWindow"
          />
        </UTooltip>

        <UTooltip :text="t('window.close')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            square
            icon="i-lucide-x"
            class="hover:bg-error/20 hover:text-error"
            :disabled="!isWindowControlsAvailable"
            :aria-label="t('window.close')"
            @click="closeWindow"
          />
        </UTooltip>
      </div>
    </div>
  </header>
</template>
