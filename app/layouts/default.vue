<script setup lang="ts">
// Layout for frameless edge-to-edge desktop application
</script>

<template>
  <div class="flex h-screen w-screen flex-col overflow-hidden bg-[var(--gv-background)] text-[var(--ui-text)]">
    <!-- Frameless Window Custom App Bar -->
    <LayoutDesktopAppBar />

    <!-- Edge-to-Edge Full-Window Viewport (No fixed width container) -->
    <main class="flex min-h-0 flex-1 flex-col overflow-hidden w-full">
      <slot />
    </main>
  </div>
</template>
