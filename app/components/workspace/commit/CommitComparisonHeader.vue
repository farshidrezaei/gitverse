<script setup lang="ts">
import type { CommitComparison } from '~/types/repository'
import CommitHashPill from '~/components/shared/CommitHashPill.vue'

interface CommitComparisonHeaderProps {
  comparison: CommitComparison
}

const props = defineProps<CommitComparisonHeaderProps>()

const emit = defineEmits<{
  swap: []
  close: []
}>()

const { t } = useI18n()
</script>

<template>
  <div>
    <!-- Top Bar: Back button, Title, Swap & Close -->
    <div class="flex h-9 shrink-0 items-center justify-between border-b border-default/70 px-3 bg-muted/20">
      <div class="flex items-center gap-1.5 min-w-0">
        <UTooltip :text="t('commit.closeComparison')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-arrow-left"
            class="text-toned hover:text-highlighted rtl:rotate-180"
            :aria-label="t('commit.closeComparison')"
            @click="emit('close')"
          />
        </UTooltip>
        <div class="h-3.5 w-px bg-default/80" aria-hidden="true" />
        <span class="text-xs font-semibold text-highlighted uppercase tracking-wider truncate">
          {{ t('commit.comparison') }}
        </span>
      </div>

      <div class="flex items-center gap-1 shrink-0">
        <UTooltip :text="t('commit.swapComparison')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-arrow-left-right"
            :aria-label="t('commit.swapComparison')"
            @click="emit('swap')"
          />
        </UTooltip>
        <UTooltip :text="t('commit.closeComparison')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-x"
            :aria-label="t('commit.closeComparison')"
            @click="emit('close')"
          />
        </UTooltip>
      </div>
    </div>

    <!-- Comparison Details Card -->
    <div class="p-3 border-b border-default/70 space-y-2.5 bg-surface/40">
      <!-- Base & Target Commits Comparison Cards -->
      <div class="space-y-1.5">
        <!-- Base Commit -->
        <div class="flex items-center gap-2 p-1.5 rounded-md bg-muted/20 border border-default/30">
          <span class="text-[10px] font-bold uppercase text-muted tracking-wider w-12 shrink-0">
            Base
          </span>
          <CommitHashPill :hash="props.comparison.base_hash" class="shrink-0" />
          <span
            v-if="props.comparison.base_subject"
            class="text-[11px] text-highlighted truncate flex-1"
            :title="props.comparison.base_subject"
          >
            {{ props.comparison.base_subject }}
          </span>
        </div>

        <!-- Target Commit -->
        <div class="flex items-center gap-2 p-1.5 rounded-md bg-primary/10 border border-primary/20">
          <span class="text-[10px] font-bold uppercase text-primary tracking-wider w-12 shrink-0">
            Target
          </span>
          <CommitHashPill :hash="props.comparison.target_hash" class="shrink-0" />
          <span
            v-if="props.comparison.target_subject"
            class="text-[11px] text-highlighted font-medium truncate flex-1"
            :title="props.comparison.target_subject"
          >
            {{ props.comparison.target_subject }}
          </span>
        </div>
      </div>

      <!-- Stats Pill -->
      <div class="flex items-center gap-2 pt-0.5">
        <UBadge size="xs" color="neutral" variant="subtle" class="font-mono">
          {{ props.comparison.changed_files.length }} {{ t('commit.filesChanged') }}
        </UBadge>
        <UBadge
          v-if="props.comparison.total_additions > 0"
          size="xs"
          color="success"
          variant="soft"
          class="font-mono"
        >
          +{{ props.comparison.total_additions }}
        </UBadge>
        <UBadge
          v-if="props.comparison.total_deletions > 0"
          size="xs"
          color="error"
          variant="soft"
          class="font-mono"
        >
          -{{ props.comparison.total_deletions }}
        </UBadge>
      </div>
    </div>
  </div>
</template>
