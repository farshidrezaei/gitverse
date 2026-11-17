<script setup lang="ts">
interface CommitHashPillProps {
  hash: string
  length?: number
  showCopy?: boolean
}

const props = withDefaults(defineProps<CommitHashPillProps>(), {
  length: 7,
  showCopy: true,
})

const { t } = useI18n()
const isCopied = ref<boolean>(false)

const truncatedHash = computed(() => {
  return props.hash.slice(0, props.length)
})

async function copyHashToClipboard(): Promise<void> {
  if (!navigator?.clipboard) return
  try {
    await navigator.clipboard.writeText(props.hash)
    isCopied.value = true
    setTimeout(() => {
      isCopied.value = false
    }, 1500)
  } catch (error) {
    console.error('Failed to copy commit hash:', error)
  }
}
</script>

<template>
  <span
    class="inline-flex items-center gap-1 font-mono text-xs px-1.5 py-0.5 rounded bg-surface-subtle text-toned border border-border-subtle group hover:border-primary/40 transition-colors"
    :title="props.hash"
  >
    <span>{{ truncatedHash }}</span>
    <button
      v-if="props.showCopy"
      type="button"
      class="opacity-0 group-hover:opacity-100 transition-opacity text-muted hover:text-default"
      :title="t('git.copyHash')"
      @click.stop="copyHashToClipboard"
    >
      <UIcon
        :name="isCopied ? 'i-lucide-check' : 'i-lucide-copy'"
        class="w-3 h-3"
        :class="{ 'text-success': isCopied }"
      />
    </button>
  </span>
</template>
