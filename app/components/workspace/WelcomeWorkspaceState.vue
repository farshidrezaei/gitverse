<script setup lang="ts">
const { t } = useI18n()
const {
  recentRepositories,
  isLoading,
  promptAndOpenRepository,
  openRepository,
  removeRecentRepository,
} = useWorkspaceManager()
const { confirmStandard } = useConfirmationDialog()

async function handleRemoveRecentRepository(
  event: MouseEvent,
  repositoryPath: string,
  repositoryName: string
): Promise<void> {
  event.stopPropagation()
  const isConfirmed = await confirmStandard({
    title: t('repository.removeRecentTitle', { name: repositoryName }),
    description: t('repository.removeRecentDescription', { name: repositoryName }),
    variant: 'danger',
    confirmLabel: t('common.delete'),
    details: [
      { label: t('repository.path'), value: repositoryPath, isMonospace: true },
    ],
  })

  if (isConfirmed) {
    await removeRecentRepository(repositoryPath)
  }
}
</script>

<template>
  <div class="flex h-full w-full flex-col items-center justify-center p-6 sm:p-12 text-center select-none overflow-y-auto">
    <div class="max-w-md w-full space-y-6">
      <!-- Icon & Welcome Header -->
      <div class="flex flex-col items-center space-y-3">
        <div class="h-16 w-16 rounded-2xl bg-primary/10 border border-primary/20 flex items-center justify-center shadow-lg">
          <UIcon name="i-lucide-git-branch" class="h-8 w-8 text-primary" />
        </div>
        <h1 class="text-xl font-bold tracking-tight text-highlighted">
          {{ t('repository.welcomeTitle') }}
        </h1>
        <p class="text-xs text-muted leading-relaxed">
          {{ t('repository.welcomeSubtitle') }}
        </p>
      </div>

      <!-- Primary Action Buttons -->
      <div class="flex flex-col sm:flex-row items-center justify-center gap-3">
        <UButton
          size="sm"
          color="primary"
          icon="i-lucide-folder-open"
          :loading="isLoading"
          class="w-full sm:w-auto font-medium"
          @click="promptAndOpenRepository"
        >
          {{ t('repository.openRepository') }}
        </UButton>
      </div>

      <!-- Recent Repositories Section -->
      <div class="pt-6 border-t border-default/70 text-start space-y-3">
        <div class="flex items-center justify-between text-xs">
          <span class="font-semibold text-highlighted flex items-center gap-1.5">
            <UIcon name="i-lucide-clock" class="h-3.5 w-3.5 text-muted" />
            {{ t('repository.recentRepositories') }}
          </span>
        </div>

        <div v-if="recentRepositories.length > 0" class="space-y-1.5">
          <button
            v-for="repo in recentRepositories"
            :key="repo.id"
            type="button"
            class="group flex w-full items-center justify-between gap-3 rounded-xl border border-default bg-[var(--gv-surface)]/60 px-3 py-2.5 text-xs transition-all hover:border-primary/50 hover:bg-muted/30 focus:outline-none"
            @click="openRepository(repo.path)"
          >
            <div class="flex items-center gap-2.5 min-w-0">
              <UIcon name="i-lucide-folder-git-2" class="h-4 w-4 shrink-0 text-muted group-hover:text-primary transition-colors" />
              <div class="flex flex-col min-w-0 text-start">
                <span class="font-medium text-highlighted truncate">{{ repo.name }}</span>
                <span class="font-mono text-[10px] text-muted truncate" dir="ltr">{{ repo.path }}</span>
              </div>
            </div>
            <div class="flex items-center gap-1 shrink-0">
              <UButton
                size="xs"
                color="neutral"
                variant="ghost"
                icon="i-lucide-trash-2"
                class="opacity-0 group-hover:opacity-100 hover:text-error transition-all"
                :title="t('common.delete')"
                @click.stop="handleRemoveRecentRepository($event, repo.path, repo.name)"
              />
              <UIcon name="i-lucide-chevron-right" class="h-3.5 w-3.5 shrink-0 text-muted group-hover:text-primary rtl:rotate-180 transition-colors" />
            </div>
          </button>
        </div>

        <div
          v-else
          class="rounded-xl border border-dashed border-default/80 p-4 text-center text-xs text-muted"
        >
          {{ t('repository.noRecentRepositories') }}
        </div>
      </div>
    </div>
  </div>
</template>
