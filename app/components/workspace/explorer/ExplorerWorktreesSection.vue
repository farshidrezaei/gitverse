<script setup lang="ts">
import { useWorktreeController } from '~/composables/useWorktreeController'

const emit = defineEmits<{
  openCreateWorktreeModal: []
  openWorktreeInTab: [directoryPath: string]
}>()

const { t } = useI18n()
const { currentRepository } = useRepositoryWorkspace()
const { worktreeList, isLoadingWorktrees, fetchWorktrees, removeWorktree } = useWorktreeController()

watch(
  () => currentRepository.value?.path,
  () => {
    void fetchWorktrees()
  },
  { immediate: true },
)

async function handleRemoveWorktree(worktreePath: string): Promise<void> {
  await removeWorktree(worktreePath)
}
</script>

<template>
  <div class="flex flex-col gap-2 p-2 text-xs">
    <div class="flex items-center justify-between pb-1 border-b border-default/40">
      <span class="text-[11px] font-semibold text-muted uppercase tracking-wider">
        {{ t('worktree.title') }}
      </span>
      <div class="flex items-center gap-1">
        <span class="text-[10px] text-muted">{{ worktreeList.length }}</span>
        <UTooltip :text="t('worktree.createTitle')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-plus"
            class="h-5 w-5 p-0"
            :aria-label="t('worktree.createTitle')"
            @click="emit('openCreateWorktreeModal')"
          />
        </UTooltip>
      </div>
    </div>

    <!-- Skeletons Loading -->
    <div v-if="isLoadingWorktrees && worktreeList.length === 0" class="space-y-1 mt-1">
      <div v-for="wIndex in 2" :key="`worktree-skeleton-${wIndex}`" class="flex items-center justify-between rounded-lg px-2 py-1">
        <div class="flex items-center gap-1.5">
          <USkeleton class="h-3.5 w-3.5 rounded shrink-0" />
          <USkeleton class="h-3 w-28 rounded" />
        </div>
        <USkeleton class="h-4 w-10 rounded" />
      </div>
    </div>

    <!-- Worktrees List -->
    <div v-else-if="worktreeList.length > 0" class="flex flex-col gap-0.5">
      <div
        v-for="worktree in worktreeList"
        :key="worktree.directory_path"
        class="group flex w-full items-center justify-between rounded-lg px-2 py-1 text-toned text-[11px] font-mono hover:bg-primary/10 transition-colors cursor-pointer"
        :class="worktree.directory_path === currentRepository?.path ? 'bg-primary/15 text-primary font-semibold' : ''"
        @click="emit('openWorktreeInTab', worktree.directory_path)"
      >
        <div class="flex items-center gap-1.5 min-w-0">
          <UIcon name="i-lucide-git-fork" class="h-3.5 w-3.5 shrink-0 text-primary" />
          <span class="truncate" :title="worktree.directory_path">
            {{ worktree.branch_name || worktree.head_commit_hash.slice(0, 7) }}
          </span>
        </div>
        <div class="flex items-center gap-1 shrink-0">
          <UBadge v-if="worktree.is_main" size="xs" color="primary" variant="subtle">Main</UBadge>
          <UBadge v-else-if="worktree.is_locked" size="xs" color="warning" variant="subtle">Locked</UBadge>
          <button
            v-if="!worktree.is_main"
            type="button"
            class="rounded p-0.5 text-muted opacity-0 group-hover:opacity-100 hover:bg-error/20 hover:text-error transition-all"
            :title="t('common.delete')"
            @click.stop="handleRemoveWorktree(worktree.directory_path)"
          >
            <UIcon name="i-lucide-trash-2" class="h-3 w-3" />
          </button>
        </div>
      </div>
    </div>

    <div v-else class="text-[11px] text-muted italic px-1 py-1">
      {{ t('worktree.noWorktrees') }}
    </div>
  </div>
</template>
