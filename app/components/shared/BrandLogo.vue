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
    class="relative inline-flex items-center justify-center shrink-0 transition-transform"
    :class="dimensionClasses"
    :aria-hidden="isDecorative ? 'true' : undefined"
  >
    <!-- GitVerse Official Brand Logo -->
    <img
      src="/logo.png"
      alt="GitVerse Logo"
      class="h-full w-full object-contain drop-shadow-xs select-none"
    />
  </div>
</template>
