<script setup lang="ts">
interface FileStatusBadgeProps {
  status: string
  size?: 'xs' | 'sm' | 'md'
}

const props = withDefaults(defineProps<FileStatusBadgeProps>(), {
  size: 'sm',
})

const badgeConfig = computed(() => {
  switch (props.status.toLowerCase()) {
    case 'modified':
      return {
        label: 'M',
        colorClass: 'text-warning border-warning/30 bg-warning/10',
      }
    case 'added':
      return {
        label: 'A',
        colorClass: 'text-success border-success/30 bg-success/10',
      }
    case 'deleted':
      return {
        label: 'D',
        colorClass: 'text-error border-error/30 bg-error/10',
      }
    case 'renamed':
      return {
        label: 'R',
        colorClass: 'text-primary border-primary/30 bg-primary/10',
      }
    case 'copied':
      return {
        label: 'C',
        colorClass: 'text-info border-info/30 bg-info/10',
      }
    case 'untracked':
      return {
        label: '?',
        colorClass: 'text-muted border-muted/30 bg-muted/10',
      }
    case 'conflicted':
      return {
        label: '!',
        colorClass: 'text-error font-extrabold border-error/50 bg-error/20',
      }
    default:
      return {
        label: props.status[0]?.toUpperCase() || 'M',
        colorClass: 'text-toned border-border-subtle bg-surface-subtle',
      }
  }
})

const sizeClass = computed(() => {
  switch (props.size) {
    case 'xs':
      return 'w-3.5 h-3.5 text-[9px]'
    case 'md':
      return 'w-5 h-5 text-xs'
    case 'sm':
    default:
      return 'w-4 h-4 text-[10px]'
  }
})
</script>

<template>
  <span
    class="inline-flex items-center justify-center font-mono font-bold rounded shrink-0 border select-none transition-colors"
    :class="[badgeConfig.colorClass, sizeClass]"
    :title="props.status"
  >
    {{ badgeConfig.label }}
  </span>
</template>
