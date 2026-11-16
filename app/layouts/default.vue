<script setup lang="ts">
const isSettingsOpen = ref(false)
</script>

<template>
  <div class="flex h-screen w-screen flex-col overflow-hidden bg-[var(--gv-background)] text-[var(--ui-text)]">
    <!-- Frameless Window Custom App Bar -->
    <LayoutDesktopAppBar @open-settings="isSettingsOpen = true" />

    <!-- Edge-to-Edge Full-Window Viewport -->
    <main class="flex min-h-0 flex-1 flex-col overflow-hidden w-full">
      <slot />
    </main>

    <!-- Desktop Status Footer -->
    <LayoutDesktopStatusFooter @open-settings="isSettingsOpen = true" />

    <!-- Global Settings Modal (Theme & Language control) -->
    <SettingsModal v-model:open="isSettingsOpen" />
  </div>
</template>
