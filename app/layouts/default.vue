<script setup lang="ts">
import CommandPaletteModal from '~/components/workspace/CommandPaletteModal.vue'
import WorkspaceBottomPanel from '~/components/workspace/panel/WorkspaceBottomPanel.vue'
import { useGitOutputLog } from '~/composables/useGitOutputLog'
import { useAppUpdater } from '~/composables/useAppUpdater'

const isSettingsOpen = ref<boolean>(false)
const { isCommandPaletteOpen, openCommandPalette, toggleCommandPalette } = useCommandPalette()
const { toggleBottomPanel } = useGitOutputLog()
const { runBackgroundUpdateCheck } = useAppUpdater()

onMounted(() => {
  void runBackgroundUpdateCheck()

  const handleKeyDown = (event: KeyboardEvent) => {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') {
      event.preventDefault()
      toggleCommandPalette()
    } else if ((event.ctrlKey || event.metaKey) && event.key === '`') {
      event.preventDefault()
      toggleBottomPanel()
    }
  }

  window.addEventListener('keydown', handleKeyDown)

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeyDown)
  })
})
</script>

<template>
  <div class="flex h-screen w-screen flex-col overflow-hidden bg-[var(--gv-background)] text-[var(--ui-text)]">
    <!-- Frameless Window Custom App Bar -->
    <LayoutDesktopAppBar
      @open-settings="isSettingsOpen = true"
      @open-command-palette="openCommandPalette"
    />

    <!-- Edge-to-Edge Full-Window Viewport -->
    <main class="flex min-h-0 flex-1 flex-col overflow-hidden w-full">
      <slot />
    </main>

    <!-- Bottom Drawer Panel (Terminal & Git Logs) -->
    <WorkspaceBottomPanel />

    <!-- Desktop Status Footer -->
    <LayoutDesktopStatusFooter @open-settings="isSettingsOpen = true" />

    <!-- Global Settings Modal (Theme & Language control) -->
    <SettingsModal v-model:open="isSettingsOpen" />

    <!-- Global Command Palette Modal (Ctrl+K) -->
    <CommandPaletteModal
      v-model:open="isCommandPaletteOpen"
      @open-settings="isSettingsOpen = true"
    />
  </div>
</template>
