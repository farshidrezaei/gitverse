<script setup lang="ts">
import type { TagSummary } from '~/types/repository'

interface ExplorerTagsSectionProps {
  tags: TagSummary[] | undefined
}

const props = defineProps<ExplorerTagsSectionProps>()
const { t } = useI18n()
</script>

<template>
  <div class="flex flex-col gap-2 p-2 text-xs">
    <div class="flex items-center justify-between pb-1 border-b border-default/40">
      <span class="text-[11px] font-semibold text-muted uppercase tracking-wider">
        {{ t('git.tags') }}
      </span>
      <span class="text-[10px] text-muted">{{ props.tags?.length || 0 }}</span>
    </div>

    <div v-if="props.tags?.length" class="space-y-0.5">
      <div
        v-for="tag in props.tags"
        :key="tag.name"
        class="flex w-full items-center gap-1.5 rounded-lg px-2 py-1 text-start text-toned text-[11px] font-mono hover:bg-primary/10 transition-colors"
      >
        <UIcon name="i-lucide-tag" class="h-3.5 w-3.5 shrink-0 text-muted" />
        <span class="truncate">{{ tag.name }}</span>
      </div>
    </div>

    <div v-else class="text-[11px] text-muted italic px-1 py-1">
      {{ t('git.noTags') }}
    </div>
  </div>
</template>
