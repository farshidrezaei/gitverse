<script setup lang="ts">
import type { CommitDetails } from '~/types/repository'
import CommitHashPill from '~/components/shared/CommitHashPill.vue'

interface CommitDetailsHeaderProps {
  commitDetails: CommitDetails
}

const props = defineProps<CommitDetailsHeaderProps>()

const emit = defineEmits<{
  createBranch: [commitSha: string]
  checkoutCommit: [commitSha: string]
  close: []
}>()

const { t } = useI18n()

function formatRelativeTime(timestampInSeconds: number): string {
  const diffMilliseconds = Date.now() - timestampInSeconds * 1000
  const diffSeconds = Math.max(0, Math.floor(diffMilliseconds / 1000))
  if (diffSeconds < 60) return `${diffSeconds}s ago`
  const diffMinutes = Math.floor(diffSeconds / 60)
  if (diffMinutes < 60) return `${diffMinutes}m ago`
  const diffHours = Math.floor(diffMinutes / 60)
  if (diffHours < 24) return `${diffHours}h ago`
  const diffDays = Math.floor(diffHours / 24)
  if (diffDays < 30) return `${diffDays}d ago`
  return new Date(timestampInSeconds * 1000).toLocaleDateString()
}
</script>

<template>
  <div>
    <!-- Panel Top Bar: Back button & action shortcuts -->
    <div class="flex h-9 shrink-0 items-center justify-between border-b border-default/70 px-3 bg-muted/20">
      <div class="flex items-center gap-1.5 min-w-0">
        <UTooltip :text="t('commit.backToWorkingTree')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-arrow-left"
            class="text-toned hover:text-highlighted rtl:rotate-180"
            :aria-label="t('commit.backToWorkingTree')"
            @click="emit('close')"
          />
        </UTooltip>
        <div class="h-3.5 w-px bg-default/80" aria-hidden="true" />
        <span class="text-xs font-semibold text-highlighted uppercase tracking-wider truncate">
          {{ t('commit.details') }}
        </span>
      </div>

      <div class="flex items-center gap-1 shrink-0">
        <UTooltip :text="t('commit.createBranchHere')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-git-branch"
            :aria-label="t('commit.createBranchHere')"
            @click="emit('createBranch', props.commitDetails.hash)"
          />
        </UTooltip>
        <UTooltip :text="t('commit.checkoutCommit')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-log-in"
            :aria-label="t('commit.checkoutCommit')"
            @click="emit('checkoutCommit', props.commitDetails.hash)"
          />
        </UTooltip>
      </div>
    </div>

    <!-- Commit Metadata Card -->
    <div class="p-3 border-b border-default/70 space-y-2.5 bg-surface/40">
      <!-- Subject & Hash -->
      <div class="flex items-start justify-between gap-2">
        <h3 class="text-xs font-bold text-highlighted leading-relaxed break-words flex-1">
          {{ props.commitDetails.subject }}
        </h3>
        <CommitHashPill :hash="props.commitDetails.hash" />
      </div>

      <!-- Extended commit body if present -->
      <p
        v-if="props.commitDetails.body"
        class="text-[11px] text-muted whitespace-pre-wrap font-sans bg-surface-subtle p-2 rounded border border-border-subtle"
      >
        {{ props.commitDetails.body }}
      </p>

      <!-- Author and Timestamp -->
      <div class="flex flex-wrap items-center gap-x-3 gap-y-1 text-[11px] text-muted">
        <div class="flex items-center gap-1">
          <UIcon name="i-lucide-user" class="w-3.5 h-3.5 text-toned" />
          <span class="font-medium text-highlighted">{{ props.commitDetails.author_name }}</span>
          <span class="text-muted/70">&lt;{{ props.commitDetails.author_email }}&gt;</span>
        </div>
        <div class="flex items-center gap-1">
          <UIcon name="i-lucide-clock" class="w-3.5 h-3.5 text-toned" />
          <span>{{ formatRelativeTime(props.commitDetails.author_timestamp) }}</span>
        </div>
      </div>

      <!-- Stats pill -->
      <div class="flex items-center gap-2 pt-0.5">
        <UBadge size="xs" color="neutral" variant="subtle" class="font-mono">
          {{ props.commitDetails.changed_files.length }} {{ t('commit.filesChanged') }}
        </UBadge>
        <UBadge
          v-if="props.commitDetails.total_additions > 0"
          size="xs"
          color="success"
          variant="soft"
          class="font-mono"
        >
          +{{ props.commitDetails.total_additions }}
        </UBadge>
        <UBadge
          v-if="props.commitDetails.total_deletions > 0"
          size="xs"
          color="error"
          variant="soft"
          class="font-mono"
        >
          -{{ props.commitDetails.total_deletions }}
        </UBadge>
      </div>
    </div>
  </div>
</template>
