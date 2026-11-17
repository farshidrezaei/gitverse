<script setup lang="ts">
interface CommitGraphToolbarProps {
  totalCommitsCount?: number | null
  activeReference?: string | null
  isOperating?: boolean
  isLoading?: boolean
  searchQuery: string
}

const props = defineProps<CommitGraphToolbarProps>()

const emit = defineEmits<{
  'update:searchQuery': [value: string]
  resetGraphFilter: []
  push: []
  pull: []
  fetch: []
  refresh: []
  createBranch: []
}>()

const { t } = useI18n()
</script>

<template>
  <div class="flex h-10 shrink-0 items-center justify-between border-b border-default/70 bg-muted/20 px-3 gap-2">
    <div class="flex items-center gap-1.5">
      <span class="text-xs font-semibold text-highlighted uppercase tracking-wider flex items-center gap-1.5 me-1">
        <UIcon name="i-lucide-git-commit-horizontal" class="h-4 w-4 text-primary" />
        {{ t('navigation.graph') }}
      </span>

      <!-- Active Branch Filter Chip or All Branches Indicator -->
      <UBadge
        v-if="props.activeReference"
        size="xs"
        color="primary"
        variant="subtle"
        class="font-mono text-[10px] flex items-center gap-1 py-0.5 me-1"
      >
        <UIcon name="i-lucide-git-branch" class="h-3 w-3 shrink-0" />
        <span class="max-w-[120px] truncate" :title="props.activeReference">{{ props.activeReference }}</span>
        <button
          type="button"
          class="text-muted hover:text-highlighted ms-0.5 p-0.5 rounded cursor-pointer transition-colors leading-none"
          :title="t('git.showAllBranches')"
          @click="emit('resetGraphFilter')"
        >
          <UIcon name="i-lucide-x" class="h-2.5 w-2.5" />
        </button>
      </UBadge>
      <span v-else class="text-[10px] text-muted font-medium me-1">
        {{ t('git.allBranches') }}
      </span>

      <UBadge
        v-if="props.totalCommitsCount !== undefined"
        size="xs"
        color="neutral"
        variant="subtle"
        class="font-mono text-[10px] me-2"
      >
        {{ props.totalCommitsCount }}
      </UBadge>

      <div class="h-3.5 w-px bg-default/80 me-1" aria-hidden="true" />

      <!-- Remote Git Actions -->
      <UButton
        size="xs"
        color="primary"
        variant="solid"
        icon="i-lucide-arrow-up-circle"
        :loading="props.isOperating"
        :label="t('git.push')"
        @click="emit('push')"
      />
      <UButton
        size="xs"
        color="neutral"
        variant="outline"
        icon="i-lucide-arrow-down-circle"
        :loading="props.isOperating"
        :label="t('git.pull')"
        @click="emit('pull')"
      />
      <UButton
        size="xs"
        color="neutral"
        variant="ghost"
        icon="i-lucide-refresh-cw"
        :loading="props.isOperating"
        :label="t('git.fetch')"
        @click="emit('fetch')"
      />
    </div>

    <!-- Quick Search & Actions -->
    <div class="flex items-center gap-1.5">
      <UInput
        :model-value="props.searchQuery"
        size="xs"
        icon="i-lucide-search"
        :placeholder="t('git.searchCommits')"
        class="w-48"
        @update:model-value="emit('update:searchQuery', $event)"
      />
      <UButton
        size="xs"
        color="neutral"
        variant="ghost"
        icon="i-lucide-rotate-cw"
        :loading="props.isLoading"
        :title="t('common.refresh')"
        @click="emit('refresh')"
      />
      <UButton
        size="xs"
        color="neutral"
        variant="outline"
        icon="i-lucide-plus"
        :title="t('branch.create')"
        @click="emit('createBranch')"
      />
    </div>
  </div>
</template>
