<script setup lang="ts">
const { t } = useI18n()

const gitExecutablePath = ref('/usr/bin/git')
const defaultBranchName = ref('main')
const autoFetchIntervalMinutes = ref('10')
const isPruneOnFetch = ref(true)
const isAutoStashOnRebase = ref(true)
</script>

<template>
  <div class="space-y-6 text-xs">
    <UCard>
      <template #header>
        <div class="space-y-0.5">
          <h3 class="text-sm font-semibold text-highlighted">
            {{ t('settings.git.title') }}
          </h3>
          <p class="text-[11px] text-muted">
            {{ t('settings.git.description') }} (ADR-001)
          </p>
        </div>
      </template>

      <div class="space-y-4">
        <!-- Executable Path -->
        <div class="flex flex-col gap-1.5">
          <label class="font-medium text-toned">{{ t('settings.git.executablePath') }}</label>
          <div class="flex items-center gap-2">
            <UInput v-model="gitExecutablePath" icon="i-lucide-terminal" class="flex-1 font-mono text-xs" dir="ltr" />
            <UButton size="xs" color="neutral" variant="outline" label="Detect" />
          </div>
          <p class="text-[10px] text-muted">
            Discovered native system executable: <span class="font-mono text-success">git 2.43.0</span>.
          </p>
        </div>

        <!-- Default Branch -->
        <div class="flex flex-col gap-1.5">
          <label class="font-medium text-toned">{{ t('settings.git.defaultBranch') }}</label>
          <UInput v-model="defaultBranchName" icon="i-lucide-git-branch" class="w-64 font-mono text-xs" dir="ltr" />
        </div>

        <!-- Prune on fetch -->
        <div class="flex items-center justify-between border-t border-default/60 pt-3">
          <div class="space-y-0.5">
            <span class="font-medium text-highlighted block">Prune remote references on fetch</span>
            <span class="text-[11px] text-muted block">Automatically remove local tracking branches that no longer exist on remote.</span>
          </div>
          <USwitch v-model="isPruneOnFetch" size="sm" />
        </div>

        <!-- Auto-stash on rebase -->
        <div class="flex items-center justify-between border-t border-default/60 pt-3">
          <div class="space-y-0.5">
            <span class="font-medium text-highlighted block">Auto-stash on pull with rebase</span>
            <span class="text-[11px] text-muted block">Automatically stash working tree before rebasing and reapply afterward.</span>
          </div>
          <USwitch v-model="isAutoStashOnRebase" size="sm" />
        </div>
      </div>
    </UCard>
  </div>
</template>
