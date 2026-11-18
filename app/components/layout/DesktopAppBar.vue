<script setup lang="ts">
const { t } = useI18n()
const {
  isMaximized,
  isWindowControlsAvailable,
  minimizeWindow,
  toggleMaximizeWindow,
  closeWindow,
} = useWindowControls()

const emit = defineEmits<{
  openSettings: []
}>()

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
    class="flex h-10 shrink-0 select-none items-center justify-between border-b border-default bg-[var(--gv-sidebar)] px-3 text-xs"
    @dblclick="handleBarDoubleClick"
  >
    <!-- Leading: Brand Logo & Title -->
    <div data-tauri-drag-region class="flex items-center gap-2 min-w-0">
      <SharedBrandLogo size="sm" is-decorative />
      <span data-tauri-drag-region class="font-bold text-highlighted tracking-tight">
        {{ t('app.name') }}
      </span>
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
    </div>

    <!-- Trailing: Settings Trigger & Native Window Controls (No theme/lang toggles) -->
    <div class="flex items-center gap-1.5 shrink-0">
      <!-- Settings Button -->
      <UTooltip :text="t('settings.title')">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-settings"
          :aria-label="t('settings.title')"
          @click="emit('openSettings')"
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
