<script setup lang="ts">
const { t } = useI18n()

const gitStatusTokens = [
  { label: 'Staged', key: 'git.status.staged', color: 'var(--gv-git-staged)', icon: 'i-lucide-check-circle-2' },
  { label: 'Unstaged', key: 'git.status.unstaged', color: 'var(--gv-git-unstaged)', icon: 'i-lucide-circle-dot' },
  { label: 'Conflicted', key: 'git.status.conflicted', color: 'var(--gv-git-conflicted)', icon: 'i-lucide-alert-triangle' },
  { label: 'Untracked', key: 'git.status.untracked', color: 'var(--gv-git-untracked)', icon: 'i-lucide-help-circle' },
  { label: 'HEAD', key: 'git.branch', color: 'var(--gv-git-head)', icon: 'i-lucide-git-commit' },
  { label: 'Tag', key: 'git.tag', color: 'var(--gv-git-tag)', icon: 'i-lucide-tag' },
]

const surfaceTokens = [
  { label: 'Background', cssVar: '--gv-background' },
  { label: 'Sidebar', cssVar: '--gv-sidebar' },
  { label: 'Surface', cssVar: '--gv-surface' },
  { label: 'Elevated Surface', cssVar: '--gv-surface-elevated' },
  { label: 'Border', cssVar: '--ui-border' },
]
</script>

<template>
  <UCard>
    <template #header>
      <SharedSectionHeader
        :title="t('designSystem.sections.colors')"
        :description="t('designSystem.descriptions.colors')"
      />
    </template>

    <div class="flex flex-col gap-6">
      <!-- Surface Tokens -->
      <div class="flex flex-col gap-2">
        <h3 class="text-xs font-semibold uppercase tracking-wider text-muted">
          Application Surfaces
        </h3>
        <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-5">
          <div
            v-for="item in surfaceTokens"
            :key="item.label"
            class="flex flex-col gap-2 rounded-lg border border-default p-3 shadow-xs"
            :style="{ backgroundColor: `var(${item.cssVar})` }"
          >
            <span class="text-xs font-medium text-highlighted">{{ item.label }}</span>
            <span class="font-mono text-[11px] text-muted" dir="ltr">{{ item.cssVar }}</span>
          </div>
        </div>
      </div>

      <!-- Git Status Tokens -->
      <div class="flex flex-col gap-2">
        <h3 class="text-xs font-semibold uppercase tracking-wider text-muted">
          {{ t('designSystem.statusTokens') }}
        </h3>
        <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-6">
          <div
            v-for="token in gitStatusTokens"
            :key="token.label"
            class="flex items-center gap-2.5 rounded-lg border border-default bg-surface p-3"
          >
            <div
              class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-white"
              :style="{ backgroundColor: token.color }"
            >
              <UIcon :name="token.icon" class="h-4 w-4" />
            </div>
            <div class="min-w-0 flex flex-col">
              <span class="truncate text-xs font-medium text-highlighted">{{ t(token.key) }}</span>
              <span class="truncate text-[10px] text-muted">{{ token.label }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Diff & Patch Preview -->
      <div class="flex flex-col gap-2">
        <h3 class="text-xs font-semibold uppercase tracking-wider text-muted">
          {{ t('designSystem.diffTokens') }}
        </h3>
        <div class="overflow-hidden rounded-lg border border-default font-mono text-xs" dir="ltr">
          <div class="bg-[var(--gv-diff-hunk-bg)] text-[var(--gv-diff-hunk-text)] px-3 py-1.5 font-medium">
            @@ -42,6 +42,7 @@ export function resolveGraphLayout(dag: CommitDag): GraphLayout
          </div>
          <div class="bg-[var(--gv-diff-remove-bg)] text-[var(--gv-diff-remove-text)] px-3 py-1 border-s-2 border-[var(--gv-diff-remove-border)]">
            - const previousLane = computeFallbackLane(commit);
          </div>
          <div class="bg-[var(--gv-diff-add-bg)] text-[var(--gv-diff-add-text)] px-3 py-1 border-s-2 border-[var(--gv-diff-add-border)]">
            + const assignedLane = deterministicLaneAssignment(commit, parentLanes);
          </div>
          <div class="bg-[var(--gv-diff-add-bg)] text-[var(--gv-diff-add-text)] px-3 py-1 border-s-2 border-[var(--gv-diff-add-border)]">
            + recordLaneAllocation(assignedLane, commit.sha);
          </div>
        </div>
      </div>
    </div>
  </UCard>
</template>
