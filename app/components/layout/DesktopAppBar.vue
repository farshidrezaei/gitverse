<script setup lang="ts">
const { t } = useI18n()
const {
  isMaximized,
  isWindowControlsAvailable,
  minimizeWindow,
  toggleMaximizeWindow,
  closeWindow,
} = useWindowControls()
const {
  currentRepository,
  recentRepositories,
  promptAndOpenRepository,
  openRepository,
  closeRepository,
  fetchRecentRepositories,
} = useWorkspaceManager()

const emit = defineEmits<{
  openSettings: []
}>()

onMounted(async () => {
  await fetchRecentRepositories()
})

const isRecentDropdownOpen = ref(false)

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
    <!-- Leading: Brand & Active Repository Context with switch dropdown -->
    <div data-tauri-drag-region class="flex items-center gap-2.5 min-w-0">
      <div data-tauri-drag-region class="flex items-center gap-1.5">
        <SharedBrandLogo size="sm" is-decorative />
        <span data-tauri-drag-region class="font-bold text-highlighted tracking-tight">
          {{ t('app.name') }}
        </span>
      </div>

      <span class="text-muted/60" aria-hidden="true">/</span>

      <!-- Repository Switcher Dropdown -->
      <UDropdownMenu
        v-model:open="isRecentDropdownOpen"
        :items="[
          [
            {
              label: t('repository.openRepository'),
              icon: 'i-lucide-folder-open',
              onSelect: () => promptAndOpenRepository(),
            },
          ],
          recentRepositories.length > 0
            ? recentRepositories.map(repo => ({
                label: repo.name,
                icon: 'i-lucide-git-fork',
                description: repo.path,
                onSelect: () => openRepository(repo.path),
              }))
            : [
                {
                  label: t('repository.noRecentRepositories'),
                  disabled: true,
                  icon: 'i-lucide-clock',
                },
              ],
          currentRepository
            ? [
                {
                  label: t('repository.closeRepository'),
                  icon: 'i-lucide-x',
                  color: 'error' as const,
                  onSelect: () => closeRepository(),
                },
              ]
            : [],
        ]"
      >
        <button
          type="button"
          class="flex items-center gap-1.5 rounded-lg px-2 py-1 text-xs font-medium transition-colors hover:bg-muted/40 focus:outline-none"
          :class="currentRepository ? 'text-highlighted' : 'text-muted'"
        >
          <UIcon
            :name="currentRepository ? 'i-lucide-folder-git-2' : 'i-lucide-folder-dashed'"
            class="h-3.5 w-3.5"
            :class="currentRepository ? 'text-primary' : 'text-muted'"
          />
          <span class="truncate max-w-[140px] sm:max-w-[200px]">
            {{ currentRepository ? currentRepository.name : t('repository.noActiveRepository') }}
          </span>
          <UBadge
            v-if="currentRepository?.current_branch_name"
            size="xs"
            color="primary"
            variant="subtle"
            icon="i-lucide-git-branch"
            class="hidden sm:inline-flex"
          >
            {{ currentRepository.current_branch_name }}
          </UBadge>
          <UIcon name="i-lucide-chevrons-up-down" class="h-3 w-3 text-muted" />
        </button>
      </UDropdownMenu>
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

    <!-- Trailing: Settings Trigger & Native Window Controls (No theme/lang toggles) -->
    <div class="flex items-center gap-1.5 shrink-0">
      <!-- Quick Open Repository Button -->
      <UTooltip :text="t('repository.openRepository')">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-folder-open"
          :aria-label="t('repository.openRepository')"
          @click="promptAndOpenRepository"
        />
      </UTooltip>

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
