<script setup lang="ts">
// Zero-CLS high-fidelity skeleton for diff loading state
</script>

<template>
  <div class="min-w-fit p-0 font-mono text-[11px] leading-5">
    <!-- Skeleton Hunk Header -->
    <div class="flex items-center bg-[var(--gv-diff-hunk-bg)]/40 px-3 py-1 border-y border-default/40">
      <USkeleton class="h-3 w-48 rounded" />
    </div>

    <!-- Skeleton Diff Lines -->
    <div class="space-y-0.5 py-1">
      <div
        v-for="index in 8"
        :key="`diff-skeleton-${index}`"
        class="flex items-center h-5 px-1 animate-pulse"
        :class="{
          'bg-[var(--gv-diff-remove-bg)]/40': index === 2 || index === 3,
          'bg-[var(--gv-diff-add-bg)]/40': index === 4 || index === 5 || index === 6,
        }"
      >
        <!-- Old Line Number -->
        <div class="w-12 pe-2 text-end">
          <USkeleton class="h-2.5 w-5 ms-auto rounded" />
        </div>
        <!-- New Line Number -->
        <div class="w-12 pe-2 text-end">
          <USkeleton class="h-2.5 w-5 ms-auto rounded" />
        </div>
        <!-- Gutter Prefix -->
        <div class="w-5 text-center text-muted font-bold">
          <span v-if="index === 2 || index === 3">-</span>
          <span v-else-if="index >= 4 && index <= 6">+</span>
        </div>
        <!-- Line Content Skeleton -->
        <div class="flex-1 pe-4">
          <USkeleton
            class="h-2.5 rounded"
            :class="[
              index === 1 ? 'w-2/5' :
              index === 2 ? 'w-3/5' :
              index === 3 ? 'w-1/2' :
              index === 4 ? 'w-4/5' :
              index === 5 ? 'w-2/3' :
              index === 6 ? 'w-3/4' : 'w-1/3'
            ]"
          />
        </div>
      </div>
    </div>
  </div>
</template>
