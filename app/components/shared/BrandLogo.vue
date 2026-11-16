<script setup lang="ts">
const properties = withDefaults(
  defineProps<{
    size?: 'sm' | 'md' | 'lg'
    isDecorative?: boolean
  }>(),
  {
    size: 'md',
    isDecorative: false,
  },
)

const dimensionClasses = computed(() => {
  switch (properties.size) {
    case 'sm':
      return 'h-6 w-6'
    case 'lg':
      return 'h-10 w-10'
    case 'md':
    default:
      return 'h-8 w-8'
  }
})
</script>

<template>
  <div
    class="relative inline-flex items-center justify-center shrink-0 rounded-lg bg-primary/10 text-primary transition-colors"
    :class="dimensionClasses"
    :aria-hidden="isDecorative ? 'true' : undefined"
  >
    <!-- GitVerse Commit Convergence Logo Icon -->
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      class="h-3/4 w-3/4"
    >
      <circle cx="6" cy="6" r="3" />
      <circle cx="6" cy="18" r="3" />
      <circle cx="18" cy="12" r="3" />
      <line x1="6" y1="9" x2="6" y2="15" />
      <path d="M6 9a9 9 0 0 1 9 3" />
      <line x1="15" y1="12" x2="18" y2="12" />
    </svg>
  </div>
</template>
